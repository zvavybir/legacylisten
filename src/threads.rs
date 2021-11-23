use std::{
    convert::TryFrom,
    io::{self, Read},
    path::PathBuf,
    sync::{
        mpsc::{Receiver, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

use id3::Tag;
use nix::sys::sysinfo::sysinfo;
use signal_hook::consts::{SIGINT, SIGTERM, SIGUSR1, SIGUSR2};
use signal_hook::iterator::Signals;

use crate::{
    commands::Command,
    config::ArcConfig,
    dbus::handle_mpris,
    l10n::{messages::Message, L10n},
};

fn input_handler(tx: &Sender<Command>, l10n: L10n)
{
    let mut buf = [0];

    l10n.write(Message::HelpNotice);

    while let Ok(num) = io::stdin().lock().read(&mut buf)
    {
        if num == 1
        {
            if buf[0] == b'?'
            {
                Command::show_help(l10n);
            }
            else if buf[0] >= b'a' && buf[0] <= b'z'
            {
                if let Ok(com) = Command::try_from(buf[0] - b'a')
                {
                    let _ = tx.send(com);
                }
                else
                {
                    l10n.write(Message::UnknownCommandChar(buf[0] as char));
                }
            }
            else if buf[0] != b'\n'
            {
                l10n.write(Message::UnknownCommandByte(buf[0]));
            }
        }
    }
}

fn signal_handler(tx: &Sender<Command>, mut signals: Signals, config: &Arc<ArcConfig>, l10n: L10n)
{
    for sig in signals.forever()
    {
        l10n.write(Message::InSignalHandler(sig));

        match sig
        {
            SIGINT | SIGTERM => tx.send(Command::Quit).unwrap(),
            SIGUSR1 => config
                .reading_paused
                .store(true, std::sync::atomic::Ordering::SeqCst),
            SIGUSR2 => config
                .reading_paused
                .store(false, std::sync::atomic::Ordering::SeqCst),
            _ => l10n.write(Message::SignalHandlerUnreachable),
        }
    }
}

fn mpris_handler(
    tx: &Sender<Command>,
    tx_control: &Sender<()>,
    rx_paused: Receiver<bool>,
    rx_path: Receiver<(PathBuf, Option<Tag>)>,
    config: &Arc<ArcConfig>,
    l10n: L10n,
)
{
    if let Err(e) = handle_mpris(tx, tx_control, rx_paused, rx_path, config)
    {
        l10n.write(Message::MprisHandlerError(e));
    }

    let _ = tx;
    let _ = tx_control;
}

fn low_memory_handler(tx: &Sender<Command>, config: &Arc<ArcConfig>, l10n: L10n)
{
    // Getting it early, so that it doesn't make problems later;
    // probably stupid.
    let s = l10n.get(Message::MemoryTight);

    loop
    {
        let not_enough_memory = sysinfo().map_or(true, |sysinfo| {
            sysinfo.ram_unused() < config.conffile.minimum_ram
        });
        if not_enough_memory && !config.conffile.ignore_ram
        {
            let _ = tx.send(Command::Quit);
            eprintln!("{}", s);
            thread::sleep(Duration::from_secs(10));
        }
        thread::sleep(Duration::from_millis(10));
    }
}

// TODO: Fix
#[allow(clippy::module_name_repetitions)]
pub fn start_threads(
    tx: Sender<Command>,
    tx_control: Sender<()>,
    rx_paused: Receiver<bool>,
    rx_path: Receiver<(PathBuf, Option<Tag>)>,
    signals: Signals,
    config: Arc<ArcConfig>,
    l10n: L10n,
)
{
    let tx1 = tx.clone();
    let tx2 = tx.clone();
    let tx3 = tx.clone();
    let tx4 = tx;
    let config1 = config.clone();
    let config2 = config.clone();
    let config3 = config;

    let _ = thread::spawn(move || input_handler(&tx1, l10n));
    let _ = thread::spawn(move || signal_handler(&tx2, signals, &config1, l10n));
    let _ =
        thread::spawn(move || mpris_handler(&tx3, &tx_control, rx_paused, rx_path, &config2, l10n));
    let _ = thread::spawn(move || low_memory_handler(&tx4, &config3, l10n));
}
