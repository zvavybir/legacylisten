use std::{convert::TryFrom, fmt};

use num_enum::TryFromPrimitive;

use crate::l10n::L10n;

mod impls;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, TryFromPrimitive)]
pub enum Command
{
    IncreaseLikelihood,
    DecreaseLikelihood,
    Quit,
    Pause,
    Resume,
    Skip,
    IncreaseVolume,
    DecreaseVolume,
    ShowDuration,
    SwitchPlayPause,
    QuitAfterSong,
    PauseAfterSong,
    ShowInfo,
    OpenCover,
    DisableRepeat,
    RepeatOnce,
    RepeatForever,
    SkipToPrevious,
}

#[derive(Clone, Copy)]
struct DisplayCommand(Command, L10n);

impl fmt::Display for DisplayCommand
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error>
    {
        let key = match self.0
        {
            Command::IncreaseLikelihood => "increase-likelihood",
            Command::DecreaseLikelihood => "decrease-likelihood",
            Command::Quit => "quit",
            Command::Pause => "pause",
            Command::Resume => "resume",
            Command::Skip => "skip",
            Command::IncreaseVolume => "increase-volume",
            Command::DecreaseVolume => "decrease-volume",
            Command::ShowDuration => "show-duration",
            Command::SwitchPlayPause => "switch-play-pause",
            Command::QuitAfterSong => "quit-after-song",
            Command::PauseAfterSong => "pause-after-song",
            Command::ShowInfo => "show-info",
            Command::OpenCover => "open-cover",
            Command::DisableRepeat => "disable-repeat",
            Command::RepeatOnce => "repeat-once",
            Command::RepeatForever => "repeat-forever",
            Command::SkipToPrevious => "skip-to-previous",
        };

        write!(f, "{}", self.1.get(key, vec![]))
    }
}

impl Command
{
    pub fn show_help(lang: L10n)
    {
        println!("{}", lang.get("help-header", vec![]));
        for (i, command) in (0..).map_while(|i| Self::try_from(i).map(|c| (i, c)).ok())
        {
            debug_assert!(i < 26);
            println!("{}): {}", (i + b'a') as char, DisplayCommand(command, lang));
        }
    }
}
