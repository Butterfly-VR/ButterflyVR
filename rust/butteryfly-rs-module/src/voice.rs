use godot::classes::AudioStream;
use godot::classes::AudioStreamPlayback;
use godot::classes::IAudioStream;
use godot::classes::IAudioStreamPlayback;
use godot::classes::native::AudioFrame;
use godot::prelude::*;
use opus::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::VecDeque;

pub const SAMPLE_RATE: u32 = 48000;
pub const FRAME_LENGTH: usize = 960;
const BITRATE: i32 = 96000;

#[derive(Default)]
pub struct VoiceStreamManager {
    encoders: HashMap<usize, Encoder>,
    decoders: HashMap<usize, Decoder>,
    next_stream_num: usize,
}

impl VoiceStreamManager {
    pub fn create_encoder(&mut self) -> usize {
        let mut encoder = Encoder::new(SAMPLE_RATE, Channels::Mono, Application::Voip).unwrap();
        encoder.set_bitrate(Bitrate::Bits(BITRATE)).unwrap();
        encoder.set_inband_fec(false).unwrap();
        encoder.set_packet_loss_perc(5).unwrap();

        self.encoders.insert(self.next_stream_num, encoder);
        self.next_stream_num += 1;
        return self.next_stream_num - 1;
    }
    pub fn create_stereo_encoder(&mut self) -> usize {
        let mut encoder = Encoder::new(SAMPLE_RATE, Channels::Stereo, Application::Voip).unwrap();
        encoder.set_bitrate(Bitrate::Bits(BITRATE)).unwrap();
        encoder.set_inband_fec(true).unwrap();
        encoder.set_packet_loss_perc(5).unwrap();

        self.encoders.insert(self.next_stream_num, encoder);
        self.next_stream_num += 1;
        return self.next_stream_num - 1;
    }
    pub fn create_decoder(&mut self) -> usize {
        self.decoders.insert(
            self.next_stream_num,
            Decoder::new(SAMPLE_RATE, Channels::Mono).unwrap(),
        );
        self.next_stream_num += 1;
        return self.next_stream_num - 1;
    }
    pub fn create_stereo_decoder(&mut self) -> usize {
        self.decoders.insert(
            self.next_stream_num,
            Decoder::new(SAMPLE_RATE, Channels::Stereo).unwrap(),
        );
        self.next_stream_num += 1;
        return self.next_stream_num - 1;
    }
    pub fn encode_audio(&mut self, stream: usize, samples: &[f32]) -> Vec<u8> {
        if let Some(encoder) = self.encoders.get_mut(&stream) {
            let mut output = vec![0; 600];
            let length = encoder.encode_float(samples, &mut output).unwrap();
            output.truncate(length);
            output
        } else {
            Vec::new()
        }
    }
    pub fn decode_audio(&mut self, stream: usize, samples: &[u8]) -> Vec<f32> {
        if let Some(decoder) = self.decoders.get_mut(&stream) {
            let mut output = vec![0.0; FRAME_LENGTH];
            decoder.decode_float(samples, &mut output, false).unwrap();
            output
        } else {
            Vec::new()
        }
    }
    pub fn decode_stereo_audio(&mut self, stream: usize, samples: &[u8]) -> Vec<f32> {
        if let Some(decoder) = self.decoders.get_mut(&stream) {
            let mut output = vec![0.0; FRAME_LENGTH * 2];
            decoder.decode_float(samples, &mut output, false).unwrap();
            output
        } else {
            Vec::new()
        }
    }
}
#[derive(GodotClass)]
#[class(init, base=AudioStream)]
struct VoiceStream {
    playback: RefCell<Option<Gd<VoiceStreamPlayback>>>,
    base: Base<AudioStream>,
}
#[godot_api]
impl IAudioStream for VoiceStream {
    fn instantiate_playback(&self) -> Option<Gd<AudioStreamPlayback>> {
        self.playback.replace(Some(VoiceStreamPlayback::new_gd()));
        Some(self.playback.clone().take().unwrap().upcast())
    }
    fn get_length(&self) -> f64 {
        0.0
    }
    fn is_monophonic(&self) -> bool {
        false
    }
}
#[godot_api]
impl VoiceStream {
    #[func]
    fn get_current_playback(&self) -> Option<Gd<VoiceStreamPlayback>> {
        self.playback.clone().take()
    }
}

#[derive(GodotClass)]
#[class(init, base=AudioStreamPlayback)]
struct VoiceStreamPlayback {
    is_playing: bool,
    audio: VecDeque<f32>,
    base: Base<AudioStreamPlayback>,
}
#[godot_api]
impl IAudioStreamPlayback for VoiceStreamPlayback {
    fn start(&mut self, _from_pos: f64) {
        self.is_playing = true;
    }
    fn stop(&mut self) {
        self.is_playing = false;
    }
    fn is_playing(&self) -> bool {
        self.is_playing
    }
    fn get_loop_count(&self) -> i32 {
        0
    }
    fn get_playback_position(&self) -> f64 {
        0.0
    }
    fn seek(&mut self, _position: f64) {
        return;
    }
    unsafe fn mix(&mut self, buffer: *mut AudioFrame, _rate_scale: f32, frames: i32) -> i32 {
        if !self.is_playing {
            return frames;
        }
        let buffer = unsafe { std::slice::from_raw_parts_mut(buffer, frames as usize) };
        for frame in buffer {
            frame.left = self.audio.pop_front().unwrap_or(0.0);
            frame.right = self.audio.pop_front().unwrap_or(0.0);
        }
        frames
    }
}
#[godot_api]
impl VoiceStreamPlayback {
    #[func]
    fn buffer_audio(&mut self, buffer: Array<f32>) {
        self.audio.extend(buffer.iter_shared());
        if self.audio.len() > 10000 {
            self.audio.drain(0..self.audio.len() - 10000);
        }
    }
}
