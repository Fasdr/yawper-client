use audiopus::{Channels, SampleRate, coder::Decoder as OpusDecoder};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::HeapRb;
use ringbuf::traits::{Consumer, Producer, Split};
use tokio::sync::mpsc::Receiver;

use std::collections::HashMap;
use std::u64;

use crossbeam_channel::unbounded;
use ringbuf::{HeapCons, HeapProd};

use crate::messages::voice_message::VoiceMessage;

type AudioSource = HeapCons<f32>;

const SAMPLE_RATE: u32 = 48000;
const CHANNELS: usize = 2;

pub struct VoiceOutput {
    _output_stream: cpal::Stream,
    queue_sender: crossbeam_channel::Sender<AudioSource>,
    user_sender: HashMap<u64, (HeapProd<f32>, OpusDecoder, u64, f32)>,
    voice_output_control_receiver: Receiver<VoiceMessage>,
}

impl VoiceOutput {
    pub fn new(
        voice_output_control_receiver: Receiver<VoiceMessage>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let output_device = host
            .default_output_device()
            .expect("No output device found");

        let config = cpal::StreamConfig {
            channels: CHANNELS as u16,
            sample_rate: SAMPLE_RATE,
            buffer_size: cpal::BufferSize::Default,
        };

        let (tx, rx) = unbounded::<AudioSource>();

        let mut active_sources: Vec<AudioSource> = Vec::new();

        let output_data_fn = move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            while let Ok(new_source) = rx.try_recv() {
                active_sources.push(new_source);
            }
            data.fill(0.0);
            for sample in data {
                for consumer in active_sources.iter_mut() {
                    *sample = consumer.try_pop().unwrap_or(0.0);
                }
                *sample = sample.clamp(-1.0, 1.0);
            }
        };

        let output_stream =
            output_device.build_output_stream(&config, output_data_fn, _err_fn, None)?;
        output_stream.play()?;

        Ok(Self {
            _output_stream: output_stream,
            queue_sender: tx,
            user_sender: HashMap::new(),
            voice_output_control_receiver,
        })
    }

    pub fn accept_packet(self: &mut Self, body: Vec<u8>, order_id: u64, user_id: u64) -> u64 {
        let mut added_new_user = u64::MAX;
        if !self.user_sender.contains_key(&user_id) {
            let ring_buffer_len = SAMPLE_RATE as usize * CHANNELS;
            let ring = HeapRb::<f32>::new(ring_buffer_len);
            let (producer, consumer) = ring.split();
            let encoder = OpusDecoder::new(SampleRate::Hz48000, Channels::Stereo).unwrap();
            let _ = self
                .user_sender
                .insert(user_id, (producer, encoder, 0u64, 1.0));
            let _ = self.queue_sender.send(consumer);
            added_new_user = user_id;
        }
        if self.user_sender.get_mut(&user_id).unwrap().2 > order_id {
            return added_new_user;
        }
        loop {
            match self.voice_output_control_receiver.try_recv() {
                Ok(message) => match message {
                    VoiceMessage::SetVoiceVolume { user_id, volume } => {
                        self.user_sender.get_mut(&user_id).unwrap().3 = volume;
                    }
                    _ => {}
                },
                Err(_) => {
                    break;
                }
            }
        }
        self.user_sender.get_mut(&user_id).unwrap().2 = order_id;

        let mut output_buffer = [0.0f32; 5760];

        let samples_decoded = match self.user_sender.get_mut(&user_id).unwrap().1.decode_float(
            Some(&body),
            &mut output_buffer[..],
            false,
        ) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Decode error: {:?}", e);
                return added_new_user;
            }
        };
        let decoded_slice = &mut output_buffer[0..samples_decoded * CHANNELS];
        let volume = self.user_sender.get_mut(&user_id).unwrap().3;
        for sample in decoded_slice.iter_mut() {
            *sample *= volume;
        }
        let _ = self
            .user_sender
            .get_mut(&user_id)
            .unwrap()
            .0
            .push_slice(decoded_slice);

        added_new_user
    }
}

fn _err_fn(err: cpal::StreamError) {
    eprintln!("Stream error: {}", err);
}
