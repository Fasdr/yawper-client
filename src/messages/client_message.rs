pub enum ClientMessage {
    ConnectionIsActive {},
    ConnectToServer {
        host_name: String,
        host_password: String,
    },
    CreateRoom {
        room_name: String,
        room_password: String,
    },
    JoinRoom {
        room_name: String,
        room_password: String,
    },
    RoomJoined {
        room_name: String,
    },
    RoomList {
        rooms: Vec<String>,
    },
    NewVoiceChannel {
        user_id: u64,
    },
    SetVoiceVolume {
        user_id: u64,
        volume: f32,
    },
}
