mod camera;
mod point;
mod sim;

use eframe::egui::{
    self,
    ahash::{HashSet, HashSetExt},
};

use crate::{camera::*, sim::*};

fn main() -> eframe::Result {
    let mut sim = Sim::new();

    let camera = Camera::new(0.0, 0.0, {
        let aabb = sim.big_square.aabb();
        // really this should be based on the aspect ratio of the rect,
        // but i don't have access to it yet,
        // and this is good enough.
        aabb.real_rad().max(aabb.imag_rad()) * 2.0
    });

    // the handle we're currently dragging or rotating.
    // the mouse is down on this.
    let mut active_handle = None;

    eframe::run_ui_native(
        "packing",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            ui.ctx().request_repaint();
            egui::Panel::left("left")
                .resizable(false)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.label("settings");
                    })
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                let camera_map = CameraMap::new(ui.available_rect_before_wrap(), camera);

                /// in egui coords.
                const HOVERED_HANDLE_RADIUS: f32 = 7.0;
                /// in egui coords.
                const UNHOVERED_HANDLE_RADIUS: f32 = 4.0;

                // if the mouse is up, clear the current `active_handle`.
                if !ui.input(|i| i.pointer.primary_down()) {
                    active_handle = None;
                }

                // if there is an `active_handle`, the hovered handle is the `active_handle`.
                // otherwise, take the handle nearest to the mouse that's inside `HOVERED_HANDLE_RADIUS`.
                let hovered_handle = if let Some(active_handle) = active_handle {
                    Some(active_handle)
                } else {
                    ui.input(|i| i.pointer.hover_pos()).and_then(|mouse_pos| {
                        sim.enumerate_handles()
                            .map(|(i, handle_world_pos)| {
                                let handle_screen_pos =
                                    camera_map.world_to_screen(handle_world_pos);
                                let dist_sq = handle_screen_pos.distance_sq(mouse_pos);
                                (i, dist_sq)
                            })
                            .filter(|(i, dist_sq)| {
                                *dist_sq <= HOVERED_HANDLE_RADIUS * HOVERED_HANDLE_RADIUS
                            })
                            .min_by(|(_, lhs), (_, rhs)| lhs.partial_cmp(rhs).unwrap())
                            .map(|(i, _dist_sq)| i)
                    })
                };

                // if the mouse was pressed, set the current interaction to `hovered_handle`.
                // TODO: should this be on mouse down or mouse press?
                if ui.input(|i| i.pointer.primary_pressed()) {
                    assert!(active_handle.is_none(), "mouse is up and down");
                    active_handle = hovered_handle;
                }

                // the squares containing the mouse
                // TODO: rename
                let hovered_squares = ui
                    .input(|i| i.pointer.hover_pos())
                    .map(|mouse_pos| {
                        sim.enumerate_squares()
                            .filter(|(_, square)| {
                                square.contains(camera_map.screen_to_world(mouse_pos))
                            })
                            .map(|(i, _)| i)
                            .collect()
                    })
                    .unwrap_or(HashSet::new());

                // draw squares (fill and stroke)
                {
                    /// if the mouse is inside this square.
                    /// multiple squares can have this.
                    /// this is the color of the fill.
                    const HOVERED_INTERIOR_FILL_COLOR: egui::Color32 =
                        egui::Color32::from_rgba_unmultiplied_const(150, 150, 150, 20);
                    const UNHOVERED_INTERIOR_FILL_COLOR: egui::Color32 = egui::Color32::TRANSPARENT;

                    /// if the hovered handle belongs to this square.
                    /// at most one square can have this.
                    /// this is the color of the stroke.
                    // TODO: maybe also have a color for the `active_handle`.
                    const HOVERED_HANDLE_STROKE_COLOR: egui::Color32 =
                        egui::Color32::from_gray(255);
                    const UNHOVERED_HANDLE_STROKE_COLOR: egui::Color32 =
                        egui::Color32::from_gray(150);

                    // TODO: shade / highlight illegal overlaps.

                    for (i, square) in sim.enumerate_squares() {
                        let vertices = square.vertices().map(|v| camera_map.world_to_screen(v));
                        ui.painter().add(egui::Shape::convex_polygon(
                            vertices.to_vec(),
                            if i != SquareIndex::Big && hovered_squares.contains(&i) {
                                HOVERED_INTERIOR_FILL_COLOR
                            } else {
                                UNHOVERED_INTERIOR_FILL_COLOR
                            },
                            egui::epaint::PathStroke {
                                width: 2.0,
                                color: eframe::epaint::ColorMode::Solid(
                                    if let Some(hovered_handle) = hovered_handle
                                        && hovered_handle.square_index() == i
                                    {
                                        HOVERED_HANDLE_STROKE_COLOR
                                    } else {
                                        UNHOVERED_HANDLE_STROKE_COLOR
                                    },
                                ),
                                kind: egui::StrokeKind::Middle,
                            },
                        ));
                    }
                }

                // draw the interaction handles (on top of the squares)
                {
                    const ACTIVE_HANDLE_COLOR: egui::Color32 =
                        egui::Color32::from_rgb(150, 150, 255);
                    const INACTIVE_HANDLE_COLOR: egui::Color32 =
                        egui::Color32::from_rgb(100, 100, 200);

                    // draw the handles.
                    // if it's inactive, draw it gray and small.
                    // if it's hovered but inactive, draw it gray and big.
                    // if it's active, draw it white and big.
                    for (i, handle) in sim.enumerate_handles() {
                        let screen_pos = camera_map.world_to_screen(handle);

                        ui.painter().add(egui::Shape::circle_filled(
                            screen_pos,
                            if Some(i) == hovered_handle {
                                HOVERED_HANDLE_RADIUS
                            } else {
                                UNHOVERED_HANDLE_RADIUS
                            },
                            if Some(i) == active_handle {
                                ACTIVE_HANDLE_COLOR
                            } else {
                                INACTIVE_HANDLE_COLOR
                            },
                        ));
                    }
                }
            });
        },
    )
}

pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn inv_lerp_f32(a: f32, b: f32, v: f32) -> f32 {
    (v - a) / (b - a)
}

pub fn lerp_f64(a: f64, b: f64, t: f64) -> f64 {
    // assert!((0.0..=1.0).contains(&t));
    a + (b - a) * t
}

pub fn inv_lerp_f64(a: f64, b: f64, v: f64) -> f64 {
    // assert!((a..=b).contains(&v));
    (v - a) / (b - a)
}
