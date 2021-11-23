use std::sync::mpsc;

use crate::{config::Config, l10n::messages::Message};

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
            config.l10n.write(Message::CommandReadingProblem);
            BigAction::Quit
        }
        Err(mpsc::TryRecvError::Empty) => BigAction::Nothing,
    }
}
