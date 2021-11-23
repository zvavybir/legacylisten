use std::{convert::TryFrom, fmt};

use num_enum::TryFromPrimitive;

use crate::l10n::{messages::Message, L10n};

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
        write!(f, "{}", self.1.get(Message::Description(self.0)))
    }
}

impl Command
{
    pub fn show_help(lang: L10n)
    {
        lang.write(Message::HelpHeader);
        for (i, command) in (0..).map_while(|i| Self::try_from(i).map(|c| (i, c)).ok())
        {
            debug_assert!(i < 26);
            println!("{}): {}", (i + b'a') as char, DisplayCommand(command, lang));
        }
    }
}
