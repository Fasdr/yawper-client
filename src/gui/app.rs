use tokio::sync::mpsc::{Receiver, Sender};

use crate::messages::client_message::ClientMessage;

pub struct EguiYawperClient {
    pub host_name: String,
    pub host_password: String,
    pub connected_to_host: bool,
    pub create_room_show: Option<bool>,
    pub new_room_name: String,
    pub new_room_password: String,
    pub join_room_show: Option<bool>,
    pub join_room_name: String,
    pub join_room_password: String,
    pub rooms: Vec<String>,
    pub active_room: String,
    pub in_room: bool,
    pub voice_channel_list: Vec<(u64, f32)>,
    pub backend_commands_transmitter: Sender<ClientMessage>,
    pub gui_commands_receiver: Receiver<ClientMessage>,
}

impl EguiYawperClient {
    pub fn new(
        backend_commands_transmitter: Sender<ClientMessage>,
        gui_commands_receiver: Receiver<ClientMessage>,
    ) -> Self {
        Self {
            host_name: String::new(),
            host_password: String::new(),
            connected_to_host: false,
            create_room_show: None,
            new_room_name: String::new(),
            new_room_password: String::new(),
            join_room_show: None,
            join_room_name: String::new(),
            join_room_password: String::new(),
            rooms: Vec::new(),
            active_room: String::new(),
            in_room: false,
            voice_channel_list: Vec::new(),
            backend_commands_transmitter,
            gui_commands_receiver,
        }
    }
}

impl eframe::App for EguiYawperClient {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(message) = self.gui_commands_receiver.try_recv() {
            match message {
                ClientMessage::ConnectionIsActive {} => self.connected_to_host = true,
                ClientMessage::RoomList { rooms } => self.rooms = rooms,
                ClientMessage::RoomJoined { room_name } => {
                    self.active_room = room_name;
                    self.in_room = true;
                    self.join_room_name.clear();
                    self.join_room_password.clear();
                    self.join_room_show = Some(false);
                }
                ClientMessage::NewVoiceChannel { user_id } => {
                    self.voice_channel_list.push((user_id, 1.0));
                }
                _ => {}
            }
        }
        self.yawper_left_panel(ctx);
        self.yawper_right_panel(ctx);
    }
}
