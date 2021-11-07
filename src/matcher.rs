use std::sync::mpsc;

use log::error;

use crate::config::Config;

#[derive(Copy, Clone, Debug)]
pub enum BigAction
{
    Quit,
    Skip,
    Nothing,
}

pub fn main_match(config: &mut Config) -> BigAction
{
    match config.rx.try_recv()
    {
        Ok(com) => com.get_handler()(config),
        Err(mpsc::TryRecvError::Disconnected) =>
        {
            error!("{}", config.l10n.get("command-reading-problem", vec![]));
            BigAction::Quit
        }
        Err(mpsc::TryRecvError::Empty) => BigAction::Nothing,
    }
}
