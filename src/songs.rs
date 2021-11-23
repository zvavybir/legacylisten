use std::{fs::File, io::Write, sync::Arc};

use rand::random;
use walkdir::WalkDir;

use crate::{
    config::{ArcConfig, Config},
    csv::Csv,
    err::Error,
    l10n::{messages::Message, L10n},
    matcher::BigAction,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Repeat
{
    Not,
    Once,
    Always,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Song
{
    pub name: String,
    pub num: u32,
    pub loud: f32,
}

#[derive(Clone)]
pub struct L10nHelper
{
    l10n: L10n,
    // I don't want to do an failable action in such a high
    // sensitivity area; probably stupid
    err_msg: String,
}

#[derive(Clone)]
pub struct Songs
{
    pub songs: Vec<Song>,
    pub config: Arc<ArcConfig>,
    pub l10n_helper: L10nHelper,
}

impl L10nHelper
{
    pub fn new(l10n: L10n) -> Self
    {
        Self {
            l10n,
            err_msg: l10n.get(Message::SavingStateErr),
        }
    }
}

impl Drop for Songs
{
    fn drop(&mut self)
    {
        let s = format!("{}", Csv::from(self));

        match Self::save(s.as_bytes(), &self.config)
        {
            Ok(()) =>
            {
                self.l10n_helper.l10n.write(Message::StateSaved);
            }
            Err(e) =>
            {
                eprintln!("{}: {:?}", self.l10n_helper.err_msg, e);
                eprintln!();
                eprintln!("{}", s);
            }
        }
    }
}

impl Songs
{
    #[must_use]
    pub fn total_likelihood(&self) -> u32
    {
        self.songs.iter().map(|x| x.num).sum()
    }

    fn save(s: &[u8], config: &Arc<ArcConfig>) -> Result<(), Error>
    {
        let mut config_file = File::create(config.config_dir.join("songs.csv"))?;

        config_file.write_all(s)?;

        Ok(())
    }

    pub fn read(config: Arc<ArcConfig>, l10n: L10n) -> Result<Self, Error>
    {
        let mut songs = Csv::new(config.config_dir.join("songs.csv"))?
            .get_songs(config, l10n)
            .ok_or(Error::MalformattedSongsCsv)?;

        for file in config_dir_handle(&songs.config)?
        {
            let file = file?;
            if file.file_type().is_dir()
            {
                // Don't want directories, since they can't be played.
                continue;
            }

            let filename = file
                .path()
                .strip_prefix(&songs.config.conffile.data_dir)?
                .to_string_lossy()
                .into_owned();

            if !songs.songs.iter().any(|x| x.name == filename)
            {
                l10n.write(Message::NewSongFound(filename.clone()));

                songs.songs.push(Song {
                    name: filename,
                    num: 10,
                    loud: 0.1,
                });
            }
        }

        Ok(songs)
    }

    pub fn choose_random<F>(&mut self, config: &mut Config, mut f: F, l10n: L10n) -> BigAction
    where
        F: FnMut(&mut Song, &mut Config) -> BigAction,
    {
        let total = self.total_likelihood();

        l10n.write(Message::TotalPlayingLikelihood(total));

        if total == 0
        {
            l10n.write(Message::NoSongs);
            return BigAction::Quit;
        }

        if config.song_index != 0
        {
            let song = &mut self.songs[config.songlist[config.song_index - 1]];

            if config.repeat == Repeat::Once
            {
                config.repeat = Repeat::Not;
                return f(song, config);
            };
            if config.repeat == Repeat::Always
            {
                return f(song, config);
            }
            if config.song_index != config.songlist.len()
            {
                config.song_index += 1;
                return f(song, config);
            }
        }

        let mut song_number = (random::<u64>() % total as u64) as _;

        for (pos, song) in self.songs.iter_mut().enumerate()
        {
            if song.num >= song_number
            {
                config.songlist.push(pos);
                config.song_index += 1;
                return f(song, config);
            }
            song_number -= song.num;
        }

        unreachable!();
    }
}

fn config_dir_handle(config: &Arc<ArcConfig>) -> Result<WalkDir, Error>
{
    let filename = &config.conffile.data_dir;
    let dir = File::open(filename).or_else(|_| {
        std::fs::create_dir_all(filename)?;
        File::open(filename)
    })?;
    if !dir.metadata()?.is_dir()
    {
        trash::delete(filename)?;
        std::fs::create_dir_all(filename)?;
    }

    Ok(WalkDir::new(filename))
}
