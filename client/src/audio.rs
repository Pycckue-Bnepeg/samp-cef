use alto::{Alto, Buffer, Context, DistanceModel, Mono, Source, SourceState, StreamingSource};
use client_api::gta::matrix::{CVector, RwMatrix};

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};

pub const MAX_DISTANCE: f32 = 50.0;
pub const REFRENCE_DISTANCE: f32 = 15.0;

#[derive(Copy, Clone)]
pub struct BrowserAudioSettings {
    pub max_distance: f32,
    pub reference_distance: f32,
}

pub struct Audio {
    alto: Alto,
    context: Context,
    paused: AtomicBool,
    terminate: AtomicBool,
    streams: Mutex<HashMap<u32, Vec<AudioStream>>>, // browser_id -> streams
}

pub struct AudioStream {
    stream_id: i32,
    sample_rate: i32,
    channels: i32,
    max_frames: i32,
    last_pts: u64,
    last_frame_len: usize,
    pending_pcm: BTreeMap<u64, Vec<f32>>,
    sources: HashMap<i32, AudioSource>, // object_id -> source
}

impl AudioStream {
    pub fn play(&mut self) {
        self.sources.values_mut().for_each(|source| source.play());
    }

    pub fn clear_queue(&mut self) {
        self.pending_pcm = BTreeMap::new();
    }

    pub fn reset(&mut self) {
        self.clear_queue();

        self.sources.values_mut().for_each(|source| {
            while source.source.buffers_queued() != 0 && source.source.buffers_processed() != 0 {
                if let Ok(buffer) = source.source.unqueue_buffer() {
                    source.buffers.push(buffer);
                }
            }
        })
    }

    pub fn unqueue_buffers(&mut self) {
        self.sources
            .values_mut()
            .for_each(|source| source.unqueue_buffers());
    }
}

pub struct AudioSource {
    buffers: Vec<Buffer>,
    source: StreamingSource,
    muted: bool,
}

impl AudioSource {
    pub fn queue(&mut self, sample_rate: i32, pcm: &[f32]) -> bool {
        if let Some(mut buffer) = self.free_buffer() {
            buffer.set_data::<Mono<f32>, _>(&pcm, sample_rate);
            self.source.queue_buffer(buffer);
            true
        } else {
            false
        }
    }

    pub fn play(&mut self) {
        match self.source.state() {
            SourceState::Initial | SourceState::Stopped => {
                self.source.play();
            }

            _ => (),
        }
    }

    pub fn stop(&mut self) {
        self.source.stop();
    }

    pub fn unqueue_buffers(&mut self) {
        while self.source.buffers_processed() != 0 {
            if let Ok(buffer) = self.source.unqueue_buffer() {
                self.buffers.push(buffer);
            }
        }
    }

    fn free_buffer(&mut self) -> Option<Buffer> {
        if let Some(buffer) = self.buffers.pop() {
            Some(buffer)
        } else if self.source.buffers_processed() > 0 {
            self.source.unqueue_buffer().ok()
        } else {
            None
        }
    }
}

impl Audio {
    pub fn new() -> Arc<Audio> {
        let path = crate::utils::cef_dir().join("sound.dll");

        log::trace!("audio path: {:?}", path);

        let alto = match Alto::load(path) {
            Ok(alto) => alto,
            Err(err) => {
                log::trace!("Alto::load error: {:?}", err);

                client_api::utils::error_message_box("CEF error", "There is no OpenAL library (sound.dll) in the CEF folder.\nPlease reinstall the plugin and try again.");

                std::thread::sleep(std::time::Duration::from_secs(10));
                std::process::exit(0);
            }
        };

        log::trace!("openaal loaded opening device and creating context");

        let context = match alto.open(None).and_then(|device| device.new_context(None)) {
            Ok(ctx) => ctx,
            Err(err) => {
                log::trace!("openal new_context error: {:?}", err);

                client_api::utils::error_message_box(
                    "CEF error",
                    "There is no default output device.",
                );

                std::thread::sleep(std::time::Duration::from_secs(10));
                std::process::exit(0);
            }
        };

        context.use_source_distance_model(true);
        context.set_gain(1.0);

        let audio = Audio {
            alto,
            context,
            paused: AtomicBool::new(false),
            terminate: AtomicBool::new(false),
            streams: Mutex::new(HashMap::new()),
        };

        let audio = Arc::new(audio);
        let another_audio = audio.clone();

        log::trace!("spawning audio thread");

        std::thread::spawn(move || audio_thread(another_audio));

        audio
    }

    pub fn create_stream(
        &self, browser: u32, stream_id: i32, channels: i32, sample_rate: i32, max_frames: i32,
    ) {
        let mut streams = self.streams.lock().unwrap();
        let entries = streams
            .entry(browser)
            .or_insert_with(|| Vec::with_capacity(1));

        let audio_stream = AudioStream {
            stream_id,
            sample_rate,
            channels,
            max_frames,
            last_pts: 0,
            last_frame_len: 0,
            pending_pcm: BTreeMap::new(),
            sources: HashMap::new(),
        };

        entries.push(audio_stream);
    }

    pub fn append_pcm(
        &self, browser: u32, stream_id: i32, data: *mut *const f32, frames: i32, pts: u64,
    ) {
        if self.paused.load(Ordering::SeqCst) {
            return;
        }

        if frames == 0 || data.is_null() {
            return;
        }

        let current_time = crate::utils::current_time();

        if current_time - pts as i128 > 0 {
            return;
        }

        let mut streams = self.streams.lock().unwrap();

        if let Some(entries) = streams.get_mut(&browser) {
            if let Some(stream) = entries
                .iter_mut()
                .find(|stream| stream.stream_id == stream_id)
            {
                let frames = frames as usize;
                let factor = 1.0 / stream.channels as f32;

                let mut pending = vec![0.0; frames];

                unsafe {
                    std::slice::from_raw_parts(data, stream.channels as usize)
                        .iter()
                        .map(|&ptr| std::slice::from_raw_parts(ptr, frames))
                        .for_each(|inner| {
                            inner
                                .iter()
                                .enumerate()
                                .for_each(|(i, x)| pending[i] += x * factor)
                        });
                }

                stream.pending_pcm.insert(pts, pending);
            }
        }
    }

    pub fn remove_stream(&self, browser: u32, stream_id: i32) {
        let mut remove = false;
        let mut streams = self.streams.lock().unwrap();

        if let Some(entries) = streams.get_mut(&browser) {
            if let Some(idx) = entries
                .iter()
                .position(|entry| entry.stream_id == stream_id)
            {
                entries.remove(idx);
            }

            remove = entries.len() == 0;
        }

        if remove {
            streams.remove(&browser);
        }
    }

    pub fn remove_all_streams(&self, browser: u32) {
        let mut streams = self.streams.lock().unwrap();
        streams.remove(&browser);
    }

    pub fn add_source(&self, browser: u32, object_id: i32) {
        let mut streams = self.streams.lock().unwrap();

        if let Some(entries) = streams.get_mut(&browser) {
            entries.iter_mut().for_each(|entry| {
                if entry.sources.contains_key(&object_id) {
                    return;
                }

                let mut source = self.context.new_streaming_source().unwrap();
                let plain_data = vec![0.0f32; entry.max_frames as usize];
                let mut buffers = Vec::with_capacity(50);

                for _ in 0..50 {
                    let buffer = self
                        .context
                        .new_buffer::<Mono<f32>, _>(plain_data.as_slice(), entry.sample_rate)
                        .unwrap();

                    buffers.push(buffer);
                }

                source.set_distance_model(DistanceModel::ExponentClamped);
                source.set_max_distance(MAX_DISTANCE);
                source.set_reference_distance(REFRENCE_DISTANCE);
                source.set_rolloff_factor(7.0);
                source.set_relative(false);
                source.set_max_gain(0.0); //

                let audio_source = AudioSource {
                    source,
                    buffers,
                    muted: true,
                };

                entry.sources.insert(object_id, audio_source);
            });
        }
    }

    pub fn remove_source(&self, browser: u32, object_id: i32) {
        let mut streams = self.streams.lock().unwrap();

        if let Some(entries) = streams.get_mut(&browser) {
            entries.iter_mut().for_each(|entry| {
                let _ = entry.sources.remove(&object_id);
            });
        }
    }

    pub fn set_gain(&self, gain: f32) {
        if !self.paused.load(Ordering::SeqCst) {
            self.context.set_gain(gain);
        }
    }

    pub fn set_velocity(&self, velocity: CVector) {
        self.context
            .set_velocity([velocity.x, velocity.y, velocity.z]);
    }

    pub fn set_position(&self, position: CVector) {
        self.context
            .set_position([position.x, position.y, position.z]);
    }

    pub fn set_orientation(&self, matrix: RwMatrix) {
        self.context.set_orientation((
            [-matrix.at.x, -matrix.at.y, -matrix.at.z],
            [matrix.up.x, matrix.up.y, matrix.up.z],
        ));
    }

    pub fn set_object_settings(
        &self, object_id: i32, pos: CVector, velo: CVector, direction: CVector,
        settings: BrowserAudioSettings,
    ) {
        self.for_object(object_id, |source| {
            source.source.set_position([pos.x, pos.y, pos.z]);
            source.source.set_velocity([velo.x, velo.y, velo.z]);

            source
                .source
                .set_direction([direction.x, direction.y, direction.z]);

            source.source.set_max_distance(settings.max_distance);
            source
                .source
                .set_reference_distance(settings.reference_distance);

            if source.muted {
                source.source.set_max_gain(1.0);
                source.muted = false;
            }
        });
    }

    pub fn object_mute(&self, object_id: i32) {
        self.for_object(object_id, |source| {
            if !source.muted {
                source.source.set_max_gain(0.0);
                source.muted = true;
            }
        });
    }

    pub fn set_paused(&self, paused: bool) {
        let prev = self.paused.swap(paused, Ordering::SeqCst);

        if prev != paused && paused {
            self.context.set_gain(0.0);
            let mut streams = self.streams.lock().unwrap();

            streams.values_mut().for_each(|stream| {
                stream.iter_mut().for_each(|stream| {
                    stream.reset();
                })
            });
        }
    }

    pub fn terminate(&self) {
        self.terminate.store(true, Ordering::SeqCst);
    }

    fn for_object<F>(&self, object_id: i32, mut func: F)
    where
        F: FnMut(&mut AudioSource),
    {
        let mut streams = self.streams.lock().unwrap();

        streams.values_mut().for_each(|stream| {
            stream.iter_mut().for_each(|stream| {
                stream.sources.get_mut(&object_id).map(|source| {
                    func(source);
                });
            })
        });
    }
}

fn audio_thread(audio: Arc<Audio>) {
    loop {
        if audio.terminate.load(Ordering::SeqCst) {
            break;
        }

        if !audio.paused.load(Ordering::SeqCst) {
            let mut browsers = audio.streams.lock().unwrap();

            for streams in browsers.values_mut() {
                for stream in streams {
                    stream.unqueue_buffers();

                    let current_time = crate::utils::current_time();

                    for (pts, pending) in stream.pending_pcm.iter() {
                        let pts_big = *pts as i128;
                        let delta = (pts_big - current_time).abs();
                        let next_tick = stream.last_pts
                            + ((stream.last_frame_len as f64 / stream.sample_rate as f64) * 1000.0)
                                as u64;

                        // 2 ms
                        let queue = if delta >= 0 && delta <= 2 {
                            // should be immediatly played
                            true
                        } else if *pts == next_tick {
                            // should be next
                            true
                        } else {
                            // should wait
                            break;
                        };

                        if queue {
                            let sample_rate = stream.sample_rate;

                            let success = stream
                                .sources
                                .values_mut()
                                .fold(true, |acc, source| acc & source.queue(sample_rate, pending));

                            if success {
                                stream.last_pts = *pts;
                                stream.last_frame_len = pending.len();
                                continue;
                            } else {
                                break;
                            }
                        }
                    }

                    stream.play();

                    let keys: Vec<u64> = stream
                        .pending_pcm
                        .range(0..=stream.last_pts)
                        .map(|(pts, _)| *pts)
                        .collect();

                    for key in keys {
                        stream.pending_pcm.remove(&key);
                    }
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_micros(500));
    }
}
