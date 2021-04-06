use ambisonic::rodio::buffer::SamplesBuffer;
use ambisonic::rodio::{OutputStream, OutputStreamHandle, Sink};
use ambisonic::{BmixerComposer, BstreamConfig, SoundController};
use client_api::gta::matrix::{CVector, RwMatrix};
use crossbeam_channel::{Receiver, Sender};
use nalgebra::{Point3, Rotation3, Vector3};
use std::collections::{BTreeMap, HashMap};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use parking_lot::{Mutex, RwLock};

pub const MAX_DISTANCE: f32 = 50.0;
pub const REFRENCE_DISTANCE: f32 = 15.0;

#[derive(Copy, Clone)]
pub struct BrowserAudioSettings {
    pub max_distance: f32,
    pub reference_distance: f32,
}

struct Listener {
    position: Point3<f32>,
    rotation: Rotation3<f32>,
}

struct Stream {
    stream_id: i32,
    sample_rate: i32,
    channels: i32,
    max_frames: i32,
    last_pts: u64,
    last_frame_len: usize,
    pending_pcm: BTreeMap<u64, Vec<f32>>,
    sources: HashMap<i32, Source>,
}

impl Stream {
    fn new(stream_id: i32, channels: i32, sample_rate: i32, max_frames: i32) -> Stream {
        Stream {
            stream_id,
            sample_rate,
            channels,
            max_frames,
            last_pts: 0,
            last_frame_len: 0,
            sources: HashMap::new(),
            pending_pcm: BTreeMap::new(),
        }
    }
}

struct Source {
    sink: Sink,
    composer: Arc<BmixerComposer>,
    sound_controller: SoundController,
    queue: Vec<Vec<f32>>,
    sample_rate: i32,
    muted: bool,
    pcm_tx: Sender<StreamingCommand>,
}

impl Source {
    fn new(handle: &OutputStreamHandle, sample_rate: i32) -> Source {
        let sink = match Sink::try_new(handle) {
            Ok(sink) => sink,
            Err(err) => {
                log::error!("cannot create a new sink: {:?}", err);
                std::process::exit(0);
            }
        };

        let (mixer, composer) = ambisonic::bmixer(sample_rate as _);
        let cfg = ambisonic::StereoConfig::default();
        let output = ambisonic::BstreamStereoRenderer::new(mixer, cfg);

        sink.append(output);

        let (src, pcm_tx) = StreamingSound::new(sample_rate as u32);
        let sound_controller = composer.play(src, BstreamConfig::new());

        sink.set_volume(0.0);
        sink.play();

        Source {
            sink,
            composer,
            sound_controller,
            sample_rate,
            pcm_tx,
            queue: Vec::new(),
            muted: true,
        }
    }

    fn queue(&mut self, _sample_rate: i32, pcm: &[f32]) {
        let _ = self.pcm_tx.send(StreamingCommand::Pcm(Vec::from(pcm)));
    }

    fn reset(&mut self) {
        let _ = self.pcm_tx.send(StreamingCommand::Reset);
    }

    fn set_position(&mut self, position: Point3<f32>) {
        self.sound_controller
            .adjust_position([position.x, position.y, position.z]);
    }

    fn set_velocity(&mut self, velocity: Point3<f32>) {
        self.sound_controller
            .set_velocity([velocity.x, velocity.y, velocity.z]);
    }
}

enum Command {
    Stream {
        browser: u32,
        stream_id: i32,
        channels: i32,
        sample_rate: i32,
        max_frames: i32,
    },

    Source {
        browser: u32,
        object_id: i32,
    },

    Pcm {
        browser: u32,
        stream_id: i32,
        data: Vec<f32>,
        frames: usize,
        pts: u64,
    },

    RemoveStream {
        browser: u32,
        stream_id: i32,
    },

    RemoveAllStreams {
        browser: u32,
    },

    RemoveSource {
        browser: u32,
        object_id: i32,
    },

    ObjectSettings {
        object_id: i32,
        position: Point3<f32>,
        velocity: Point3<f32>,
    },

    Gain(f32),
    MuteObject(i32),
    TogglePause(bool),
    Terminate,
}

pub struct Audio {
    command_tx: Sender<Command>,
    stream_channels: RwLock<HashMap<(u32, i32), i32>>,
    listener: Mutex<Listener>,
}

impl Audio {
    pub fn new() -> Arc<Audio> {
        let (tx, rx) = crossbeam_channel::unbounded();

        // TODO: crash app
        std::thread::spawn(move || {
            if let Some(audio) = AudioInner::new(rx) {
                log::trace!("audio_thread start");
                audio_thread(audio);
            }
        });

        let listener = Listener {
            position: Point3::new(0.0, 0.0, 0.0),
            rotation: Rotation3::identity(),
        };

        Arc::new(Audio {
            command_tx: tx,
            stream_channels: RwLock::new(HashMap::new()),
            listener: Mutex::new(listener),
        })
    }

    pub fn create_stream(
        &self, browser: u32, stream_id: i32, channels: i32, sample_rate: i32, max_frames: i32,
    ) {
        self.stream_channels
            .write()
            .insert((browser, stream_id), channels);

        let _ = self.command_tx.send(Command::Stream {
            browser,
            stream_id,
            channels,
            sample_rate,
            max_frames,
        });
    }

    pub fn append_pcm(
        &self, browser: u32, stream_id: i32, data: *mut *const f32, frames: i32, pts: u64,
    ) {
        if frames == 0 || data.is_null() {
            return;
        }

        let current_time = crate::utils::current_time();

        // пакеты могут отставать на ~20-30мс, что НЕ критично
        // дропаем все, что хотя бы на 1с отстают (паузы / лаги еще какая херня)
        if current_time - pts as i128 >= 1000 {
            return;
        }

        let channels = self
            .stream_channels
            .read()
            .get(&(browser, stream_id))
            .unwrap_or(&1)
            .clone();

        let frames = frames as usize;
        let factor = 1.0 / channels as f32;

        let mut pending = vec![0.0; frames];

        unsafe {
            std::slice::from_raw_parts(data, channels as usize)
                .iter()
                .map(|&ptr| std::slice::from_raw_parts(ptr, frames))
                .for_each(|inner| {
                    inner
                        .iter()
                        .enumerate()
                        .for_each(|(i, x)| pending[i] += x * factor)
                });
        }

        let _ = self.command_tx.send(Command::Pcm {
            data: pending,
            browser,
            stream_id,
            frames,
            pts,
        });
    }

    pub fn remove_stream(&self, browser: u32, stream_id: i32) {
        let _ = self
            .command_tx
            .send(Command::RemoveStream { browser, stream_id });
    }

    pub fn remove_all_streams(&self, browser: u32) {
        let _ = self.command_tx.send(Command::RemoveAllStreams { browser });
    }

    pub fn add_source(&self, browser: u32, object_id: i32) {
        let _ = self.command_tx.send(Command::Source { browser, object_id });
    }

    pub fn remove_source(&self, browser: u32, object_id: i32) {
        let _ = self
            .command_tx
            .send(Command::RemoveSource { browser, object_id });
    }

    pub fn set_gain(&self, gain: f32) {
        let _ = self.command_tx.send(Command::Gain(gain));
    }

    // TODO: ?
    pub fn set_velocity(&self, _velocity: CVector) {}

    pub fn set_position(&self, position: CVector) {
        let point = Point3::new(position.x, position.y, position.z);
        self.listener.lock().position = point;
    }

    pub fn set_orientation(&self, matrix: RwMatrix) {
        let at = Vector3::new(-matrix.at.x, -matrix.at.y, -matrix.at.z);
        let up = Vector3::new(matrix.up.x, matrix.up.y, matrix.up.z);

        let rotation = Rotation3::face_towards(&at, &up);

        self.listener.lock().rotation = rotation;
    }

    pub fn set_object_settings(
        &self, object_id: i32, pos: CVector, velo: CVector, _direction: CVector,
        settings: BrowserAudioSettings,
    ) {
        let position = {
            let listener = self.listener.lock();
            let diff = Point3::new(
                pos.x - listener.position.x,
                pos.y - listener.position.y,
                pos.z - listener.position.z,
            );

            listener.rotation.transform_point(&diff)
        };

        let dist = (position.clone() - Point3::origin()).magnitude();

        // let position = if dist <= settings.reference_distance {
        //     position.map(|i| i / dist)
        // } else {
        //     position.map(|i| i * (settings.max_distance - settings.reference_distance) * 0.01)
        // };

        let make = |x1: f32, y1: f32, x2: f32, y2: f32| {
            let a = y1 - y2;
            let b = x2 - x1;
            let c = x1 * y2 - x2 * y1;

            move |x: f32| -> f32 { (-a * x - c) / b }
        };

        let position = if dist <= settings.reference_distance {
            position.map(|i| i / dist * 0.01) // полная громкость, источник почти в ухе
        } else {
            let f = make(
                0.0,
                0.001,
                settings.max_distance - settings.reference_distance,
                100.0,
            );
            let k = f(dist - settings.reference_distance);
            position.map(|i| i / dist * k)
        };

        let _ = self.command_tx.send(Command::ObjectSettings {
            object_id,
            position,
            velocity: Point3::new(velo.x, velo.y, velo.z),
        });
    }

    pub fn object_mute(&self, object_id: i32) {
        let _ = self.command_tx.send(Command::MuteObject(object_id));
    }

    pub fn set_paused(&self, paused: bool) {
        let _ = self.command_tx.send(Command::TogglePause(paused));
    }

    pub fn terminate(&self) {
        let _ = self.command_tx.send(Command::Terminate);
    }
}

struct AudioInner {
    command_rx: Receiver<Command>,
    output_stream: OutputStream,
    stream_handle: OutputStreamHandle,
    paused: bool,
    gain: f32,
    streams: HashMap<u32, Vec<Stream>>,
}

impl AudioInner {
    fn new(command_rx: Receiver<Command>) -> Option<AudioInner> {
        let (output_stream, stream_handle) = match OutputStream::try_default() {
            Ok(val) => val,
            Err(err) => {
                log::error!("rodeo no default output: {:?}", err);
                // std::process::exit(0);
                return None;
            }
        };

        Some(AudioInner {
            output_stream,
            stream_handle,
            command_rx,
            paused: false,
            gain: 1.0,
            streams: HashMap::new(),
        })
    }

    fn create_stream(
        &mut self, browser: u32, stream_id: i32, channels: i32, sample_rate: i32, max_frames: i32,
    ) {
        let stream = Stream::new(stream_id, channels, sample_rate, max_frames);

        let entries = self
            .streams
            .entry(browser)
            .or_insert_with(|| Vec::with_capacity(1));

        entries.push(stream);
    }

    fn append_pcm(
        &mut self, browser: u32, stream_id: i32, data: Vec<f32>, _frames: usize, pts: u64,
    ) {
        if self.paused {
            return;
        }

        if let Some(entries) = self.streams.get_mut(&browser) {
            if let Some(stream) = entries
                .iter_mut()
                .find(|stream| stream.stream_id == stream_id)
            {
                stream.pending_pcm.insert(pts, data);
            }
        }
    }

    fn remove_stream(&mut self, browser: u32, stream_id: i32) {
        let mut remove = false;

        if let Some(entries) = self.streams.get_mut(&browser) {
            if let Some(idx) = entries
                .iter()
                .position(|entry| entry.stream_id == stream_id)
            {
                entries.remove(idx);
            }

            remove = entries.len() == 0;
        }

        if remove {
            self.streams.remove(&browser);
        }
    }

    fn remove_all_streams(&mut self, browser: u32) {
        self.streams.remove(&browser);
    }

    fn add_source(&mut self, browser: u32, object_id: i32) {
        let Self {
            ref mut streams,
            ref stream_handle,
            ..
        } = self;

        if let Some(entries) = streams.get_mut(&browser) {
            entries.iter_mut().for_each(|entry| {
                if entry.sources.contains_key(&object_id) {
                    return;
                }

                let source = Source::new(stream_handle, entry.sample_rate);
                entry.sources.insert(object_id, source);
            });
        }
    }

    fn remove_source(&mut self, browser: u32, object_id: i32) {
        if let Some(entries) = self.streams.get_mut(&browser) {
            entries.iter_mut().for_each(|entry| {
                let _ = entry.sources.remove(&object_id);
            });
        }
    }

    pub fn set_gain(&mut self, gain: f32) {
        if self.gain != gain {
            self.gain = gain;

            if self.paused {
                return;
            }

            self.set_sinks_gain(gain);
        }
    }

    pub fn set_object_settings(&mut self, object_id: i32, pos: Point3<f32>, velo: Point3<f32>) {
        let gain = self.gain;

        self.for_object(object_id, |source| {
            source.set_position(pos.clone());
            source.set_velocity(velo.clone());

            if source.muted {
                source.sink.set_volume(gain);
                source.muted = false;
            }
        });
    }

    pub fn object_mute(&mut self, object_id: i32) {
        self.for_object(object_id, |source| {
            if !source.muted {
                source.sink.set_volume(0.0);
                source.muted = true;
            }
        });
    }

    pub fn set_paused(&mut self, paused: bool) {
        if self.paused == paused {
            return;
        }

        self.paused = paused;

        let gain = if paused { 0.0 } else { self.gain };

        self.set_sinks_gain(gain);
    }

    fn set_sinks_gain(&mut self, gain: f32) {
        self.streams.values_mut().for_each(|stream| {
            stream.iter_mut().for_each(|stream| {
                stream.sources.values().for_each(|source| {
                    if !source.muted {
                        source.sink.set_volume(gain);
                    }
                });
            });
        });
    }

    fn reset_sinks(&mut self) {
        self.streams.values_mut().for_each(|stream| {
            stream.iter_mut().for_each(|stream| {
                stream
                    .sources
                    .values_mut()
                    .for_each(|source| source.reset());
            });
        });
    }

    fn for_object<F>(&mut self, object_id: i32, mut func: F)
    where
        F: FnMut(&mut Source),
    {
        self.streams.values_mut().for_each(|stream| {
            stream.iter_mut().for_each(|stream| {
                stream.sources.get_mut(&object_id).map(|source| {
                    func(source);
                });
            })
        });
    }
}

fn audio_thread(mut audio: AudioInner) {
    loop {
        while let Ok(command) = audio.command_rx.try_recv() {
            match command {
                Command::Source { browser, object_id } => {
                    audio.add_source(browser, object_id);
                }

                Command::Stream {
                    browser,
                    stream_id,
                    sample_rate,
                    channels,
                    max_frames,
                } => {
                    audio.create_stream(browser, stream_id, channels, sample_rate, max_frames);
                }

                Command::Pcm {
                    browser,
                    stream_id,
                    data,
                    frames,
                    pts,
                } => {
                    audio.append_pcm(browser, stream_id, data, frames, pts);
                }

                Command::RemoveStream { browser, stream_id } => {
                    audio.remove_stream(browser, stream_id);
                }

                Command::RemoveAllStreams { browser } => {
                    audio.remove_all_streams(browser);
                }

                Command::RemoveSource { browser, object_id } => {
                    audio.remove_source(browser, object_id);
                }

                Command::ObjectSettings {
                    object_id,
                    position,
                    velocity,
                } => {
                    audio.set_object_settings(object_id, position, velocity);
                }

                Command::Gain(volume) => {
                    audio.set_gain(volume);
                }

                Command::MuteObject(object_id) => {
                    audio.object_mute(object_id);
                }

                Command::TogglePause(paused) => {
                    audio.set_paused(paused);
                }

                Command::Terminate => return,
            }
        }

        let mut browsers = &mut audio.streams;

        if !audio.paused {
            for streams in browsers.values_mut() {
                for stream in streams {
                    for (pts, pending) in stream.pending_pcm.iter() {
                        let sample_rate = stream.sample_rate;

                        stream
                            .sources
                            .values_mut()
                            .for_each(|source| source.queue(sample_rate, pending));

                        stream.last_pts = *pts;
                        stream.last_frame_len = pending.len();
                    }

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

#[derive(Debug, Clone)]
enum StreamingCommand {
    Pcm(Vec<f32>),
    Reset,
}

struct StreamingSound {
    sample_rate: u32,
    pcm_rx: Receiver<StreamingCommand>,
    queue: std::vec::IntoIter<f32>,
    pending: Vec<Vec<f32>>,
    playing: bool,
}

impl StreamingSound {
    fn new(sample_rate: u32) -> (StreamingSound, Sender<StreamingCommand>) {
        let (tx, rx) = crossbeam_channel::unbounded();
        let sound = StreamingSound {
            sample_rate,
            pcm_rx: rx,
            queue: Vec::new().into_iter(),
            pending: Vec::with_capacity(8),
            playing: false,
        };

        (sound, tx)
    }

    fn reset(&mut self) {
        self.queue = Vec::new().into_iter();
        self.pending = Vec::with_capacity(8);
        self.playing = false;
    }
}

impl Iterator for StreamingSound {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        while let Ok(cmd) = self.pcm_rx.try_recv() {
            match cmd {
                StreamingCommand::Pcm(pending) => {
                    self.pending.push(pending);
                }
                StreamingCommand::Reset => self.reset(),
            }
        }

        loop {
            match self.queue.next() {
                Some(sample) => return Some(sample),
                None => {
                    let push = if self.playing && self.pending.len() >= 1 {
                        true
                    } else if !self.playing && self.pending.len() >= 8 {
                        true
                    } else {
                        self.playing = false;
                        false
                    };

                    if push {
                        let next = std::mem::replace(&mut self.pending, Vec::with_capacity(8));
                        self.queue = next.into_iter().flatten().collect::<Vec<f32>>().into_iter();
                        self.playing = true;
                        continue;
                    }

                    return Some(0.0);
                }
            }
        }
    }
}

impl ambisonic::rodio::Source for StreamingSound {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
