use either::Either;
use fluent::types::FluentNumber;
use id3::frame::{Comment, Lyrics};
use Either::{Left, Right};

use crate::{commands::Command, Error};

pub enum LogLevel
{
    Error,
    Warn,
    Info,
    Debug,
    // TODO: Remove
    // Variant currently not used, but it will be.
    #[allow(dead_code)]
    Trace,
    Println,
    Unreachable,
}

pub enum Message<'a>
{
    TotalPlayingLikelihood(u32),
    HelpNotice,
    UnknownCommandChar(char),
    UnknownCommandByte(u8),
    InSignalHandler(i32),
    SignalHandlerUnreachable,
    MprisHandlerError(dbus::Error),
    MemoryTight,
    NewSongFound(String),
    NoSongs,
    PrintPlayingSong,
    Title(String),
    Album(String),
    Artist(String),
    AlbumArtist(String),
    Year(String),
    Genre(String),
    DateRecorded(String),
    DateReleased(String),
    Disc(String),
    DiscsTotal(String),
    Track(String),
    TracksTotal(String),
    Duration(String),
    Lyrics(&'a Lyrics),
    SyncLyrics(String),
    Comment(&'a Comment),
    NumPictures(usize),
    ExtLinks,
    ExtTexts,
    MetadataUnsupported(String),
    PrintInfoUnreachable,
    MaxLikelihoodReached,
    LikelihoodIncreased(u32),
    SongAlreadyNever,
    LikelihoodDecreased(u32),
    SongNever,
    StoppingProgram,
    AlreadyPaused,
    Pausing,
    Resuming,
    AlreadyRunning,
    SkippingSong,
    MakingLouder(f64),
    MakingQuieter(f64),
    LoudZero,
    DurationKnown(f64, f64),
    DurationUnknown(f64),
    AlreadyQuitting,
    QuittingAfter,
    AlreadyPausing,
    PausingAfter,
    OpeningPicture,
    NothingPlayingYet,
    CantOpenPicture,
    NotRepeatingAlready,
    StoppingRepeating,
    AlreadyRepeatingOnce,
    RepeatingOnce,
    AlreadyRepeatingForever,
    RepeatingForever,
    AlreadyPlayingFirst,
    HelpHeader,
    CommandReadingProblem,
    ReadingSongProblem(&'a str, Error),
    ChoosingNewSong,
    PlayingSong(String),
    PlayingSongUnknown,
    SongLikelihood(u32),
    SignalPaused,
    RequestedPause,
    SavingStateErr,
    StateSaved,
    UnknownTitle,
    UnknownArtist,
    Previous,
    ControllerOut,
    TooManyTries,
    Description(Command),
    PositiveBonus(u32),
    NegativeBonus(u32),
}

impl Message<'_>
{
    // It's not (nicely) possible otherwise; also it's a pedantic
    // lint.
    #[allow(clippy::too_many_lines)]
    pub const fn to_str(&self) -> &'static str
    {
        match self
        {
            Self::TotalPlayingLikelihood(_) => "total-playing-likelihood",
            Self::HelpNotice => "help-notice",
            Self::UnknownCommandChar(_) => "unknown-command-char",
            Self::UnknownCommandByte(_) => "unknown-command-byte",
            Self::InSignalHandler(_) => "in-signal-handler",
            Self::SignalHandlerUnreachable => "signal-handler-unreachable",
            Self::MprisHandlerError(_) => "mpris-handler-error",
            Self::MemoryTight => "memory-tight",
            Self::NewSongFound(_) => "new-song-found",
            Self::NoSongs => "no-songs",
            Self::PrintPlayingSong => "print-playing-song",
            Self::Title(_) => "title",
            Self::Album(_) => "album",
            Self::Artist(_) => "artist",
            Self::AlbumArtist(_) => "album-artist",
            Self::Year(_) => "year",
            Self::Genre(_) => "genre",
            Self::DateRecorded(_) => "date-recorded",
            Self::DateReleased(_) => "date-released",
            Self::Disc(_) => "disc",
            Self::DiscsTotal(_) => "discs-total",
            Self::Track(_) => "track",
            Self::TracksTotal(_) => "tracks-total",
            Self::Duration(_) => "duration",
            Self::Lyrics(_) => "lyrics",
            Self::SyncLyrics(_) => "sync-lyrics",
            Self::Comment(_) => "comment",
            Self::NumPictures(_) => "num-pictures",
            Self::ExtLinks => "ext-links",
            Self::ExtTexts => "ext-texts",
            Self::MetadataUnsupported(_) => "metadata-unsupported",
            Self::PrintInfoUnreachable => "print-info-unreachable",
            Self::MaxLikelihoodReached => "max-likelihood-reached",
            Self::LikelihoodIncreased(_) => "likelihood-increased",
            Self::SongAlreadyNever => "song-already-never",
            Self::LikelihoodDecreased(_) => "likelihood-decreased",
            Self::SongNever => "song-never",
            Self::StoppingProgram => "stopping-program",
            Self::AlreadyPaused => "already-paused",
            Self::Pausing => "pausing",
            Self::Resuming => "resuming",
            Self::AlreadyRunning => "already-running",
            Self::SkippingSong => "skipping-song",
            Self::MakingLouder(_) => "making-louder",
            Self::MakingQuieter(_) => "making-quieter",
            Self::LoudZero => "loud-zero",
            Self::DurationKnown(_, _) => "duration-known",
            Self::DurationUnknown(_) => "duration-unknown",
            Self::AlreadyQuitting => "already-quitting",
            Self::QuittingAfter => "quitting-after",
            Self::AlreadyPausing => "already-pausing",
            Self::PausingAfter => "pausing-after",
            Self::OpeningPicture => "opening-picture",
            Self::NothingPlayingYet => "nothing-playing-yet",
            Self::CantOpenPicture => "cant-open-picture",
            Self::NotRepeatingAlready => "not-repeating-already",
            Self::StoppingRepeating => "stopping-repeating",
            Self::AlreadyRepeatingOnce => "already-repeating-once",
            Self::RepeatingOnce => "repeating-once",
            Self::AlreadyRepeatingForever => "already-repeating-forever",
            Self::RepeatingForever => "repeating-forever",
            Self::AlreadyPlayingFirst => "already-playing-first",
            Self::HelpHeader => "help-header",
            Self::CommandReadingProblem => "command-reading-problem",
            Self::ReadingSongProblem(_, _) => "reading-song-problem",
            Self::ChoosingNewSong => "choosing-new-song",
            Self::PlayingSong(_) => "playing-song",
            Self::PlayingSongUnknown => "playing-song-unknown",
            Self::SongLikelihood(_) => "song-likelihood",
            Self::SignalPaused => "signal-paused",
            Self::RequestedPause => "requested-pause",
            Self::SavingStateErr => "saving-state-err",
            Self::StateSaved => "state-saved",
            Self::UnknownTitle => "unknown-title",
            Self::UnknownArtist => "unknown-artist",
            Self::Previous => "previous",
            Self::ControllerOut => "controller-out",
            Self::TooManyTries => "too-many-tries",
            Self::Description(Command::IncreaseLikelihood) => "increase-likelihood",
            Self::Description(Command::DecreaseLikelihood) => "decrease-likelihood",
            Self::Description(Command::Quit) => "quit",
            Self::Description(Command::Pause) => "pause",
            Self::Description(Command::Resume) => "resume",
            Self::Description(Command::Skip) => "skip",
            Self::Description(Command::IncreaseVolume) => "increase-volume",
            Self::Description(Command::DecreaseVolume) => "decrease-volume",
            Self::Description(Command::ShowDuration) => "show-duration",
            Self::Description(Command::SwitchPlayPause) => "switch-play-pause",
            Self::Description(Command::QuitAfterSong) => "quit-after-song",
            Self::Description(Command::PauseAfterSong) => "pause-after-song",
            Self::Description(Command::ShowInfo) => "show-info",
            Self::Description(Command::OpenCover) => "open-cover",
            Self::Description(Command::DisableRepeat) => "disable-repeat",
            Self::Description(Command::RepeatOnce) => "repeat-once",
            Self::Description(Command::RepeatForever) => "repeat-forever",
            Self::Description(Command::SkipToPrevious) => "skip-to-previous",
            Self::PositiveBonus(_) => "positive-bonus",
            Self::NegativeBonus(_) => "negative-bonus",
        }
    }

    pub fn into_vec(self) -> Vec<(&'static str, Either<String, FluentNumber>)>
    {
        match self
        {
            Self::TotalPlayingLikelihood(val) => vec![("val", Right(FluentNumber::from(val)))],
            Self::UnknownCommandChar(c) => vec![("char", Left(c.to_string()))],
            Self::UnknownCommandByte(b) => vec![("byte", Left(b.to_string()))],
            Self::InSignalHandler(sig) => vec![("sig", Left(sig.to_string()))],
            Self::MprisHandlerError(err) => vec![("err", Left(format!("{:?}", err)))],
            Self::NewSongFound(filename) => vec![("filename", Left(filename))],
            Self::Title(text)
            | Self::Album(text)
            | Self::Artist(text)
            | Self::AlbumArtist(text)
            | Self::Year(text)
            | Self::Genre(text)
            | Self::DateRecorded(text)
            | Self::DateReleased(text)
            | Self::Disc(text)
            | Self::DiscsTotal(text)
            | Self::Track(text)
            | Self::TracksTotal(text)
            | Self::Duration(text)
            | Self::SyncLyrics(text) => vec![("text", Left(text))],
            Self::Lyrics(text) => vec![("text", Left(text.to_string()))],
            Self::Comment(text) => vec![("text", Left(text.to_string()))],
            Self::NumPictures(text) => vec![("text", Left(text.to_string()))],
            Self::MetadataUnsupported(err) => vec![("err", Left(err))],
            Self::LikelihoodIncreased(num) | Self::LikelihoodDecreased(num) =>
            {
                vec![("likelihood", Right(FluentNumber::from(num)))]
            }
            Self::MakingLouder(loud) | Self::MakingQuieter(loud) =>
            {
                vec![("loud", Right(FluentNumber::from(loud * 100.0)))]
            }
            Self::DurationKnown(pos, len) => vec![
                ("pos", Right(FluentNumber::from(pos))),
                ("len", Right(FluentNumber::from(len))),
            ],
            Self::DurationUnknown(pos) => vec![("pos", Right(FluentNumber::from(pos)))],
            Self::ReadingSongProblem(path, err) => vec![
                ("path", Left(format!("{:?}", path))),
                ("err", Left(format!("{:?}", err))),
            ],
            Self::PlayingSong(s) => vec![("song", Left(s))],
            Self::SongLikelihood(num) => vec![("likelihood", Right(FluentNumber::from(num)))],
            Self::PositiveBonus(bonus) | Self::NegativeBonus(bonus) =>
            {
                vec![("bonus", Right(FluentNumber::from(bonus)))]
            }
            Self::HelpNotice
            | Self::SignalHandlerUnreachable
            | Self::PrintPlayingSong
            | Self::ExtLinks
            | Self::ExtTexts
            | Self::MemoryTight
            | Self::NoSongs
            | Self::PrintInfoUnreachable
            | Self::MaxLikelihoodReached
            | Self::SongAlreadyNever
            | Self::SongNever
            | Self::StoppingProgram
            | Self::AlreadyPaused
            | Self::Pausing
            | Self::Resuming
            | Self::AlreadyRunning
            | Self::SkippingSong
            | Self::LoudZero
            | Self::AlreadyQuitting
            | Self::QuittingAfter
            | Self::AlreadyPausing
            | Self::PausingAfter
            | Self::OpeningPicture
            | Self::NothingPlayingYet
            | Self::CantOpenPicture
            | Self::NotRepeatingAlready
            | Self::StoppingRepeating
            | Self::AlreadyRepeatingOnce
            | Self::RepeatingOnce
            | Self::AlreadyRepeatingForever
            | Self::RepeatingForever
            | Self::AlreadyPlayingFirst
            | Self::HelpHeader
            | Self::CommandReadingProblem
            | Self::ChoosingNewSong
            | Self::PlayingSongUnknown
            | Self::SignalPaused
            | Self::RequestedPause
            | Self::SavingStateErr
            | Self::StateSaved
            | Self::UnknownTitle
            | Self::UnknownArtist
            | Self::Previous
            | Self::ControllerOut
            | Self::TooManyTries
            | Self::Description(_) => vec![],
        }
    }

    pub const fn loglevel(&self) -> LogLevel
    {
        match self
        {
            Self::MemoryTight
            | Self::NoSongs
            | Self::PrintInfoUnreachable
            | Self::CommandReadingProblem
            | Self::SavingStateErr
            | Self::TooManyTries => LogLevel::Error,
            Self::UnknownCommandChar(_)
            | Self::UnknownCommandByte(_)
            | Self::MprisHandlerError(_)
            | Self::MetadataUnsupported(_)
            | Self::SongAlreadyNever
            | Self::SongNever
            | Self::MaxLikelihoodReached
            | Self::AlreadyQuitting
            | Self::AlreadyPausing
            | Self::CantOpenPicture
            | Self::NotRepeatingAlready
            | Self::AlreadyRepeatingOnce
            | Self::AlreadyRepeatingForever
            | Self::AlreadyPlayingFirst
            | Self::ReadingSongProblem(_, _) => LogLevel::Warn,
            Self::TotalPlayingLikelihood(_)
            | Self::HelpNotice
            | Self::NewSongFound(_)
            | Self::LikelihoodIncreased(_)
            | Self::LikelihoodDecreased(_)
            | Self::StoppingProgram
            | Self::AlreadyPaused
            | Self::Pausing
            | Self::Resuming
            | Self::AlreadyRunning
            | Self::SkippingSong
            | Self::MakingLouder(_)
            | Self::MakingQuieter(_)
            | Self::LoudZero
            | Self::DurationKnown(_, _)
            | Self::DurationUnknown(_)
            | Self::QuittingAfter
            | Self::PausingAfter
            | Self::OpeningPicture
            | Self::NothingPlayingYet
            | Self::StoppingRepeating
            | Self::RepeatingOnce
            | Self::RepeatingForever
            | Self::ChoosingNewSong
            | Self::PlayingSong(_)
            | Self::PlayingSongUnknown
            | Self::SongLikelihood(_)
            | Self::SignalPaused
            | Self::RequestedPause
            | Self::StateSaved
            | Self::Previous
            | Self::PositiveBonus(_)
            | Self::NegativeBonus(_) => LogLevel::Info,
            Self::InSignalHandler(_) | Self::PrintPlayingSong => LogLevel::Debug,
            Self::Title(_)
            | Self::Album(_)
            | Self::Artist(_)
            | Self::AlbumArtist(_)
            | Self::Year(_)
            | Self::Genre(_)
            | Self::DateRecorded(_)
            | Self::DateReleased(_)
            | Self::Disc(_)
            | Self::DiscsTotal(_)
            | Self::Track(_)
            | Self::TracksTotal(_)
            | Self::Duration(_)
            | Self::Lyrics(_)
            | Self::SyncLyrics(_)
            | Self::Comment(_)
            | Self::NumPictures(_)
            | Self::ExtLinks
            | Self::ExtTexts
            | Self::HelpHeader
            | Self::UnknownTitle
            | Self::UnknownArtist
            | Self::ControllerOut
            | Self::Description(_) => LogLevel::Println,
            Self::SignalHandlerUnreachable => LogLevel::Unreachable,
        }
    }
}
