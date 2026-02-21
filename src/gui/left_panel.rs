use crate::messages::client_message::ClientMessage;

use super::app::EguiYawperClient;

impl EguiYawperClient {
    pub fn yawper_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("my_right_side_panel").show(ctx, |ui| {
            if !self.connected_to_host {
                ui.heading("Server Login:");
                ui.add(egui::TextEdit::singleline(&mut self.host_name).hint_text("Host"));
                ui.add(
                    egui::TextEdit::singleline(&mut self.host_password)
                        .hint_text("Password")
                        .password(true),
                );
                if ui.button("Connect").clicked() {
                    let message = ClientMessage::ConnectToServer {
                        host_name: self.host_name.clone(),
                        host_password: self.host_password.clone(),
                    };
                    let _ = self.backend_commands_transmitter.try_send(message);
                }
                ui.separator();
            } else {
                ui.heading("Server Login:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.host_name)
                        .hint_text("Host")
                        .interactive(false),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.host_password)
                        .hint_text("Password")
                        .password(true)
                        .interactive(false),
                );
                if ui.button("Disconnect").clicked() {}
                ui.separator();

                let create_room_id = ui.make_persistent_id("create_room_header");
                let mut create_room_state =
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        create_room_id,
                        false,
                    );
                if let Some(set_create_room) = self.create_room_show {
                    create_room_state.set_open(set_create_room);
                    self.create_room_show = None;
                }
                create_room_state
                    .show_header(ui, |ui| {
                        ui.label("Create Room:");
                    })
                    .body(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_room_name).hint_text("Name"),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.new_room_password)
                                .hint_text("Password")
                                .password(true),
                        );
                        if ui.button("Create").clicked() {
                            if !self.new_room_name.is_empty()
                                && !self.rooms.contains(&self.new_room_name)
                            {
                                let message = ClientMessage::CreateRoom {
                                    room_name: self.new_room_name.clone(),
                                    room_password: self.new_room_password.clone(),
                                };
                                let _ = self.backend_commands_transmitter.try_send(message);
                                self.create_room_show = Some(false);
                            }
                        }
                    });

                // ui.collapsing("Create Room:", |ui| {
                //     ui.add(egui::TextEdit::singleline(&mut self.new_room_name).hint_text("Name"));
                //     ui.add(
                //         egui::TextEdit::singleline(&mut self.new_room_password)
                //             .hint_text("Password")
                //             .password(true),
                //     );
                //     if ui.button("Create").clicked() {
                //         if !self.new_room_name.is_empty()
                //             && !self.rooms.contains(&self.new_room_name)
                //         {
                //             let message = ClientMessage::CreateRoom {
                //                 room_name: self.new_room_name.clone(),
                //                 room_password: self.new_room_password.clone(),
                //             };
                //             let _ = self.backend_commands_transmitter.try_send(message);
                //         }
                //     }
                // });

                let join_room_id = ui.make_persistent_id("join_room_header");
                let mut join_room_state =
                    egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        join_room_id,
                        false,
                    );
                if let Some(set_join_room) = self.join_room_show {
                    join_room_state.set_open(set_join_room);
                    self.join_room_show = None;
                }

                join_room_state
                    .show_header(ui, |ui| {
                        ui.label("Join Room:");
                    })
                    .body(|ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut self.join_room_name).hint_text("Name"),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut self.join_room_password)
                                .hint_text("Password")
                                .password(true),
                        );
                        if ui.button("Join").clicked() {
                            if !self.join_room_name.is_empty()
                                && self.rooms.contains(&self.join_room_name)
                            {
                                let message = ClientMessage::JoinRoom {
                                    room_name: self.join_room_name.clone(),
                                    room_password: self.join_room_password.clone(),
                                };
                                let _ = self.backend_commands_transmitter.try_send(message);
                            }
                        }
                    });

                ui.separator();
                ui.heading("Room List:");
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for room in &self.rooms {
                            let current_room_joined = room == &self.active_room;
                            if ui
                                .selectable_label(
                                    current_room_joined,
                                    room.clone()
                                        + if current_room_joined {
                                            " - joined"
                                        } else {
                                            " "
                                        },
                                )
                                .clicked()
                            {
                                if !self.in_room {
                                    self.join_room_name = room.clone();
                                    self.join_room_show = Some(true);
                                }
                            }
                        }
                    });
                ui.separator();
            }
        });
    }
}
