use std::{
    fs::File,
    path::Path,
    sync::{
        atomic::Ordering,
        mpsc::{sync_channel, Receiver},
        Arc,
    },
    thread,
    time::Duration,
};

use id3::Tag;
use log::{debug, error, warn};
use rodio::{Decoder, Source};

use crate::{buffer::Buffer, config::ArcConfig, err::Error, l10n::L10n};

pub struct ChannelSource
{
    channels: u16,
    sample_rate: u32,
    data_rx: Receiver<(usize, i16)>,
    config: Arc<ArcConfig>,
}

// I think it sounds better
#[allow(clippy::module_name_repetitions)]
pub struct ChannelAudio
{
    pub sample_rate: u32,
    pub inner: Option<ChannelSource>,
    pub config: Arc<ArcConfig>,
}

fn get_size<P: AsRef<Path>>(path: P) -> Result<usize, Error>
{
    let decoder2 = Decoder::new(Buffer::new(File::open(path)?)?)?;

    Ok(decoder2.count())
}

impl ChannelAudio
{
    pub fn new<P: AsRef<Path>>(path: P, config: Arc<ArcConfig>) -> Result<Self, Error>
    {
        let (data_tx, data_rx) = sync_channel(64);

        let decoder = Decoder::new(Buffer::new(File::open(&path)?)?)?;
        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();

        config.current_len.store(0, Ordering::SeqCst);
        config.current_pos.store(0, Ordering::SeqCst);
        config
            .sample_rate
            .store(sample_rate as usize, Ordering::SeqCst);
        config.channels.store(channels as usize, Ordering::SeqCst);

        let mut size_already_safed = false;
        if let (min, Some(max)) = decoder.size_hint()
        {
            if min == max && min != 0
            {
                config.current_len.store(min, Ordering::SeqCst);
                size_already_safed = true;
            }
        };

        thread::spawn(move || {
            decoder.enumerate().for_each(|sample| {
                let _ = data_tx.send(sample);
            });
        });

        let path = path.as_ref().to_path_buf();
        let config2 = config.clone();
        thread::spawn(move || {
            if !size_already_safed
            {
                if let Ok(size) = get_size(path)
                {
                    config2.current_len.store(size, Ordering::SeqCst);
                    config2.update_dbus.store(true, Ordering::SeqCst);
                }
            }
        });

        Ok(Self {
            sample_rate,
            config: config.clone(),
            inner: Some(ChannelSource {
                channels,
                sample_rate,
                data_rx,
                config,
            }),
        })
    }

    pub fn get_pos(&mut self) -> usize
    {
        self.config.current_pos.load(Ordering::SeqCst)
    }

    pub fn samples_len(&mut self) -> Option<usize>
    {
        let len = self.config.current_len.load(Ordering::SeqCst);

        if len == 0
        {
            None
        }
        else
        {
            Some(len)
        }
    }
}

impl Iterator for ChannelSource
{
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item>
    {
        if let Ok((i, val)) = self.data_rx.recv()
        {
            self.config.current_pos.store(i, Ordering::SeqCst);
            Some(val)
        }
        else
        {
            None
        }
    }
}

impl Source for ChannelSource
{
    fn current_frame_len(&self) -> Option<usize>
    {
        None
    }

    fn channels(&self) -> u16
    {
        self.channels
    }

    fn sample_rate(&self) -> u32
    {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration>
    {
        None
    }
}

fn print_tag(tag: &Tag, l10n: L10n)
{
    macro_rules! simple {
        ( $l10n : tt , $method : tt , $name : tt ) => {
            if let Some(v) = tag.$method()
            {
                println!("{}: {}", $l10n.get($name, vec![]), v);
            }
        };
    }

    debug!("{}", l10n.get("print-playing-song", vec![]));

    simple!(l10n, title, "title");
    simple!(l10n, album, "album");
    simple!(l10n, artist, "artist");
    simple!(l10n, album_artist, "album-artist");
    simple!(l10n, year, "year");
    simple!(l10n, genre, "genre");
    simple!(l10n, date_recorded, "date-recorded");
    simple!(l10n, date_released, "date-released");
    simple!(l10n, disc, "disc");
    simple!(l10n, total_discs, "discs-total");
    simple!(l10n, track, "track");
    simple!(l10n, total_tracks, "tracks-total");
    simple!(l10n, duration, "duration");

    for v in tag.lyrics()
    {
        println!("{}: {}", l10n.get("lyrics", vec![]), v);
    }
    for v in tag.synchronised_lyrics()
    {
        println!("{}: {:?}", l10n.get("sync-lyrics", vec![]), v);
    }
    for v in tag.comments()
    {
        println!("{}: {}", l10n.get("comment", vec![]), v);
    }

    println!(
        "{}: {}",
        l10n.get("num-pictures", vec![]),
        tag.pictures().count()
    );

    if tag.extended_links().next().is_some()
    {
        println!("{}", l10n.get("ext-links", vec![]));
        for v in tag.extended_links()
        {
            println!("{}", v);
        }
    }
    if tag.extended_texts().next().is_some()
    {
        println!("{}", l10n.get("ext-texts", vec![]));
        for v in tag.extended_texts()
        {
            println!("{}", v);
        }
    }
}

pub fn print_info(tag: &Option<Result<Tag, id3::Error>>, l10n: L10n)
{
    match tag
    {
        Some(Ok(tag)) => print_tag(tag, l10n),
        Some(Err(err)) =>
        {
            warn!("{}: {:?}", l10n.get("metadata-unsupported", vec![]), err);
        }
        None => error!("{}", l10n.get("print-info-unreachable", vec![])),
    }
}
