pub enum VoiceMessage {
    CloseVoiceInput {},
    SetVoiceVolume { user_id: u64, volume: f32 },
}
