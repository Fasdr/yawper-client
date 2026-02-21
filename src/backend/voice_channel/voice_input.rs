use std::sync::Arc;

use audiopus::{Application, Channels, SampleRate, coder::Encoder as OpusEncoder};
use cpal::Stream;
use ringbuf::storage::Heap;
use ringbuf::wrap::caching::Caching;
use std::u64;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::error::TryRecvError;
use wtransport::Connection;

use crate::messages::room_message::RoomMessage;
use crate::messages::voice_message::VoiceMessage;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::{Consumer, Observer, Producer, Split};
use ringbuf::{HeapRb, SharedRb};
use tokio::time::{Duration, sleep};

const SAMPLE_RATE: u32 = 48000;
const CHANNELS: usize = 2;
const FRAME_SIZE_MS: u32 = 20;
const SAMPLES_PER_CHANNEL: usize = (SAMPLE_RATE as usize * FRAME_SIZE_MS as usize) / 1000;
const TOTAL_SAMPLES_PER_FRAME: usize = SAMPLES_PER_CHANNEL * CHANNELS;

pub struct VoiceInput {
    voice_input_control_receiver: Receiver<VoiceMessage>,
    connection: Arc<Connection>,
    encoder: OpusEncoder,
    consumer: Caching<Arc<SharedRb<Heap<f32>>>, false, true>,
    input_stream: Stream,
}

impl VoiceInput {
    pub fn new(
        voice_input_control_receiver: Receiver<VoiceMessage>,
        connection_clone: Arc<Connection>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let encoder = OpusEncoder::new(SampleRate::Hz48000, Channels::Stereo, Application::Voip)?;
        let host = cpal::default_host();
        let input_device = host.default_input_device().expect("No input device found");
        let config = cpal::StreamConfig {
            channels: CHANNELS as u16,
            sample_rate: SAMPLE_RATE,
            buffer_size: cpal::BufferSize::Default,
        };
        let ring_buffer_len = SAMPLE_RATE as usize * CHANNELS;
        let ring = HeapRb::<f32>::new(ring_buffer_len);
        let (mut producer, consumer) = ring.split();
        let input_stream = input_device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let _ = producer.push_slice(data);
            },
            move |err| eprintln!("Stream error: {}", err),
            None,
        )?;

        Ok(Self {
            voice_input_control_receiver,
            connection: connection_clone,
            encoder,
            consumer,
            input_stream,
        })
    }

    pub fn run(mut self: Self) -> Result<(), Box<dyn std::error::Error>> {
        self.input_stream.play()?;
        tokio::spawn(async move {
            let _input_stream = self.input_stream;
            let mut sequence_number: u64 = 0;
            let mut raw_samples = vec![0.0f32; TOTAL_SAMPLES_PER_FRAME];
            let mut opus_output_buffer = [0u8; 1500];
            loop {
                if self.consumer.occupied_len() >= TOTAL_SAMPLES_PER_FRAME {
                    self.consumer.pop_slice(&mut raw_samples);

                    let opus_size = match self
                        .encoder
                        .encode_float(&raw_samples, &mut opus_output_buffer)
                    {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("Encode error: {:?}", e);
                            continue;
                        }
                    };

                    let packet = RoomMessage::VoicePacket {
                        body: opus_output_buffer[0..opus_size].to_vec(),
                        order_id: sequence_number,
                        user_id: u64::MAX,
                    };

                    sequence_number = sequence_number.wrapping_add(1);

                    match bincode::serialize(&packet) {
                        Ok(serialized_data) => {
                            match self.connection.send_datagram(serialized_data) {
                                Ok(_) => {}
                                Err(err) => {
                                    println!("Error during sending voice datagram: {}", err);
                                    break;
                                }
                            }
                        }
                        Err(err) => {
                            println!("Error during serializing voice datagram: {}", err);
                            break;
                        }
                    }

                    match self.voice_input_control_receiver.try_recv() {
                        Ok(message) => match message {
                            VoiceMessage::CloseVoiceInput {} => break,
                            _ => {}
                        },
                        Err(TryRecvError::Disconnected) => {
                            println!("Voice input channel closed");
                            break;
                        }
                        _ => {}
                    }
                } else {
                    sleep(Duration::from_millis(1)).await;
                }
            }
        });

        Ok(())
    }
}
