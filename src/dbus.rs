use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::File,
    io::Write,
    iter::FromIterator,
    path::{Path, PathBuf},
    sync::{
        atomic::Ordering,
        mpsc::{Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use dbus::{
    arg::{messageitem::MessageItem, Variant},
    blocking::Connection,
    channel::MatchingReceiver,
    MethodErr,
};
use dbus_crossroads::{Crossroads, IfaceBuilder};
use id3::Tag;

use crate::{
    commands::Command,
    config::ArcConfig,
    helpers::{recv_last, unwrap_two},
    parser::SmallMetadata,
};

fn path_to_dbus_obj(path: &Path) -> String
{
    path.to_str().map_or_else(
        || String::from("/org/mpris/MediaPlayer2/legacylisten/path/error"),
        |s| {
            vec![String::from("/org/mpris/MediaPlayer2/legacylisten/path/ok")]
                .into_iter()
                .chain(s.bytes().map(|x| format!("a{}", x)))
                .collect::<Vec<_>>()
                .join("/")
        },
    )
}

fn to_dbus_time(val: usize, config: &Arc<ArcConfig>) -> i64
{
    ((val * 1000 * 1000) / config.sample_rate.load(Ordering::SeqCst)) as _
}

const fn itemify(x: String) -> Variant<MessageItem>
{
    Variant(MessageItem::Str(x))
}

fn register_interface1(b: &mut IfaceBuilder<()>, tx_clone: Sender<Command>)
{
    // Do nothing on Raise(), because there is nothing to do for a
    // terminal program.
    b.method("Raise", (), (), |_, _, _: ()| Ok(()));
    b.method("Quit", (), (), move |_, _, _: ()| {
        let _ = tx_clone.send(Command::Quit);
        Ok(())
    });

    b.property("CanQuit").get(|_, _| Ok(true));
    b.property("CanSetFullscreen").get(|_, _| Ok(false));
    b.property("CanRaise").get(|_, _| Ok(false));
    b.property("HasTrackList").get(|_, _| Ok(false));
    b.property("Identity")
        .get(|_, _| Ok("legacylisten".to_owned()));
    b.property("SupportedUriSchemes")
        .get(move |_, _| Ok(Vec::<String>::new()));
    b.property("SupportedMimeTypes")
        .get(move |_, _| Ok(Vec::<String>::new()));
}

fn set_metadata(
    tx_control: &Sender<()>,
    rx_path: &Arc<Mutex<ComplexReceiver>>,
    config: &Arc<ArcConfig>,
) -> HashMap<String, Variant<MessageItem>>
{
    let _ = tx_control.send(());
    let (path, tag) = recv_last(&rx_path.lock().unwrap());

    let art_url_raw = config
        .config_dir
        .join("default.png")
        .into_os_string()
        .into_string()
        .unwrap();

    let mut art_url = String::from("file://");

    art_url.push_str(&art_url_raw);

    if let Ok(mut pic_path) = config.pic_path.lock()
    {
        *pic_path = Some(art_url_raw);
    }

    let mut hm = HashMap::<_, _>::from_iter(vec![
        (
            String::from("mpris:trackid"),
            itemify(path_to_dbus_obj(&path)),
        ),
        // Fallback icon
        (String::from("mpris:artUrl"), itemify(art_url)),
    ]);

    let title = tag
        .as_ref()
        .and_then(|tag| tag.artist().map(ToOwned::to_owned));
    let artist = tag
        .as_ref()
        .and_then(|tag| tag.title().map(ToOwned::to_owned));

    let (title, artist) = unwrap_two(title, artist, || {
        SmallMetadata::new(path.file_stem().and_then(OsStr::to_str), config).into_tuple()
    });

    hm.insert(String::from("xesam:artist"), itemify(title));

    hm.insert(String::from("xesam:title"), itemify(artist));

    if let Some(picture) = tag.and_then(|tag| tag.pictures().next().cloned())
    {
        let path = config.config_dir.join("icon.art");
        if let Ok(mut file) = File::create(&path)
        {
            if file.write_all(&picture.data).is_ok()
            {
                let art_url_raw = path.to_str().unwrap();
                let mut art_url = String::from("file://");
                art_url.push_str(art_url_raw);

                if let Ok(mut pic_path) = config.pic_path.lock()
                {
                    *pic_path = Some(art_url_raw.to_string());
                }

                hm.insert(String::from("mpris:artUrl"), itemify(art_url));
            }
        }
    }

    if config.current_len.load(Ordering::SeqCst) != 0
    {
        let key = String::from("mpris:length");
        let value = Variant(MessageItem::Int64(to_dbus_time(
            config.current_len.load(Ordering::SeqCst) / config.channels.load(Ordering::SeqCst),
            config,
        )));

        hm.insert(key, value);
    }

    hm
}

type ComplexReceiver = Receiver<(PathBuf, Option<Tag>)>;

fn register_interface2(
    b: &mut IfaceBuilder<()>,
    tx: Sender<Command>,
    tx_control: Sender<()>,
    rx_paused: Arc<Mutex<Receiver<bool>>>,
    rx_path: Arc<Mutex<ComplexReceiver>>,
    config: Arc<ArcConfig>,
)
{
    let tx_clone = tx.clone();
    b.method("Next", (), (), move |_, _, _: ()| {
        let _ = tx_clone.send(Command::Skip);
        Ok(())
    });
    b.method("Previous", (), (), move |_, _, _: ()| Ok(()));
    let tx_clone = tx.clone();
    b.method("Pause", (), (), move |_, _, _: ()| {
        let _ = tx_clone.send(Command::Pause);
        Ok(())
    });
    let tx_clone = tx.clone();
    b.method("PlayPause", (), (), move |_, _, _: ()| {
        let _ = tx_clone.send(Command::SwitchPlayPause);
        Ok(())
    });
    b.method("Stop", (), (), move |_, _, _: ()| Ok(()));
    let tx_clone = tx;
    b.method("Play", (), (), move |_, _, _: ()| {
        let _ = tx_clone.send(Command::Resume);
        Ok(())
    });

    let tx_control_clone = tx_control.clone();
    let l10n = config.l10n;
    b.property("PlaybackStatus")
        .emits_changed_true()
        .get(move |_, _| {
            tx_control_clone
                .send(())
                .map_err(|_| MethodErr::failed(&l10n.get("controller-out", vec![])))?;
            Ok(
                if recv_last(&rx_paused.lock().unwrap())
                {
                    String::from("Paused")
                }
                else
                {
                    String::from("Playing")
                },
            )
        });
    b.property("Rate").get(|_, _| Ok(1.0_f64));
    let config2 = config.clone();
    b.property("Metadata")
        .emits_changed_true()
        .get(move |_, _| Ok(set_metadata(&tx_control.clone(), &rx_path.clone(), &config)));

    b.property("Volume").get(|_, _| Ok(1.0_f64));
    b.property("MaximumRate").get(|_, _| Ok(1.0_f64));
    b.property("MinimumRate").get(|_, _| Ok(1.0_f64));
    b.property("CanGoNext").get(|_, _| Ok(true));
    b.property("CanGoPrevious").get(|_, _| Ok(false));
    b.property("CanPlay").get(|_, _| Ok(true));
    b.property("CanPause").get(|_, _| Ok(true));
    b.property("CanSeek").get(|_, _| Ok(false));
    b.property("CanControl").get(|_, _| Ok(true));
    b.property("Position").get(move |_, _| {
        Ok(to_dbus_time(
            config2.current_pos.load(Ordering::SeqCst) / config2.channels.load(Ordering::SeqCst),
            &config2,
        ))
    });
}

pub fn handle_mpris(
    tx: &Sender<Command>,
    tx_control: &Sender<()>,
    rx_paused: Receiver<bool>,
    rx_path: Receiver<(PathBuf, Option<Tag>)>,
    config: &Arc<ArcConfig>,
) -> Result<(), dbus::Error>
{
    let rx_paused = Arc::new(Mutex::new(rx_paused));
    let rx_path = Arc::new(Mutex::new(rx_path));

    loop
    {
        thread::sleep(Duration::from_millis(20));

        let tx = tx.clone();
        let tx_control = tx_control.clone();

        let c = Connection::new_session()?;
        c.request_name("org.mpris.MediaPlayer2.legacylisten", false, false, false)?;

        let mut cr = Crossroads::new();

        let tx_clone = tx.clone();
        let interface1 = cr.register("org.mpris.MediaPlayer2", move |b| {
            register_interface1(b, tx_clone);
        });

        let rx_paused = rx_paused.clone();
        let rx_path = rx_path.clone();
        let config2 = config.clone();
        let interface2 = cr.register("org.mpris.MediaPlayer2.Player", move |b| {
            register_interface2(b, tx, tx_control, rx_paused, rx_path, config2);
        });

        cr.insert("/org/mpris/MediaPlayer2", &[interface1, interface2], ());

        c.start_receive(
            dbus::message::MatchRule::new_method_call(),
            // Needed to adhere to dbus API.
            #[allow(box_pointers)]
            Box::new(move |msg, conn| {
                cr.handle_message(msg, conn).unwrap();
                true
            }),
        );

        // TODO: Send real "property changed" signals instead of
        // restarting the whole dbus module.
        while !config.update_dbus.load(Ordering::SeqCst)
        {
            c.process(std::time::Duration::from_millis(10))?;
        }
        config.update_dbus.store(false, Ordering::SeqCst);
    }
}
