use std::{
    fs::{create_dir_all, OpenOptions},
    path::Path,
};

use crate::Error;

pub fn ensure_file_existence(home: &Path) -> Result<(), Error>
{
    let base = home.join(".zvavybir/legacylisten");
    create_dir_all(&base)?;
    OpenOptions::new()
        .write(true)
        .create(true)
        .open(base.join("conffile.csv"))?;
    OpenOptions::new()
        .write(true)
        .create(true)
        .open(base.join("songs.csv"))?;

    Ok(())
}
