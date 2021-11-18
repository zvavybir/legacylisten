/*
    legacylisten – A simple CLI audio player with strange features.
    Copyright (C) 2021  Matthias Kaak

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! # legacylisten
//! `legacylisten` is a simple CLI audio player I wrote because no
//! existing one fulfilled my needs.  The main feature is that you can
//! change how often a song is played (`legacylisten` is always on
//! shuffle-all), but there are some other even odder features.
//!
//! ## How it works
//! `legacylisten` creates a list of all songs in
//! `~/.zvavybir/legacylisten/data`[^1] and sub directories together with
//! their associated so-called "playing likelihood"[^2] and volume (the
//! standard values are 10 and 10% respectively).  Then it will choose a
//! song at random with the probability proportional to it's playing
//! likelihood and plays it, unless you request something different.
//!
//! The volume is adjustable on a per-song basis and is saved.  Although
//! simple (ridiculously trivial indeed) to implement, there is no way to
//! change the global volume, since I figured that this is better left to
//! the operating system.  What a audio player can do very good is
//! recognizing which song is playing and acting according to it.  The
//! intended use of that feature is to adjust the volume of very quiet
//! songs once and than the user doesn't have to be bothered ever again.
//!
//! Another quite obscure feature is that you can not only pause/quit
//! immediately, but also only on the end of the current song, but the
//! strangest one is that `legacylisten` will sever all connection to the
//! disk if the *NIX signal `SIGUSR1` arrives and only starts reading
//! again when `SIGUSR2` arrives.  `SIGUSR1` doesn't interrupt an already
//! playing song since songs are buffered[^3].
//!
//! ## Commands
//! Commands are how `legacylisten` is controlled and consist always out
//! of a single character.  Originally these were the first letter of the
//! command name, but since this caused rather strange names (like `f` –
//! "*f*ainter" – to decrease the volume), I settled to just number them
//! through alphabetically.
//!
//! To execute a command, just type it's letter (but remember that
//! terminals are usually line-buffered, meaning that until you press
//! enter `legacylisten` won't see – and react to – your input).
//!
//! The following commands exist:
//!
//! * `?`: Shows a list of all command with a help message (essentially
//!   this very one).[^4]
//! * `a`: Increases playing likelihood of the current song by 1.
//! * `b`: Decreases playing likelihood of the current song by 1.
//! * `c`: Quits `legacylisten` and saves the songs likelihoods and
//!   volumes to `~/.zvavybir/legacylisten/songs.csv`.
//! * `d`: Pauses playing.
//! * `e`: Resumes playing after pausing with `d` or `l` (doesn't
//!   overwrite `SIGUSR1` though).
//! * `f`: Skips song.
//! * `g`: Increases permanently the volume of the current song by 1% (but
//!   not above 100%).
//! * `h`: Decreases permanently the volume of the current song by 1% (but
//!   not below 0%).
//! * `i`: Shows how long the song is already playing and – if
//!   available[^5] – how long it will take in total.
//! * `j`: Switches between playing and pausing.
//! * `k`: Quits `legacylisten` as soon as the current song has finished
//!   playing (takes precedence over `l`).
//! * `l`: Like `k`, but just pauses instead of quitting.
//! * `m`: Shows the metadata in the song's id3 tag (the length usually
//!   has to be queried by `i` since it's rarely saved in the id3 tag).
//! * `n`: Opens the cover image of the song in your preferred image viewer
//!   (it uses `mimeopen` which is AFAIK not available on MS Windows, so
//!   this won't work there).  If the song has no cover it opens
//!   `~/.zvavybir/legacylisten/default.png` (doesn't need to be an PNG
//!   file) instead.  For the fallback image I use (and made, so it's
//!   quite bad) see
//!   [here](https://github.com/zvavybir/legacylisten/blob/master/imgs/default.png).
//! * `o`: Stops all repeating (but if the current songs is an repetition
//!   it's not ended immediately; if you want that also skip with `f`).
//! * `p`: Repeats the current song once.
//! * `q`: Repeats the current song forever.
//! * `r`: Skips to the beginning of the current song or – if it already
//!   is at the beginning – to the previous one.  You can go as many songs
//!   back as you want (or more precisely how many there are).  All played
//!   songs are saved (but only in one run of `legacylisten`, if you
//!   restart it the history is lost) and if you went back the next song
//!   is the same as previously followed on that song.
//!
//! ## Low memory handler
//! As already briefly pointed out previously, especially older versions
//! of `legacylisten` had an horrendous memory footprint, which rendered
//! my system unusable for a few seconds a couple times.  Under Linux such
//! problems are usually handled by the OOM killer (which ends the process
//! with least importance and largest memory consumption), but it turns
//! out that the Chromium web browser (the free software variant of Google
//! Chrome), which out of other reasons I'm forced to use every once in a
//! while, is even worse than my crimes.  Instead of doing something
//! sensible, I added a routine to `legacylisten` that watches the amount
//! of free memory and terminates itself when it falls under some certain
//! (configurable) threshold (a GiB currently).
//!
//! This uses currently a wrong notion of "free ram" (it counts memory
//! used for disk caching as used although it's not; see [this famous
//! site](https://www.linuxatemyram.com/) for more), so it triggers
//! unnecessarily.  Although this is better than the reverse, the low
//! memory handler is off as default because of that.
//!
//! This uses the *NIX function `sysconf(3)`, so it won't work on outdated
//! platforms.
//!
//! ## Configuration file
//! `legacylisten` can be configured by the
//! `~/.zvavybir/legacylisten/conffile.csv` file.  Despite it's file
//! extension it's *not* a real CSV file, just very much inspired by it.
//! If an option can't be parsed it's just silently ignored, so be
//! careful.  Every option has an own line (with mandatory newline at the
//! end, even for the last line and under MS Windows) and every part of it
//! has to be comma-separated (and every line has to end with a comma).
//! As an example, this is my configuration file:
//! ```text
//! data_dir,/media/my_user_name/external_harddrive/legacylisten,
//! ignore_ram,false,
//! lang,german,
//! ```
//! There are currently four possible options:
//! * `data_dir`: If you have your music collection somewhere else (like
//!   me on an external hard drive or in `~/Music`) you can use this
//!   option to change the directory `legacylisten` will search.  The `~/`
//!   notation is not usable in the configuration file, even under *NIX
//!   systems.
//! * `minimum_ram`: The threshold for the [low memory
//!   handler](#low-memory-handler) in bytes.
//! * `ignore_ram`: Disables the low memory handler (possible values are
//!   `true` and `false`).  If this is set (currently the default)
//!   `minimum_ram` is ignored.
//! * `lang`: `legacylisten` supports basic internationalization and this
//!   is the option to activate it.  There are currently three possible
//!   values for this option:
//!   * `english`: Sets the language to English (this is the default).
//!   * `german` or `deutsch`: Sets the language to German.
//!   * `custom`: If you have a translation file, but it's not included in
//!     the official sources (maybe because you're still working on
//!     finishing it, you just want to try something out or you are
//!     forbidden by legal reasons to publish it under `legacylisten`'s
//!     [license](#license)) this option enables you to still use it.
//!     This option requires two further values, the path to the
//!     translation file and the language ID.  As an example, if English
//!     weren't included already you could use such an option to
//!     circumvent that:
//!     ```text
//!     lang,custom,/path/to/file/translation.fl,en-US,
//!     ```
//!     The path has no requirements about filename or file
//!     extension, but the language identifier *has* to be correct.
//!
//! ## Plugin interface
//! In case there is no metadata tag in the song, you can use the plugin
//! interface to tell `legacylisten` the song's title and artist.  Every
//! plugin is a shell script (or executable if you prefer) in the
//! `~/.zvavybir/legacylisten/parser` directory (or sub directories
//! thereof) and gets the song's file name (without new line character) as
//! input (on stdin).  If the file name could be parsed it has to output
//! the song's title and artist (delimited by zero bytes and optionally a
//! trailing zero byte) on stdout.  If no zero bytes are in the output the
//! parsing is treated as having failed and will be ignored.
//!
//! ## Installing
//! The simplest way to install `legacylisten` is with
//! [rustup](https://rustup.rs) and Cargo.  After installing rustup as
//! indicated on it's website, issue to following command to install
//! `legacylisten` itself:
//! ```text
//! cargo install legacylisten
//! ```
//!
//! ## Contributing
//! As every software `legacylisten` too always can be improved.  While
//! I'm trying to get it usable alone, I don't have unlimited time and
//! especially not always the best ideas.  If you can help with that or on
//! some other way (like with a feature request, an additional language or
//! documentation improvements) **please help**.
//!
//! I assume that unless stated otherwise every contribution follows the
//! necessary license.
//!
//! ## License
//! Though unusual for a rust program, `legacylisten` is released under
//! the GNU General Public License version 3 or (at your option) any later
//! version.
//!
//! For more see
//! [LICENSE.md](https://github.com/zvavybir/legacylisten/blob/master/LICENSE.md).
//!
//! [^1]: Although not intended ([even to the
//!     contrary](https://www.fefe.de/nowindows/)) `legacylisten` should
//!     be quite portable (`~/` refers to the user's home directory – in
//!     `legacylisten` even under MS Windows).
//!
//! [^2]: Or "likelihood" for short.
//!
//! [^3]: This is of course quite bad on the memory footprint, but it's
//!     the best I could manage so far (at least it's a whole magnitude
//!     better than the worst implementation I had).  If you have an
//!     better idea, **please** [contribute](#contributing)!
//!
//! [^4]: This command is a bit special since it's handled differently
//!     internally.  You can see this on the one hand directly by it's
//!     special name (only non-letter one) and on the other hand (when you
//!     run `legacylisten`) that while usually commands are executed
//!     strictly in order this one is run before all others specified on
//!     the same line.
//!
//! [^5]: `legacylisten` tries to read it out of the metadata of the audio
//!     file or – if that fails (which happen often, since the underlying
//!     routine seems to be still work-in-progress) – decodes the whole
//!     song a second time to get the length on a simple, but costly way
//!     after a short waiting period.  Until that is fixed (if you can
//!     help, **please** [contribute](#contributing)) I wouldn't recommend
//!     skipping multiple songs in short sequence, since per song there's
//!     one thread trying to decode it – even after it's already certain
//!     that it's never needed.

#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo_common_metadata
)]
// Anachronism
#![allow(clippy::non_ascii_literal)]
// More or less manual checked and documentation agrees with me that
// it's usually not needed.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_lossless
)]
// Explicitly decided against; I think `let _ = …` is better than
// `mem::drop(…)`. TODO: align my opinion and community's one with
// each other.
#![allow(clippy::let_underscore_drop)]

mod audio;
mod buffer;
mod commands;
mod conffile;
mod config;
mod csv;
mod dbus;
mod err;
mod files;
mod helpers;
mod l10n;
mod matcher;
mod parser;
mod songs;
mod threads;

pub mod runner;

pub use err::Error;
