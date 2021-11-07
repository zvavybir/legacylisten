use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::Read,
    mem,
    path::Path,
    sync::Arc,
};

use crate::{
    config::ArcConfig,
    err::Error,
    l10n::L10n,
    songs::{L10nHelper, Song, Songs},
};

#[derive(Clone, Debug)]
pub struct Csv
{
    pub entries: Vec<Vec<String>>,
}

impl Display for Csv
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error>
    {
        for line in &self.entries
        {
            for field in line
            {
                for c in field.chars()
                {
                    match c
                    {
                        '\\' => write!(f, "\\\\")?,
                        '\n' => writeln!(f, "\\")?,
                        ',' => write!(f, "\\,")?,
                        _ => write!(f, "{}", c)?,
                    }
                }
                write!(f, ",")?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Csv
{
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error>
    {
        let mut buf = vec![];

        File::open(path)?.read_to_end(&mut buf)?;

        let mut csv = vec![];
        let mut entry = vec![];
        let mut field = vec![];
        let mut quoted = false;

        for c in buf
        {
            if quoted
            {
                field.push(c);
                quoted = false;
            }
            else if c == b'\\'
            {
                quoted = true;
                continue;
            }
            else if c == b','
            {
                entry.push(String::from_utf8(mem::take(&mut field))?);
            }
            else if c == b'\n'
            {
                csv.push(mem::take(&mut entry));
            }
            else
            {
                field.push(c);
            }
        }

        Ok(Self { entries: csv })
    }

    #[must_use]
    pub fn from(songs: &Songs) -> Self
    {
        let x = songs
            .songs
            .iter()
            .map(|song| {
                vec![
                    song.name.clone(),
                    song.num.to_string(),
                    song.loud.to_string(),
                ]
            })
            .collect();

        Self { entries: x }
    }

    #[must_use]
    pub fn get_songs(self, config: Arc<ArcConfig>, l10n: L10n) -> Option<Songs>
    {
        if self.entries.iter().all(|song| song.len() == 3)
        {
            Some(Songs {
                songs: self
                    .entries
                    .into_iter()
                    .map(|mut song| Song {
                        name: mem::take(&mut song[0]),
                        num: song[1].parse().unwrap(),
                        loud: song[2].parse().unwrap(),
                    })
                    .collect(),
                config,
                l10n_helper: L10nHelper::new(l10n),
            })
        }
        else
        {
            None
        }
    }
}
