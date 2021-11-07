use std::{
    io::Write,
    process::{Command, Stdio},
    sync::Arc,
};

use walkdir::WalkDir;

use crate::config::ArcConfig;

#[derive(Clone, Debug)]
pub struct SmallMetadata
{
    pub name: String,
    pub artist: String,
}

impl SmallMetadata
{
    fn from(s: &str, config: &Arc<ArcConfig>) -> Option<Self>
    {
        for entry in WalkDir::new(config.config_dir.join("parser"))
        {
            let mut parser = if let Ok(parser) = Command::new(entry.ok()?.path().as_os_str())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
            {
                parser
            }
            else
            {
                continue;
            };

            let mut stdin = parser.stdin.take()?;

            write!(stdin, "{}", s).ok()?;

            let output = parser.wait_with_output().ok()?.stdout;
            let output_split = output.split(|&c| c == b'\0').collect::<Vec<_>>();

            if output_split.len() > 2
            {
                return Some(Self {
                    name: String::from_utf8_lossy(output_split[0]).to_string(),
                    artist: String::from_utf8_lossy(output_split[1]).to_string(),
                });
            }
        }

        None
    }

    pub fn new(s: Option<&str>, config: &Arc<ArcConfig>) -> Self
    {
        s.and_then(|s| Self::from(s, config))
            .unwrap_or_else(|| Self {
                name: config.l10n.get("unknown-title", vec![]),
                artist: config.l10n.get("unknown-artist", vec![]),
            })
    }

    // This nursery lint is known to have some false-positives around
    // destructors.
    #[allow(clippy::missing_const_for_fn)]
    pub fn into_tuple(self) -> (String, String)
    {
        let Self { name, artist } = self;

        (name, artist)
    }
}
