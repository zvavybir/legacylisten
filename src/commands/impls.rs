use std::{process, sync::atomic::Ordering, thread};

use log::{info, warn};
use rodio::Sink;

use crate::{audio::print_info, config::Config, matcher::BigAction, songs::Repeat};

use super::Command;

fn increase_likelihood(config: &mut Config) -> BigAction
{
    if config.num == u32::MAX
    {
        warn!("{}", config.l10n.get("max-likelihood-reached", vec![]));
    }
    else
    {
        config.num += 1;
        info!(
            "{}",
            config
                .l10n
                .get_num("likelihood-increased", vec![("likelihood", config.num)])
        );
    }

    BigAction::Nothing
}

fn decrease_likelihood(config: &mut Config) -> BigAction
{
    if config.num == 0
    {
        warn!("{}", config.l10n.get("song-already-never", vec![]));
    }
    else
    {
        config.num -= 1;
        info!(
            "{}",
            config
                .l10n
                .get_num("likelihood-decreased", vec![("likelihood", config.num)])
        );

        if config.num == 0
        {
            warn!("{}", config.l10n.get("song-never", vec![]));
        }
    }

    BigAction::Nothing
}

fn quit(config: &mut Config) -> BigAction
{
    info!("{}", config.l10n.get("stopping-program", vec![]));
    BigAction::Quit
}

fn pause(config: &mut Config) -> BigAction
{
    if config.paused
    {
        info!("{}", config.l10n.get("already-paused", vec![]));
    }
    else
    {
        info!("{}", config.l10n.get("pausing", vec![]));
        config.paused = true;
        config.sink.pause();
    }
    config.arc_config.update_dbus.store(true, Ordering::SeqCst);

    BigAction::Nothing
}

fn resume(config: &mut Config) -> BigAction
{
    if config.paused
    {
        info!("{}", config.l10n.get("resuming", vec![]));
        config.paused = false;
        config.sink.play();
    }
    else
    {
        info!("{}", config.l10n.get("already-running", vec![]));
    }
    config.arc_config.update_dbus.store(true, Ordering::SeqCst);

    BigAction::Nothing
}

fn skip(config: &mut Config) -> BigAction
{
    info!("{}", config.l10n.get("skipping-song", vec![]));
    config.repeat = Repeat::Not;
    config.paused = false;
    config.sink = Sink::try_new(&config.stream_handle).unwrap();
    config.arc_config.update_dbus.store(true, Ordering::SeqCst);

    BigAction::Nothing
}

fn increase_volume(config: &mut Config) -> BigAction
{
    config.loud += 0.01;
    info!(
        "{}",
        config
            .l10n
            .get_per("making-louder", vec![("loud", config.loud as f64 * 100.0)])
    );
    config.sink.set_volume(config.loud);

    BigAction::Nothing
}

fn decrease_volume(config: &mut Config) -> BigAction
{
    config.loud -= 0.01;
    info!(
        "{}",
        config
            .l10n
            .get_per("making-quieter", vec![("loud", config.loud as f64 * 100.0)])
    );
    if config.loud < 0.0
    {
        info!("{}", config.l10n.get("loud-zero", vec![]));
        config.loud = 0.0;
    }
    config.sink.set_volume(config.loud);

    BigAction::Nothing
}

fn show_duration(config: &mut Config) -> BigAction
{
    let pos = config.source.get_pos() as f64
        / config.source.sample_rate as f64
        / config.arc_config.channels.load(Ordering::SeqCst) as f64;

    if let Some(len) = config.source.samples_len()
    {
        let len = len as f64
            / config.source.sample_rate as f64
            / config.arc_config.channels.load(Ordering::SeqCst) as f64;
        info!(
            "{}",
            config
                .l10n
                .get_num("duration-known", vec![("pos", pos), ("len", len)])
        );
    }
    else
    {
        info!(
            "{}",
            config.l10n.get_num("duration-unknown", vec![("pos", pos)])
        );
    }

    BigAction::Nothing
}

fn switch_play_pause(config: &mut Config) -> BigAction
{
    if config.paused
    {
        let _ = config.tx.send(Command::Resume);
    }
    else
    {
        let _ = config.tx.send(Command::Quit);
    }

    BigAction::Nothing
}

fn quit_after_song(config: &mut Config) -> BigAction
{
    if config.quit_after_song
    {
        warn!("{}", config.l10n.get("already-quitting", vec![]));
    }
    else
    {
        info!("{}", config.l10n.get("quitting-after", vec![]));
        config.quit_after_song = true;
    };

    BigAction::Nothing
}

fn pause_after_song(config: &mut Config) -> BigAction
{
    if config.pause_after_song
    {
        warn!("{}", config.l10n.get("already-pausing", vec![]));
    }
    else
    {
        info!("{}", config.l10n.get("pausing-after", vec![]));
        config.pause_after_song = true;
    };

    BigAction::Nothing
}

fn show_info(config: &mut Config) -> BigAction
{
    print_info(&config.tag, config.l10n);

    BigAction::Nothing
}

fn open_cover(config: &mut Config) -> BigAction
{
    if let Ok(pic_path) = config.arc_config.pic_path.lock()
    {
        if let Some(pic_path) = pic_path.clone()
        {
            info!("{}", config.l10n.get("opening-picture", vec![]));
            thread::spawn(|| {
                let _ = process::Command::new("mimeopen")
                    .arg("-")
                    .arg(pic_path)
                    .spawn()
                    .and_then(|mut handle| handle.wait());
            });
        }
        else
        {
            info!("{}", config.l10n.get("nothing-playing-yet", vec![]));
        }
    }
    else
    {
        warn!("{}", config.l10n.get("cant-open-picture", vec![]));
    }

    BigAction::Nothing
}

fn disable_repeat(config: &mut Config) -> BigAction
{
    if config.repeat == Repeat::Not
    {
        warn!("{}", config.l10n.get("not-repeating-already", vec![]));
    }
    else
    {
        info!("{}", config.l10n.get("stopping-repeating", vec![]));
        config.repeat = Repeat::Not;
    }

    BigAction::Nothing
}

fn repeat_once(config: &mut Config) -> BigAction
{
    if config.repeat == Repeat::Once
    {
        warn!("{}", config.l10n.get("already-repeating-once", vec![]));
    }
    else
    {
        info!("{}", config.l10n.get("repeating-once", vec![]));
        config.repeat = Repeat::Once;
    }

    BigAction::Nothing
}

fn repeat_forever(config: &mut Config) -> BigAction
{
    if config.repeat == Repeat::Always
    {
        warn!("{}", config.l10n.get("already-repeating-forever", vec![]));
    }
    else
    {
        info!("{}", config.l10n.get("repeating-forever", vec![]));
        config.repeat = Repeat::Always;
    }

    BigAction::Nothing
}

fn skip_to_previous(config: &mut Config) -> BigAction
{
    if config.song_index == 0
    {
        warn!("{}", config.l10n.get("already-playing-first", vec![]));
    }
    else
    {
        info!("{}", config.l10n.get("previous", vec![]));
        config.song_index -= 1;
    }

    BigAction::Nothing
}

impl Command
{
    pub fn get_handler(self) -> fn(&mut Config) -> BigAction
    {
        match self
        {
            Self::IncreaseLikelihood => increase_likelihood,
            Self::DecreaseLikelihood => decrease_likelihood,
            Self::Quit => quit,
            Self::Pause => pause,
            Self::Resume => resume,
            Self::Skip => skip,
            Self::IncreaseVolume => increase_volume,
            Self::DecreaseVolume => decrease_volume,
            Self::ShowDuration => show_duration,
            Self::SwitchPlayPause => switch_play_pause,
            Self::QuitAfterSong => quit_after_song,
            Self::PauseAfterSong => pause_after_song,
            Self::ShowInfo => show_info,
            Self::OpenCover => open_cover,
            Self::DisableRepeat => disable_repeat,
            Self::RepeatOnce => repeat_once,
            Self::RepeatForever => repeat_forever,
            Self::SkipToPrevious => skip_to_previous,
        }
    }
}
