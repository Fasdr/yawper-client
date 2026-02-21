use tokio::{
    select,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::messages::{client_message::ClientMessage, voice_message::VoiceMessage};

use super::{
    server_connection::ConnectionYawperClient,
    voice_channel::{voice_input::VoiceInput, voice_output::VoiceOutput},
};

pub struct BackendYawperClient {
    backend_commands_receiver: Receiver<ClientMessage>,
    gui_commands_transmitter: Sender<ClientMessage>,
    server_connection: Option<ConnectionYawperClient>,
    server_connection_is_active: bool,
    voice_input_control_transmitter: Option<Sender<VoiceMessage>>,
    voice_output_control_transmitter: Option<Sender<VoiceMessage>>,
}

impl BackendYawperClient {
    pub fn new(
        backend_commands_receiver: Receiver<ClientMessage>,
        gui_commands_transmitter: Sender<ClientMessage>,
    ) -> Self {
        Self {
            backend_commands_receiver,
            gui_commands_transmitter,
            server_connection: None,
            server_connection_is_active: false,
            voice_input_control_transmitter: None,
            voice_output_control_transmitter: None,
        }
    }

    pub async fn run(self: &mut Self) {
        loop {
            select! {
                Some(message) = self.backend_commands_receiver.recv() => {
                    self.process_gui_commands(message).await;
                }
                else => {
                    break;
                }
            }
        }
    }

    pub async fn process_gui_commands(self: &mut Self, message: ClientMessage) {
        match message {
            ClientMessage::ConnectToServer {
                host_name,
                host_password,
            } => {
                if !self.server_connection_is_active {
                    match ConnectionYawperClient::new(host_name, host_password).await {
                        Ok(new_server_connection) => {
                            self.server_connection = Some(new_server_connection);
                            self.server_connection_is_active = true;
                            let _ = self
                                .gui_commands_transmitter
                                .send(ClientMessage::ConnectionIsActive {})
                                .await;
                            if let Some(conn) = &self.server_connection {
                                conn.start_updates(self.gui_commands_transmitter.clone());
                            }
                        }
                        Err(err) => {
                            // TODO: provide this error to the gui
                            println!("{}", err)
                        }
                    }
                }
            }
            ClientMessage::CreateRoom {
                room_name,
                room_password,
            } => {
                if self.server_connection_is_active
                    && let Some(conn) = &self.server_connection
                {
                    match conn
                        .send_command(ClientMessage::CreateRoom {
                            room_name,
                            room_password,
                        })
                        .await
                    {
                        Ok(_) => {}
                        Err(err) => println!("Error during room creation: {}", err),
                    }
                }
            }
            ClientMessage::JoinRoom {
                room_name,
                room_password,
            } => {
                if self.server_connection_is_active
                    && let Some(conn) = &self.server_connection
                {
                    match conn
                        .send_command(ClientMessage::JoinRoom {
                            room_name: room_name.clone(),
                            room_password,
                        })
                        .await
                    {
                        Ok(_) => {
                            let _ = self
                                .gui_commands_transmitter
                                .send(ClientMessage::RoomJoined { room_name })
                                .await;

                            let (voice_input_control_transmitter, voice_input_control_receiver) =
                                mpsc::channel::<VoiceMessage>(100);
                            let connection_clone = conn.connection.clone();
                            match VoiceInput::new(voice_input_control_receiver, connection_clone) {
                                Ok(voice_input) => match voice_input.run() {
                                    Ok(_) => {
                                        self.voice_input_control_transmitter =
                                            Some(voice_input_control_transmitter);
                                    }
                                    Err(err) => {
                                        println!(
                                            "Error during voice input stream creation: {}",
                                            err
                                        )
                                    }
                                },
                                Err(err) => println!("Error during voice input creation: {}", err),
                            }
                            let (voice_output_control_transmitter, voice_output_control_receiver) =
                                mpsc::channel::<VoiceMessage>(100);
                            let mut voice_output_opt = None;
                            match VoiceOutput::new(voice_output_control_receiver) {
                                Ok(voice_output) => {
                                    voice_output_opt = Some(voice_output);
                                    self.voice_output_control_transmitter =
                                        Some(voice_output_control_transmitter);
                                }
                                Err(err) => println!("Error during voice output creation: {}", err),
                            }

                            conn.receive_datagrams(
                                voice_output_opt,
                                self.gui_commands_transmitter.clone(),
                            );
                        }
                        Err(err) => println!("Error during joining room: {}", err),
                    }
                }
            }
            ClientMessage::SetVoiceVolume { user_id, volume } => {
                if let Some(voice_output_control_transmitter) =
                    &self.voice_output_control_transmitter
                {
                    match voice_output_control_transmitter
                        .send(VoiceMessage::SetVoiceVolume { user_id, volume })
                        .await
                    {
                        Ok(_) => {}
                        Err(err) => println!("Error during changing volume level: {}", err),
                    }
                }
            }
            _ => {}
        }
    }
}
