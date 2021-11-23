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
use rodio::{Decoder, Source};

use crate::{
    buffer::Buffer,
    config::ArcConfig,
    err::Error,
    l10n::{messages::Message, L10n},
};

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
                //println!("{}: {}", $l10n.get($name, vec![]), v);
                $l10n.write(Message::$name(v.to_string()));
            }
        };
    }

    l10n.write(Message::PrintPlayingSong);

    simple!(l10n, title, Title);
    simple!(l10n, album, Album);
    simple!(l10n, artist, Artist);
    simple!(l10n, album_artist, AlbumArtist);
    simple!(l10n, year, Year);
    simple!(l10n, genre, Genre);
    simple!(l10n, date_recorded, DateRecorded);
    simple!(l10n, date_released, DateReleased);
    simple!(l10n, disc, Disc);
    simple!(l10n, total_discs, DiscsTotal);
    simple!(l10n, track, Track);
    simple!(l10n, total_tracks, TracksTotal);
    simple!(l10n, duration, Duration);

    for v in tag.lyrics()
    {
        l10n.write(Message::Lyrics(v));
    }
    for v in tag.synchronised_lyrics()
    {
        l10n.write(Message::SyncLyrics(format!("{:?}", v)));
    }
    for v in tag.comments()
    {
        l10n.write(Message::Comment(v));
    }

    l10n.write(Message::NumPictures(tag.pictures().count()));

    if tag.extended_links().next().is_some()
    {
        l10n.write(Message::ExtLinks);
        for v in tag.extended_links()
        {
            println!("{}", v);
        }
    }
    if tag.extended_texts().next().is_some()
    {
        l10n.write(Message::ExtTexts);
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
            l10n.write(Message::MetadataUnsupported(err.to_string()));
        }
        None => l10n.write(Message::PrintInfoUnreachable),
    }
}
