//! Entry point for `legacylisten`
//!
//! This is the top level module and `legacylisten` is started by
//! calling the `run()` function.

use std::{
    sync::{atomic::Ordering, mpsc},
    thread,
    time::Duration,
};

use id3::Tag;
use signal_hook::{
    consts::{SIGINT, SIGTERM, SIGUSR1, SIGUSR2},
    iterator::Signals,
};
use simple_logger::SimpleLogger;

use crate::{
    audio::ChannelAudio,
    config::Config,
    err::Error,
    helpers::take_error,
    l10n::messages::Message,
    matcher::{main_match, BigAction},
    songs::{Song, Songs},
    threads::start_threads,
};

// Called by songs::choose_random.
fn handle_song(song: &mut Song, config: &mut Config) -> BigAction
{
    let data_dir = &config.arc_config.conffile.data_dir;
    let song_path = data_dir.join(song.name.clone());
    let (tag, tag_option) = take_error(Tag::read_from_path(&song_path));
    config.tag = Some(tag);
    config.source = match ChannelAudio::new(&song_path, config.arc_config.clone())
    {
        Ok(source) => source,
        Err(e) =>
        {
            config
                .l10n
                .write(Message::ReadingSongProblem(&song.name, e));

            if config.unsuccessful_tries == 255
            {
                config.l10n.write(Message::TooManyTries);
                return BigAction::Quit;
            }
            config.unsuccessful_tries += 1;

            config.l10n.write(Message::ChoosingNewSong);
            return BigAction::Skip;
        }
    };

    config.unsuccessful_tries = 0;

    config.num = song.num;
    config.loud = song.loud;

    config.sink.append(config.source.inner.take().unwrap());
    config.sink.set_volume(song.loud);

    if let Ok(s) = data_dir
        .join(song.name.clone())
        .into_os_string()
        .into_string()
    {
        config.l10n.write(Message::PlayingSong(s));
    }
    else
    {
        config.l10n.write(Message::PlayingSongUnknown);
    }

    config.l10n.write(Message::SongLikelihood(song.num));

    config.arc_config.update_dbus.store(true, Ordering::SeqCst);
    while !config.sink.empty()
    {
        match main_match(config)
        {
            BigAction::Nothing =>
            {
                song.num = config.num;
                song.loud = config.loud;
            }
            x => return x,
        }
        if config.rx_control.try_recv().is_ok()
        {
            let _ = config.tx_paused.send(config.paused);
            let _ = config.tx_path.send((song_path.clone(), tag_option.clone()));
        }
        if config.arc_config.reading_paused.load(Ordering::SeqCst)
        {
            config.l10n.write(Message::SignalPaused);
        }
        // To prevent busy loop
        thread::sleep(Duration::from_micros(1));
    }

    BigAction::Nothing
}

fn handle_pausely(config: &mut Config) -> bool
{
    if config.quit_after_song
    {
        // Returns only `true` when legacylisten should quit.
        return true;
    }
    if config.pause_after_song
    {
        config.l10n.write(Message::RequestedPause);
        config.sink.pause();
        config.paused = true;
        config.pause_after_song = false;
    }
    while config
        .arc_config
        .reading_paused
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        // Is there any way to make that better than a poll loop?
        thread::sleep(Duration::from_millis(1));
    }

    false
}

/// Entry point for `legacylisten`
///
/// By calling this function `legacylisten` is started.
/// # Panics
/// It will panic if an fatal condition is encountered and it can't be
/// passed down as [`Error`](Error).
/// # Errors
/// It will return an error if an fatal condition occurs and it
/// actually can be passed down.
pub fn run() -> Result<(), Error>
{
    SimpleLogger::new().init().unwrap();

    // Initializing some channels for communication between some
    // far-away parts.  Better than the original globals, but still
    // not how I'd like it.
    let (tx_control, rx_control) = mpsc::channel();
    let (tx_paused, rx_paused) = mpsc::channel();
    let (tx_path, rx_path) = mpsc::channel();
    // Initializing the configuration; nearly every function gets a
    // reference to that.
    let mut config = Config::new(rx_control, tx_paused, tx_path)?;
    // Reading the likelihoods and volumes of all songs.
    let mut songs = Songs::read(config.arc_config.clone(), config.l10n)?;
    // Copied to make the borrowck happy.
    let l10n = config.l10n;

    // Starts a couple minor threads.
    start_threads(
        config.tx.clone(),
        tx_control,
        rx_paused,
        rx_path,
        Signals::new(&[SIGINT, SIGTERM, SIGUSR1, SIGUSR2])?,
        config.arc_config.clone(),
        config.l10n,
    );

    loop
    {
        // There are multiple ways of pausing; handle all of them.
        if handle_pausely(&mut config)
        {
            break;
        }

        // A new song is about to be chosen, so notice dbus.
        config.arc_config.update_dbus.store(true, Ordering::SeqCst);

        // Choose a new song, play it and handle everthing else.  Name
        // might be a bit of a misnomer, since it does more than
        // choosing a song.  If through a command or something else,
        // we should quit the `break` handles that.
        match songs.choose_random(&mut config, handle_song, l10n)
        {
            BigAction::Nothing | BigAction::Skip =>
            {}
            BigAction::Quit => break,
        }
    }

    config
        .l10n
        .write(Message::TotalPlayingLikelihood(songs.total_likelihood()));

    Ok(())
}
