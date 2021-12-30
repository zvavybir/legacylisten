use std::{process, sync::atomic::Ordering, thread};

use rodio::Sink;

use crate::{
    audio::print_info, config::Config, l10n::messages::Message, matcher::BigAction, songs::Repeat,
};

use super::Command;

fn increase_likelihood(config: &mut Config) -> BigAction
{
    if config.num == u32::MAX
    {
        config.l10n.write(Message::MaxLikelihoodReached);
    }
    else
    {
        config.num += 1;
        config.l10n.write(Message::LikelihoodIncreased(config.num));
    }

    BigAction::Nothing
}

fn decrease_likelihood(config: &mut Config) -> BigAction
{
    if config.num == 0
    {
        config.l10n.write(Message::SongAlreadyNever);
    }
    else
    {
        config.num -= 1;
        config.l10n.write(Message::LikelihoodDecreased(config.num));

        if config.num == 0
        {
            config.l10n.write(Message::SongNever);
        }
    }

    BigAction::Nothing
}

fn quit(config: &mut Config) -> BigAction
{
    config.l10n.write(Message::StoppingProgram);
    BigAction::Quit
}

fn pause(config: &mut Config) -> BigAction
{
    if config.paused
    {
        config.l10n.write(Message::AlreadyPaused);
    }
    else
    {
        config.l10n.write(Message::Pausing);
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
        config.l10n.write(Message::Resuming);
        config.paused = false;
        config.sink.play();
    }
    else
    {
        config.l10n.write(Message::AlreadyRunning);
    }
    config.arc_config.update_dbus.store(true, Ordering::SeqCst);

    BigAction::Nothing
}

fn skip(config: &mut Config) -> BigAction
{
    config.l10n.write(Message::SkippingSong);
    config.paused = false;
    config.sink = Sink::try_new(&config.stream_handle).unwrap();
    config.arc_config.update_dbus.store(true, Ordering::SeqCst);

    BigAction::Nothing
}

fn increase_volume(config: &mut Config) -> BigAction
{
    config.loud += 0.01;
    config.l10n.write(Message::MakingLouder(config.loud as f64));
    config.sink.set_volume(config.loud);

    BigAction::Nothing
}

fn decrease_volume(config: &mut Config) -> BigAction
{
    config.loud -= 0.01;
    config
        .l10n
        .write(Message::MakingQuieter(config.loud as f64));
    if config.loud < 0.0
    {
        config.l10n.write(Message::LoudZero);
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
        config.l10n.write(Message::DurationKnown(pos, len));
    }
    else
    {
        config.l10n.write(Message::DurationUnknown(pos));
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
        config.l10n.write(Message::AlreadyQuitting);
    }
    else
    {
        config.l10n.write(Message::QuittingAfter);
        config.quit_after_song = true;
    };

    BigAction::Nothing
}

fn pause_after_song(config: &mut Config) -> BigAction
{
    if config.pause_after_song
    {
        config.l10n.write(Message::AlreadyPausing);
    }
    else
    {
        config.l10n.write(Message::PausingAfter);
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
            config.l10n.write(Message::OpeningPicture);
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
            config.l10n.write(Message::NothingPlayingYet);
        }
    }
    else
    {
        config.l10n.write(Message::CantOpenPicture);
    }

    BigAction::Nothing
}

fn disable_repeat(config: &mut Config) -> BigAction
{
    if config.repeat == Repeat::Not
    {
        config.l10n.write(Message::NotRepeatingAlready);
    }
    else
    {
        config.song_index += 1;
        config.l10n.write(Message::StoppingRepeating);
        config.repeat = Repeat::Not;
    }

    BigAction::Nothing
}

fn repeat_once(config: &mut Config) -> BigAction
{
    if config.repeat == Repeat::Once
    {
        config.l10n.write(Message::AlreadyRepeatingOnce);
    }
    else
    {
        if config.repeat == Repeat::Not
        {
            config.song_index -= 1;
        }
        config.l10n.write(Message::RepeatingOnce);
        config.repeat = Repeat::Once;
    }

    BigAction::Nothing
}

fn repeat_forever(config: &mut Config) -> BigAction
{
    if config.repeat == Repeat::Always
    {
        config.l10n.write(Message::AlreadyRepeatingForever);
    }
    else
    {
        if config.repeat == Repeat::Not
        {
            config.song_index -= 1;
        }
        config.l10n.write(Message::RepeatingForever);
        config.repeat = Repeat::Always;
    }

    BigAction::Nothing
}

fn skip_to_previous(config: &mut Config) -> BigAction
{
    if config.song_index == 0
    {
        config.l10n.write(Message::AlreadyPlayingFirst);
    }
    else
    {
        config.l10n.write(Message::Previous);
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
