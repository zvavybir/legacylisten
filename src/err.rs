use std::{error, fmt, io, path::StripPrefixError, string::FromUtf8Error, sync::mpsc::RecvError};

use fluent::FluentError;
use fluent_syntax::parser::ParserError;
use nix::errno::Errno;
use rodio::{decoder::DecoderError, PlayError, StreamError};
use unic_langid::LanguageIdentifierError;

/// `legacylisten`'s Error type
///
/// This type bundles all errors which could occur.
#[derive(Debug)]
pub enum Error
{
    Io(io::Error),
    Trash(trash::Error),
    Decoder(DecoderError),
    Stream(StreamError),
    Play(PlayError),
    Errno(Errno),
    Utf8(FromUtf8Error),
    /// The (pseudo-)CSV file which stores the playing likelihoods and
    /// the volumes was malformatted.
    MalformattedSongsCsv,
    /// Language id have to be correct, not only so that minor things
    /// like decimal separator are handled correctly, but also because
    /// fluent will complain if it can't parse it.
    LangId(LanguageIdentifierError),
    Fluent(FluentError),
    FluentParse(ParserError),
    Recv(RecvError),
    Walkdir(walkdir::Error),
    StripPrefixError(StripPrefixError),
    Custom(String),
    Vec(Vec<Error>),
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error>
    {
        match self
        {
            Self::Io(err) => write!(f, "IO error: {}", err),
            Self::Trash(err) => write!(f, "Trash error: {}", err),
            Self::Decoder(err) => write!(f, "Decoder error: {}", err),
            Self::Stream(err) => write!(f, "Stream error: {}", err),
            Self::Play(err) => write!(f, "Play error: {}", err),
            Self::Errno(err) => write!(f, "Errno: {}", err),
            Self::Utf8(err) => write!(f, "From UTF8 error: {}", err),
            Self::MalformattedSongsCsv => write!(f, "Malformatted songs.csv file"),
            Self::LangId(err) => write!(f, "Malformated lang id: {}", err),
            Self::Fluent(err) => write!(f, "Fluent error: {}", err),
            Self::FluentParse(err) => write!(f, "Fluent parse error: {}", err),
            Self::Recv(err) => write!(f, "Recv error: {}", err),
            Self::Walkdir(err) => write!(f, "Walkdir error: {}", err),
            Self::StripPrefixError(err) => write!(f, "Strip prefix error: {}", err),
            Self::Custom(err) => write!(f, "Custom error: {}", err),
            Self::Vec(v) =>
            {
                if v.len() == 1
                {
                    write!(f, "{}", v[0])
                }
                else
                {
                    writeln!(f, "Multiple errors ({}):", v.len())?;
                    for (i, e) in v.iter().enumerate()
                    {
                        write!(f, "{}: {}", i, e)?;
                        if i != v.len() + 1
                        {
                            writeln!(f)?;
                        }
                    }

                    Ok(())
                }
            }
        }
    }
}

impl From<io::Error> for Error
{
    fn from(err: io::Error) -> Self
    {
        Self::Io(err)
    }
}

impl From<trash::Error> for Error
{
    fn from(err: trash::Error) -> Self
    {
        Self::Trash(err)
    }
}

impl From<DecoderError> for Error
{
    fn from(err: DecoderError) -> Self
    {
        Self::Decoder(err)
    }
}

impl From<StreamError> for Error
{
    fn from(err: StreamError) -> Self
    {
        Self::Stream(err)
    }
}

impl From<PlayError> for Error
{
    fn from(err: PlayError) -> Self
    {
        Self::Play(err)
    }
}

impl From<Errno> for Error
{
    fn from(err: Errno) -> Self
    {
        Self::Errno(err)
    }
}

impl From<FromUtf8Error> for Error
{
    fn from(err: FromUtf8Error) -> Self
    {
        Self::Utf8(err)
    }
}

impl error::Error for Error {}

impl From<LanguageIdentifierError> for Error
{
    fn from(err: LanguageIdentifierError) -> Self
    {
        Self::LangId(err)
    }
}

impl From<FluentError> for Error
{
    fn from(err: FluentError) -> Self
    {
        Self::Fluent(err)
    }
}

impl From<ParserError> for Error
{
    fn from(err: ParserError) -> Self
    {
        Self::FluentParse(err)
    }
}

impl From<RecvError> for Error
{
    fn from(err: RecvError) -> Self
    {
        Self::Recv(err)
    }
}

impl From<walkdir::Error> for Error
{
    fn from(err: walkdir::Error) -> Self
    {
        Self::Walkdir(err)
    }
}

impl From<StripPrefixError> for Error
{
    fn from(err: StripPrefixError) -> Self
    {
        Self::StripPrefixError(err)
    }
}

impl From<String> for Error
{
    fn from(err: String) -> Self
    {
        Self::Custom(err)
    }
}

impl<T> From<Vec<T>> for Error
where
    Self: From<T>,
{
    fn from(err: Vec<T>) -> Self
    {
        Self::Vec(err.into_iter().map(Into::into).collect())
    }
}
