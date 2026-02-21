use std::{error::Error, sync::Arc, time::Duration};

use tokio::{io::AsyncReadExt, sync::mpsc::Sender, time::sleep};
use wtransport::{ClientConfig, Connection, Endpoint};

use crate::messages::{
    client_message::ClientMessage, lobby_message::LobbyMessage, room_message::RoomMessage,
};

use super::voice_channel::voice_output::VoiceOutput;

pub struct ConnectionYawperClient {
    pub connection: Arc<Connection>,
}

impl ConnectionYawperClient {
    pub async fn new(host_name: String, host_password: String) -> Result<Self, Box<dyn Error>> {
        let config = ClientConfig::builder()
            .with_bind_default()
            .with_no_cert_validation()
            .keep_alive_interval(Some(Duration::from_secs(20)))
            .build();

        let connection = Endpoint::client(config)?
            .connect(host_name.as_str().trim())
            .await?;

        let (mut send, _) = connection.open_bi().await?.await?;
        send.write_all(host_password.as_str().trim().as_bytes())
            .await?;

        send.finish().await?;

        let msg = RoomMessage::TxtMessage {
            body: "Test message".to_string(),
            user_id: u64::MAX,
        };
        let mut send = connection.open_uni().await?.await?;
        let bytes = bincode::serialize(&msg).unwrap();
        send.write_all(&bytes).await?;
        send.finish().await?;

        let connection = Arc::new(connection);

        Ok(Self { connection })
    }

    pub fn start_updates(self: &Self, gui_commands_transmitter: Sender<ClientMessage>) {
        let connection = self.connection.clone();
        tokio::spawn(async move {
            loop {
                if gui_commands_transmitter.is_closed() {
                    return;
                }

                let (mut send, mut recv) = match connection.open_bi().await {
                    Ok(opening) => match opening.await {
                        Ok(val) => val,
                        Err(err) => {
                            println!("{}", err);
                            return;
                        }
                    },
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                };

                let msg = LobbyMessage::ListRooms {};
                match send.write_all(&bincode::serialize(&msg).unwrap()).await {
                    Ok(_) => {}
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                }
                match send.finish().await {
                    Ok(_) => {}
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                }

                let mut buffer = Vec::new();
                match recv.read_to_end(&mut buffer).await {
                    Ok(_) => {}
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                }

                match bincode::deserialize(&buffer) {
                    Ok(LobbyMessage::ListRoomsResult { rooms }) => {
                        let _ = gui_commands_transmitter
                            .send(ClientMessage::RoomList { rooms })
                            .await;
                    }
                    Err(err) => {
                        println!("{}", err);
                        return;
                    }
                    _ => {
                        println!("Couldn't deserialize room list!");
                        return;
                    }
                }

                sleep(Duration::from_millis(100)).await;
            }
        });
    }

    pub async fn send_command(self: &Self, message: ClientMessage) -> Result<(), Box<dyn Error>> {
        match message {
            ClientMessage::CreateRoom {
                room_name,
                room_password,
            } => {
                let msg = LobbyMessage::CreateRoom {
                    room_name: room_name.trim().to_string(),
                    password: room_password.trim().to_string(),
                };
                let (mut send, _) = self.connection.open_bi().await?.await?;
                let bytes = bincode::serialize(&msg)?;
                send.write_all(&bytes).await?;
                send.finish().await?;
                Ok(())
            }
            ClientMessage::JoinRoom {
                room_name,
                room_password,
            } => {
                let msg = LobbyMessage::JoinRoom {
                    room_name: room_name.trim().to_string(),
                    password: room_password.trim().to_string(),
                };
                let (mut send, mut recv) = self.connection.open_bi().await?.await?;
                let bytes = bincode::serialize(&msg).unwrap();
                send.write_all(&bytes).await?;
                send.finish().await?;

                let mut buffer = Vec::new();
                recv.read_to_end(&mut buffer).await?;

                let room_message: RoomMessage = bincode::deserialize(&buffer).unwrap();

                if let RoomMessage::Connected {} = room_message {
                    Ok(())
                } else {
                    Err("Server didn't connect to the room".into())
                }
            }
            _ => Ok(()),
        }
    }

    pub fn receive_datagrams(
        self: &Self,
        mut voice_output_opt: Option<VoiceOutput>,
        gui_commands_transmitter_clone: Sender<ClientMessage>,
    ) {
        let connection_clone = self.connection.clone();
        tokio::spawn(async move {
            loop {
                match connection_clone.receive_datagram().await {
                    Ok(data) => {
                        let message: RoomMessage = bincode::deserialize(&data).unwrap();
                        match message {
                            RoomMessage::VoicePacket {
                                body,
                                order_id,
                                user_id,
                            } => {
                                if let Some(voice_output) = &mut voice_output_opt {
                                    let added_voice_channel =
                                        voice_output.accept_packet(body, order_id, user_id);
                                    if added_voice_channel != u64::MAX {
                                        let _ = gui_commands_transmitter_clone
                                            .send(ClientMessage::NewVoiceChannel {
                                                user_id: added_voice_channel,
                                            })
                                            .await;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    Err(err) => {
                        println!("Error during receiving datagram: {}", err);
                        break;
                    }
                }
            }
        });
    }
}
