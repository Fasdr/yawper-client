use crate::messages::client_message::ClientMessage;

use super::app::EguiYawperClient;

impl EguiYawperClient {
    pub fn yawper_right_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("my_left_side_panel").show(ctx, |ui| {
            if self.in_room {
                ui.heading("Room Voice Level:");
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (user_id, volume) in self.voice_channel_list.iter_mut() {
                            ui.horizontal(|ui| {
                                ui.label("User ".to_owned() + user_id.to_string().as_str() + ":");
                                let response = ui.add(
                                    egui::Slider::new(volume, 0.0..=4.0)
                                        .text("Volume")
                                        .custom_formatter(|n, _| {
                                            format!("{}%", (n * 100.0) as i32)
                                        }),
                                );

                                if response.drag_stopped() {
                                    match self.backend_commands_transmitter.try_send(
                                        ClientMessage::SetVoiceVolume {
                                            user_id: *user_id,
                                            volume: *volume,
                                        },
                                    ) {
                                        Ok(_) => {}
                                        Err(err) => {
                                            println!("Error during sending user id volume: {}", err)
                                        }
                                    }
                                }

                                if *volume != 1.0 {
                                    if ui.button("R").on_hover_text("Reset to 100%").clicked() {
                                        *volume = 1.0;
                                        match self.backend_commands_transmitter.try_send(
                                            ClientMessage::SetVoiceVolume {
                                                user_id: *user_id,
                                                volume: *volume,
                                            },
                                        ) {
                                            Ok(_) => {}
                                            Err(err) => {
                                                println!(
                                                    "Error during sending user id volume: {}",
                                                    err
                                                )
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    });
            }
        });
    }
}
