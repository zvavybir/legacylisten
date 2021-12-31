use std::{
    mem::take,
    path::{Path, PathBuf},
};

use crate::{csv::Csv, err::Error, l10n::Lang};

#[derive(Clone, Debug)]
pub struct Conffile
{
    pub data_dir: PathBuf,
    pub minimum_ram: u64,
    pub ignore_ram: bool,
    pub lang: Lang,
    pub repeat_bonus: i64,
}

impl Conffile
{
    pub fn default(conffile_dir: &Path) -> Self
    {
        Self {
            data_dir: conffile_dir.join("data"),
            minimum_ram: 1024 * 1024 * 1024,
            ignore_ram: true,
            lang: Lang::English,
            repeat_bonus: 0,
        }
    }

    pub fn new(conffile_dir: &Path) -> Result<Self, Error>
    {
        let mut rv = Self::default(conffile_dir);

        let mut csv = Csv::new(conffile_dir.join("conffile.csv"))?;
        for line in &mut csv.entries
        {
            if line.len() != 2
            {
                continue;
            }

            match line[0].as_str()
            {
                "data_dir" => rv.data_dir = PathBuf::from(take(&mut line[1])),
                "minimum_ram" =>
                {
                    let _ = line[1].parse().map(|x| rv.minimum_ram = x);
                }
                "ignore_ram" =>
                {
                    let _ = line[1].parse().map(|x| rv.ignore_ram = x);
                }
                "lang" => match line[1].as_str()
                {
                    "english" => rv.lang = Lang::English,
                    "german" | "deutsch" => rv.lang = Lang::German,
                    "custom" =>
                    {
                        let path = line[2].parse();
                        let id = line[3].parse();

                        if let (Ok(path), Ok(id)) = (path, id)
                        {
                            rv.lang = Lang::Custom(path, id);
                        }
                    }
                    _ =>
                    {}
                },
                "repeat_bonus" =>
                {
                    let _ = line[1].parse().map(|x| rv.repeat_bonus = x);
                }
                _ =>
                {}
            }
        }

        Ok(rv)
    }
}
