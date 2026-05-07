mod camera;
mod point;
mod sim;

use eframe::egui;

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

                // square outlines
                {
                    // big square
                    {
                        ui.painter().add(egui::Shape::closed_line(
                            sim.big_square
                                .vertices()
                                .map(|v| camera_map.world_to_screen(v))
                                .to_vec(),
                            egui::Stroke {
                                width: 2.0,
                                color: egui::Color32::WHITE,
                            },
                        ));
                    }

                    // small squares
                    for square in &sim.small_squares {
                        ui.painter().add(egui::Shape::closed_line(
                            square
                                .vertices()
                                .map(|v| camera_map.world_to_screen(v))
                                .to_vec(),
                            egui::Stroke {
                                width: 2.0,
                                color: egui::Color32::WHITE,
                            },
                        ));
                    }
                }

                // interaction handles
                {
                    /// in egui coords.
                    const HOVERED_HANDLE_RADIUS: f32 = 7.0;
                    /// in egui coords.
                    const UNHOVERED_HANDLE_RADIUS: f32 = 3.0;

                    // note that active kinda implies hovered, but due to frame stuff it might not actually be hovered,
                    // so we need to persist the active handle across frames.

                    // `None` if no handles are inside `ACTIVE_HANDLE_RADIUS`.
                    // if multiple handles are inside `ACTIVE_HANDLE_RADIUS`, take the nearest.
                    let nearest_handle =
                        ui.input(|i| i.pointer.hover_pos()).and_then(|mouse_pos| {
                            // // TODO: only compute if there isn't an active handle?
                            // // TODO: refactor this
                            // let drag_handle = sim
                            //     .enumerate()
                            //     .map(|(i, square)| {
                            //         (i, camera_map.world_to_screen(square.drag_handle()))
                            //     })
                            //     .filter(|(i, screen_pos)| {
                            //         screen_pos.distance(mouse_pos) <= HOVERED_HANDLE_RADIUS
                            //     })
                            //     .min_by(|(_, screen_pos_lhs), (_, screen_pos_rhs)| {
                            //         let dist_sq_lhs = screen_pos_lhs.distance_sq(mouse_pos);
                            //         let dist_sq_rhs = screen_pos_rhs.distance_sq(mouse_pos);
                            //         dist_sq_lhs.partial_cmp(&dist_sq_rhs).unwrap()
                            //     });

                            // let resize_handle = sim
                            //     .enumerate()
                            //     .map(|(i, square)| {
                            //         (i, camera_map.world_to_screen(square.resize_handle()))
                            //     })
                            //     .filter(|(i, screen_pos)| {
                            //         screen_pos.distance(mouse_pos) <= HOVERED_HANDLE_RADIUS
                            //     })
                            //     .min_by(|(_, screen_pos_lhs), (_, screen_pos_rhs)| {
                            //         let dist_sq_lhs = screen_pos_lhs.distance_sq(mouse_pos);
                            //         let dist_sq_rhs = screen_pos_rhs.distance_sq(mouse_pos);
                            //         dist_sq_lhs.partial_cmp(&dist_sq_rhs).unwrap()
                            //     });

                            // let rotate_handle = sim
                            //     .enumerate()
                            //     .map(|(i, square)| {
                            //         (i, camera_map.world_to_screen(square.rotate_handle()))
                            //     })
                            //     .filter(|(i, screen_pos)| {
                            //         screen_pos.distance(mouse_pos) <= HOVERED_HANDLE_RADIUS
                            //     })
                            //     .min_by(|(_, screen_pos_lhs), (_, screen_pos_rhs)| {
                            //         let dist_sq_lhs = screen_pos_lhs.distance_sq(mouse_pos);
                            //         let dist_sq_rhs = screen_pos_rhs.distance_sq(mouse_pos);
                            //         dist_sq_lhs.partial_cmp(&dist_sq_rhs).unwrap()
                            //     });

                            // match (drag_handle, rotate_handle) {
                            //     (
                            //         Some((drag_i, drag_screen_pos)),
                            //         Some((rotate_i, rotate_screen_pos)),
                            //     ) => {
                            //         if drag_screen_pos.distance_sq(mouse_pos)
                            //             <= rotate_screen_pos.distance_sq(mouse_pos)
                            //         {
                            //             Some(Interaction::Dragging(drag_i))
                            //         } else {
                            //             Some(Interaction::Rotating(rotate_i))
                            //         }
                            //     }
                            //     (Some((drag_i, drag_screen_pos)), None) => {
                            //         Some(Interaction::Dragging(drag_i))
                            //     }
                            //     (None, Some((rotate_i, rotate_screen_pos))) => {
                            //         Some(Interaction::Rotating(rotate_i))
                            //     }
                            //     (None, None) => None,
                            // }

                            // let mut best = drag_handle;
                            // if let Some((resize_i, resize_screen_pos)) = resize_handle {
                            //     if let Some((best_i, best_screen_pos)) = best {
                            //         if resize_screen_pos.distance_sq(mouse_pos)
                            //             <= best_screen_pos.distance_sq(mouse_pos)
                            //         {
                            //             Some((resize_i, resize_screen_pos))
                            //         } else {
                            //             best
                            //         }
                            //     } else {
                            //         Some((resize_i, resize_screen_pos))
                            //     }
                            // } else {
                            //     best
                            // };

                            // best
                            // nearest_handle, nearest_screen_pos

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
                        });

                    // if the mouse is up, clear the current interaction.
                    if !ui.input(|i| i.pointer.primary_down()) {
                        active_handle = None;
                    }

                    // if the mouse was pressed, set the current interaction to `nearest_handle`.
                    if ui.input(|i| i.pointer.primary_pressed()) {
                        assert!(active_handle.is_none(), "mouse is up and down");
                        active_handle = nearest_handle;
                    }

                    // override the nearest_handle with the active_handle.
                    // it's possibly that the handle that's nearest to the mouse isn't the one we're dragging,
                    // but for the upcoming logic, we want to pretend that it is.
                    let nearest_handle = if let Some(active_handle) = active_handle {
                        Some(active_handle)
                    } else {
                        nearest_handle
                    };

                    const ACTIVE_HANDLE_COLOR: egui::Color32 = egui::Color32::WHITE;
                    const INACTIVE_HANDLE_COLOR: egui::Color32 = egui::Color32::GRAY;

                    // draw the handles.
                    // if it's inactive, draw it gray and small.
                    // if it's hovered but inactive, draw it gray and big.
                    // if it's active, draw it white and big.
                    for (i, handle) in sim.enumerate_handles() {
                        let screen_pos = camera_map.world_to_screen(handle);

                        ui.painter().add(egui::Shape::circle_filled(
                            screen_pos,
                            if Some(i) == nearest_handle {
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

                    // // draw the rotation handles.
                    // // if it's inactive, draw it gray and small.
                    // // if it's hovered but inactive, draw it gray and big.
                    // // if it's active, draw it white and big.
                    // for (i, square) in sim.enumerate() {
                    //     let screen_pos = camera_map.world_to_screen(square.rotate_handle());

                    //     ui.painter().add(egui::Shape::circle_filled(
                    //         screen_pos,
                    //         if Some(HandleIndex::Rotating(i)) == nearest_handle {
                    //             HOVERED_HANDLE_RADIUS
                    //         } else {
                    //             UNHOVERED_HANDLE_RADIUS
                    //         },
                    //         if Some(HandleIndex::Rotating(i)) == active_handle {
                    //             ACTIVE_HANDLE_COLOR
                    //         } else {
                    //             INACTIVE_HANDLE_COLOR
                    //         },
                    //     ));
                    // }
                }

                // TODO: shade / highlight squares if the mouse is inside them
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
