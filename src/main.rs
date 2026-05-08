mod camera;
mod point;
mod sim;

use eframe::egui::{
    self,
    ahash::{HashSet, HashSetExt},
};

use crate::{camera::*, sim::*};

// TODO: rather than just having a drag handle at the center,
// click and drag on square boundary to apply a force.

fn main() -> eframe::Result {
    let mut sim = Sim::new();

    let mut camera = Camera::new(0.0, 0.0, {
        let aabb = sim.big_square.aabb();
        // really this should be based on the aspect ratio of the rect,
        // but i don't have access to it yet,
        // and this is good enough.
        aabb.real_rad().max(aabb.imag_rad()) * 2.0
    });

    // the handle we're currently dragging or rotating.
    // the mouse is down on this.
    let mut active_handle = None;

    let mut k_c = 1.0;

    // in world coords.
    let mut offset_radius = 0.4;

    let mut draw_gauss_map = true;

    eframe::run_ui_native(
        "packing",
        eframe::NativeOptions::default(),
        move |ui, _frame| {
            ui.ctx().request_repaint();
            egui::Panel::left("left")
                .resizable(false)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.group(|ui| {
                            ui.label("camera controls");
                            ui.add(
                                egui::Slider::new(&mut camera.real_mid, -10.0..=10.0)
                                    .text("center x")
                                    .clamping(egui::SliderClamping::Never),
                            );
                            ui.add(
                                egui::Slider::new(&mut camera.imag_mid, -10.0..=10.0)
                                    .text("center y")
                                    .clamping(egui::SliderClamping::Never),
                            );
                            ui.add(
                                egui::Slider::new(&mut camera.real_rad, 0.1..=10.0)
                                    .text("radius")
                                    .clamping(egui::SliderClamping::Never),
                            );
                        });

                        ui.group(|ui| {
                            ui.label("sim settings");
                            ui.add(
                                egui::Slider::new(&mut k_c, 0.1..=1000.0)
                                    .text("k_c")
                                    .clamping(egui::SliderClamping::Never)
                                    .logarithmic(true),
                            );

                            ui.add(
                                egui::Slider::new(&mut offset_radius, 0.0..=1.0)
                                    .text("offset radius")
                                    .clamping(egui::SliderClamping::Never),
                            );

                            if ui.button("step").clicked() {
                                // sim.step(k_c, offset_radius);
                                let impulses = sim.get_impulse_builders();
                                sim.apply_impulse_builders(&impulses);
                            }
                        });

                        ui.group(|ui| {
                            ui.label("debug visualizations");
                            ui.checkbox(&mut draw_gauss_map, "draw gauss map");
                        });
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

                // apply the interaction if there is an `active_handle`.
                if let Some(active_handle) = active_handle {
                    let mouse_pos = ui
                        .input(|i| i.pointer.hover_pos())
                        .expect("active_handle implies mouse_pos");
                    let world_pos = camera_map.screen_to_world(mouse_pos);
                    match active_handle {
                        HandleIndex::Dragging(square_index) => {
                            sim.get_mut(square_index).center = world_pos
                        }
                        HandleIndex::Resizing(square_index) => {
                            // minimize the distance between square.resize_handle() and world_pos.
                            // square.normal = proj(world_pos - square.center, square.normal)
                            let square = sim.get_mut(square_index);
                            square.normal = square.normal
                                * square.normal.dot(world_pos - square.center)
                                / square.normal.length_sq();
                        }
                        HandleIndex::Rotating(square_index) => {
                            // minimize the distance between square.rotate_handle() and world_pos.
                            // new_normal + new_tangent = new_corner;
                            // new_tangent = Vec2 { x: -new_normal.y, y: new_normal.x }
                            // Vec2 { x: new_normal.x - new_normal.y, y: new_normal.y + new_normal.x } = new_corner
                            let square = sim.get_mut(square_index);
                            let new_corner = (square.normal + square.tangent()).length()
                                * (world_pos - square.center).normalized();
                            square.normal = (new_corner + new_corner.cw()) / 2.0;
                        }
                    }
                }

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

                // draw gauss map
                #[cfg(false)]
                if draw_gauss_map && let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let world_pos = camera_map.screen_to_world(mouse_pos);
                    for (_, square) in sim.enumerate_squares() {
                        let closest_point = square.nearest_point(world_pos).inner().0;
                        if let Some(normal) =
                            square.gauss_map_offset_radius(world_pos, offset_radius)
                        {
                            // ui.painter().arrow(
                            //     camera_map.world_to_screen(closest_point),
                            //     camera_map.delta_complex_to_vec2(normal),
                            //     egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 100, 100)),
                            // );
                            ui.painter().arrow(
                                mouse_pos,
                                camera_map.delta_complex_to_vec2(normal),
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 100, 100)),
                            );
                            ui.painter().circle(
                                camera_map.world_to_screen(closest_point),
                                5.0,
                                egui::Color32::from_rgb(255, 100, 100),
                                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 100, 100)),
                            );
                        }
                    }
                }

                // highlight the vertices and edges whose blocks contain the mouse
                if draw_gauss_map && let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
                    let world_pos = camera_map.screen_to_world(mouse_pos);
                    for (_, square) in sim.enumerate_squares() {
                        for i in 0..4 {
                            // vertex
                            if square.vertex_block_contains(offset_radius, i, world_pos) {
                                ui.painter().circle(
                                    camera_map.world_to_screen(square.vertices()[i]),
                                    5.0,
                                    egui::Color32::from_rgb(100, 255, 100),
                                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 255, 100)),
                                );
                            }

                            // edge
                            if square.edge_block_contains(offset_radius, i, world_pos) {
                                let v1 = square.vertices()[i];
                                let v2 = square.vertices()[(i + 1) % 4];
                                ui.painter().line_segment(
                                    [
                                        camera_map.world_to_screen(v1),
                                        camera_map.world_to_screen(v2),
                                    ],
                                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 255, 100)),
                                );
                            }
                        }
                    }
                }

                // draw impulses
                {
                    let impulse_builders = sim.get_impulse_builders();
                    for (square, impulses) in sim.small_squares.iter().zip(impulse_builders.iter())
                    {
                        for &builder in impulses {
                            match builder {
                                SquareImpulseBuilder::Vertex {
                                    vertex_index: i,
                                    impulse,
                                } => {
                                    // arrow from square.vertex(i) in the direction of impulse.
                                    let vertex = square.vertices()[i];
                                    ui.painter().arrow(
                                        camera_map.world_to_screen(vertex),
                                        camera_map.delta_complex_to_vec2(impulse),
                                        egui::Stroke::new(
                                            2.0,
                                            egui::Color32::from_rgb(255, 100, 100),
                                        ),
                                    );
                                }
                                SquareImpulseBuilder::Edge {
                                    edge_index: i,
                                    point,
                                    impulse,
                                } => {
                                    // arrow from the nearest point on the edge to `point` in the direction of impulse.
                                    let edge = square.get_edge(i);
                                    let nearest_point = edge.nearest_point(point).unwrap();
                                    ui.painter().arrow(
                                        camera_map.world_to_screen(nearest_point),
                                        camera_map.delta_complex_to_vec2(impulse),
                                        egui::Stroke::new(
                                            2.0,
                                            egui::Color32::from_rgb(255, 100, 100),
                                        ),
                                    );
                                }
                            }
                        }
                    }

                    let stepped = {
                        let mut sim = sim.clone();
                        sim.apply_impulse_builders(&impulse_builders);
                        sim
                    };
                    // draw stepped as ghosts
                    stepped.enumerate_squares().for_each(|(i, square)| {
                        let vertices = square.vertices().map(|v| camera_map.world_to_screen(v));
                        ui.painter().add(egui::Shape::convex_polygon(
                            vertices.to_vec(),
                            egui::Color32::TRANSPARENT,
                            egui::epaint::PathStroke {
                                width: 2.0,
                                color: eframe::epaint::ColorMode::Solid(
                                    egui::Color32::from_rgba_unmultiplied(255, 100, 100, 100),
                                ),
                                kind: egui::StrokeKind::Middle,
                            },
                        ));
                    });
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
