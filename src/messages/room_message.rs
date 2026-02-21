use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum RoomMessage {
    Empty {},
    Connected {},
    NotConnected {},
    AcceptUser {},
    RemoveUser {},
    RoomEntered {},
    TxtMessage {
        body: String,
        user_id: u64,
    },
    VoicePacket {
        body: Vec<u8>,
        order_id: u64,
        user_id: u64,
    },
}
