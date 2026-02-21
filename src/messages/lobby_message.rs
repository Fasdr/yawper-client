use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum LobbyMessage {
    Empty {},
    CreateRoom { room_name: String, password: String },
    ListRooms {},
    ListRoomsResult { rooms: Vec<String> },
    JoinRoom { room_name: String, password: String },
    ExitRoom {},
}
