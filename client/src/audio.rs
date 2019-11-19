use alto::{Alto, Buffer, Context, DistanceModel, Mono, Source, SourceState, StreamingSource};
use client_api::gta::matrix::{CVector, RwMatrix};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const MAX_DISTANCE: f32 = 30.0;

pub struct Audio {
    alto: Alto,
    context: Context,
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

    pub fn unqueue_buffers(&mut self) {
        self.sources
            .values_mut()
            .for_each(|source| source.unqueue_buffers());
    }
}

pub struct AudioSource {
    buffers: Vec<Buffer>,
    source: StreamingSource,
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
        if
        /*self.source.buffers_queued() >= 1 &&*/
        self.source.state() != SourceState::Playing {
            self.source.play();
        }
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
        let alto = match Alto::load("openal.dll") {
            Ok(alto) => alto,
            Err(_) => {
                client_api::utils::error_message_box("CEF error", "There is no OpenAL library in the root folder of GTA.\nPlease reinstall the plugin and try again.");
                std::process::exit(0);
            }
        };

        let context = match alto.open(None).and_then(|device| device.new_context(None)) {
            Ok(ctx) => ctx,
            Err(_) => {
                client_api::utils::error_message_box(
                    "CEF error",
                    "There is no default output device.",
                );

                std::process::exit(0);
            }
        };

        context.use_source_distance_model(true);
        context.set_gain(1.0);

        let audio = Audio {
            alto,
            context,
            streams: Mutex::new(HashMap::new()),
        };

        let audio = Arc::new(audio);
        let another_audio = audio.clone();

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
        if frames == 0 || data.is_null() {
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

                source.set_distance_model(DistanceModel::InverseClamped);
                source.set_max_distance(MAX_DISTANCE);
                source.set_position([0.0, 0.0, 0.0]);
                source.set_reference_distance(1.0);
                source.set_air_absorption_factor(10.0);
                source.set_cone_inner_angle(120.0);
                source.set_cone_outer_angle(180.0);
                source.set_relative(true);
                source.set_gain(1.0);
                //                source.set_relative(false);

                let audio_source = AudioSource { source, buffers };

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
        self.context.set_gain(gain);
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
}

fn audio_thread(audio: Arc<Audio>) {
    loop {
        {
            let mut browsers = audio.streams.lock().unwrap();

            for streams in browsers.values_mut() {
                for stream in streams {
                    stream.unqueue_buffers();

                    let current_time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_else(|_| Duration::from_secs(0))
                        .as_millis() as i128;

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
