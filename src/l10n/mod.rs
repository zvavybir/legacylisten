use std::{
    fs::File,
    io::Read,
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Mutex,
    },
    thread,
};

use either::Either;
use fluent::{
    types::{FluentNumber, FluentNumberOptions, FluentNumberStyle},
    FluentArgs, FluentBundle, FluentResource, FluentValue,
};
use unic_langid::LanguageIdentifier;

use crate::{
    helpers::ResultExtend,
    l10n::{
        english::{ENGLISH_L10N_LANG_ID, ENGLISH_L10N_STR},
        german::{GERMAN_L10N_LANG_ID, GERMAN_L10N_STR},
    },
    Error,
};

mod english;
mod german;

#[derive(Clone, Debug)]
pub enum Lang
{
    English,
    German,
    Custom(PathBuf, LanguageIdentifier),
}

struct L10nInner
{
    bundle: FluentBundle<FluentResource>,
    fallback: Option<FluentBundle<FluentResource>>,
}

type KeyType = &'static str;
type ArgSliceType = Vec<(&'static str, Either<String, FluentNumber>)>;
type Command = Sender<(KeyType, ArgSliceType)>;
type Answer = Receiver<String>;

#[derive(Copy, Clone)]
pub struct L10n
{
    inner: &'static Mutex<(Command, Answer)>,
}

impl L10nInner
{
    fn new(lang: Lang) -> Result<Self, Error>
    {
        let (s, lang, fallback) = match lang
        {
            Lang::English => (
                String::from(ENGLISH_L10N_STR),
                ENGLISH_L10N_LANG_ID.parse()?,
                None,
            ),
            Lang::German => (
                String::from(GERMAN_L10N_STR),
                GERMAN_L10N_LANG_ID.parse()?,
                Some((
                    String::from(ENGLISH_L10N_STR),
                    ENGLISH_L10N_LANG_ID.parse()?,
                )),
            ),
            Lang::Custom(path, lang) =>
            {
                let mut buf = String::new();
                File::open(path)?.read_to_string(&mut buf)?;

                (
                    buf,
                    lang,
                    Some((
                        String::from(ENGLISH_L10N_STR),
                        ENGLISH_L10N_LANG_ID.parse()?,
                    )),
                )
            }
        };

        let mut bundle = FluentBundle::new(vec![lang]);
        bundle.add_resource(FluentResource::try_new(s).map_err(|(_, x)| x)?)?;

        let fallback = if let Some((s, lang)) = fallback
        {
            let mut bundle = FluentBundle::new(vec![lang]);
            bundle.add_resource(FluentResource::try_new(s).map_err(|(_, x)| x)?)?;
            Some(bundle)
        }
        else
        {
            None
        };

        Ok(Self { bundle, fallback })
    }

    fn get_raw(
        bundle: &FluentBundle<FluentResource>,
        key: &str,
        args: Option<&FluentArgs>,
    ) -> Result<String, Error>
    {
        let mut errors = vec![];

        let msg = bundle.format_pattern(
            bundle
                .get_message(key)
                .ok_or_else(|| format!("Message doesn't exist: {:?}", key))
                .map(|msg| {
                    msg.value()
                        .ok_or_else(|| format!("Message has no value: {:?}", key))
                })
                .flatten_stable()?,
            args,
            &mut errors,
        );

        if !errors.is_empty()
        {
            return Err(errors.into());
        }

        Ok(msg.to_string())
    }

    fn get(&self, key: &KeyType, arg_slice: ArgSliceType) -> String
    {
        let args = if arg_slice.is_empty()
        {
            None
        }
        else
        {
            let mut args = FluentArgs::new();
            for (key, value) in arg_slice
            {
                match value
                {
                    Either::Left(s) => args.set(key, FluentValue::from(s)),
                    Either::Right(s) => args.set(key, FluentValue::from(s)),
                }
            }

            Some(args)
        };

        // To big to do readable in a `map_or_else`; nursery lint
        #[allow(clippy::option_if_let_else)]
        if let Some(fallback) = &self.fallback
        {
            Self::get_raw(&self.bundle, key, args.as_ref())
                .map_err(|_| Self::get_raw(fallback, key, args.as_ref()))
                .flatten_stable()
                .expect("Error with l10n")
        }
        else
        {
            Self::get_raw(&self.bundle, key, args.as_ref()).expect("Error with l10n")
        }
    }
}

impl L10n
{
    pub fn new(lang: Lang) -> Result<Self, Error>
    {
        let (tx_error, rx_error) = channel();
        let (tx_com, rx_com) = channel();
        let (tx_data, rx_data) = channel();

        thread::spawn(move || match L10nInner::new(lang)
        {
            Ok(lang) =>
            {
                tx_error
                    .send(Ok(Self {
                        inner: Box::leak(Box::new(Mutex::new((tx_com, rx_data)))),
                    }))
                    .expect("Fail to initialise l10n");

                while let Ok((key, arg_slice)) = rx_com.recv()
                {
                    tx_data
                        .send(lang.get(&key, arg_slice))
                        .expect("Failed to answer l10n info");
                }
            }
            Err(err) => tx_error
                .send(Err(err))
                .expect("Failed to signal the failing of initialising of l10n"),
        });

        rx_error.recv().map_err(Into::into).and_then(|x| x)
    }

    pub fn get_raw(self, key: &'static str, arg_slice: ArgSliceType) -> String
    {
        let lock = self
            .inner
            .lock()
            .expect("Lock over l10n struct is poisoned");

        lock.0
            .send((key, arg_slice))
            .expect("Can't request l10n info");
        lock.1.recv().expect("Can't get l10n info")
    }

    pub fn get(self, key: &'static str, arg_slice: Vec<(&'static str, String)>) -> String
    {
        self.get_raw(
            key,
            arg_slice
                .into_iter()
                .map(|(x, y)| (x, Either::Left(y)))
                .collect(),
        )
    }

    pub fn get_num<T>(self, key: &'static str, arg_slice: Vec<(&'static str, T)>) -> String
    where
        FluentNumber: From<T>,
    {
        self.get_raw(
            key,
            arg_slice
                .into_iter()
                .map(|(x, y)| (x, Either::Right(FluentNumber::from(y))))
                .collect(),
        )
    }

    pub fn get_per(self, key: &'static str, arg_slice: Vec<(&'static str, f64)>) -> String
    {
        let options = FluentNumberOptions {
            style: FluentNumberStyle::Percent,
            ..FluentNumberOptions::default()
        };

        self.get_raw(
            key,
            arg_slice
                .into_iter()
                .map(move |(x, y)| (x, Either::Right(FluentNumber::new(y, options.clone()))))
                .collect(),
        )
    }
}
