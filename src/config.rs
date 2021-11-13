use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};

use id3::Tag;
use rodio::{OutputStream, OutputStreamHandle, Sink};

use crate::{
    audio::ChannelAudio, commands::Command, conffile::Conffile, files::ensure_file_existence,
    l10n::L10n, songs::Repeat, Error,
};

pub struct Config
{
    pub sink: Sink,
    pub stream_handle: OutputStreamHandle,
    pub _stream: OutputStream,
    pub source: ChannelAudio,
    pub tx: Sender<Command>,
    pub rx: Receiver<Command>,
    pub rx_control: Receiver<()>,
    pub tx_paused: Sender<bool>,
    pub tx_path: Sender<(PathBuf, Option<Tag>)>,
    pub tag: Option<Result<Tag, id3::Error>>,
    pub num: u32,
    pub loud: f32,
    pub paused: bool,
    pub pause_after_song: bool,
    pub quit_after_song: bool,
    pub repeat: Repeat,
    pub songlist: Vec<usize>,
    pub song_index: usize,
    pub arc_config: Arc<ArcConfig>,
    pub l10n: L10n,
    pub unsuccessful_tries: u8,
}

// False positive of pedantic lint.  I think this is the best name.
#[allow(clippy::module_name_repetitions)]
pub struct ArcConfig
{
    pub pic_path: Mutex<Option<String>>,
    pub reading_paused: AtomicBool,
    pub update_dbus: AtomicBool,
    pub current_pos: AtomicUsize,
    pub current_len: AtomicUsize,
    pub sample_rate: AtomicUsize,
    pub channels: AtomicUsize,
    pub home_dir: PathBuf,
    pub config_dir: PathBuf,
    pub conffile: Conffile,
    pub l10n: L10n,
}

impl ArcConfig
{
    fn new() -> Result<Self, Error>
    {
        let home_dir = home::home_dir().unwrap_or_else(|| PathBuf::from("./"));
        ensure_file_existence(&home_dir)?;
        let conffile_dir = home_dir.join(PathBuf::from("./.zvavybir/legacylisten"));
        let conffile =
            Conffile::new(&conffile_dir).unwrap_or_else(|_| Conffile::default(&conffile_dir));
        let l10n = L10n::new(conffile.lang.clone())?;

        Ok(Self {
            pic_path: Mutex::new(None),
            reading_paused: AtomicBool::new(false),
            update_dbus: AtomicBool::new(false),
            current_pos: AtomicUsize::new(0),
            current_len: AtomicUsize::new(0),
            sample_rate: AtomicUsize::new(1),
            channels: AtomicUsize::new(1),
            home_dir,
            config_dir: conffile_dir,
            conffile,
            l10n,
        })
    }
}

impl Config
{
    pub fn new(
        rx_control: Receiver<()>,
        tx_paused: Sender<bool>,
        tx_path: Sender<(PathBuf, Option<Tag>)>,
    ) -> Result<Self, Error>
    {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        let (tx, rx) = channel();
        let arc_config = Arc::new(ArcConfig::new()?);
        let l10n = arc_config.l10n;

        Ok(Self {
            sink,
            stream_handle,
            _stream: stream,
            source: ChannelAudio {
                sample_rate: 0,
                inner: None,
                config: arc_config.clone(),
            },
            tx,
            rx,
            rx_control,
            tx_paused,
            tx_path,
            tag: None,
            num: 0,
            loud: 0.0,
            paused: false,
            pause_after_song: false,
            quit_after_song: false,
            repeat: Repeat::Not,
            songlist: vec![],
            song_index: 0,
            arc_config,
            l10n,
            unsuccessful_tries: 0,
        })
    }
}
