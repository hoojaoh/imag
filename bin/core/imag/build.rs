//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2019 Matthias Beyer <mail@beyermatthias.de> and contributors
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; version
// 2.1 of the License.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//

extern crate clap;
extern crate libimagrt;
extern crate libimagentrytag;
extern crate libimagutil;

use clap::Shell;
use libimagrt::runtime::Runtime;

#[allow(unused_imports)]
use libimagrt::application::ImagApplication;

#[cfg(feature = "cc-imag-annotate")]
extern crate libimagannotatecmd;
#[cfg(feature = "cc-imag-diagnostics")]
extern crate libimagdiagnosticscmd;
#[cfg(feature = "cc-imag-edit")]
extern crate libimageditcmd;
#[cfg(feature = "cc-imag-gps")]
extern crate libimaggpscmd;
#[cfg(feature = "cc-imag-grep")]
extern crate libimaggrepcmd;
#[cfg(feature = "cc-imag-ids")]
extern crate libimagidscmd;
#[cfg(feature = "cc-imag-init")]
extern crate libimaginitcmd;
#[cfg(feature = "cc-imag-link")]
extern crate libimaglinkcmd;
#[cfg(feature = "cc-imag-mv")]
extern crate libimagmvcmd;
#[cfg(feature = "cc-imag-ref")]
extern crate libimagrefcmd;
#[cfg(feature = "cc-imag-store")]
extern crate libimagstorecmd;
#[cfg(feature = "cc-imag-tag")]
extern crate libimagtagcmd;
#[cfg(feature = "cc-imag-view")]
extern crate libimagviewcmd;
#[cfg(feature = "cc-imag-bookmark")]
extern crate libimagbookmarkfrontend;
#[cfg(feature = "cc-imag-contact")]
extern crate libimagcontactfrontend;
#[cfg(feature = "cc-imag-diary")]
extern crate libimagdiaryfrontend;
#[cfg(feature = "cc-imag-habit")]
extern crate libimaghabitfrontend;
#[cfg(feature = "cc-imag-log")]
extern crate libimaglogfrontend;
#[cfg(feature = "cc-imag-mail")]
extern crate libimagmailfrontend;
#[cfg(feature = "cc-imag-notes")]
extern crate libimagnotesfrontend;
#[cfg(feature = "cc-imag-timetrack")]
extern crate libimagtimetrackfrontend;
#[cfg(feature = "cc-imag-todo")]
extern crate libimagtodofrontend;

/// This macro reduces boilerplate code.
///
/// For example: `build_subcommand!("counter", libbinimagcounter, ImagCounter)`
/// will result in the following code:
/// ```ignore
/// ImagCounter::build_cli(Runtime::get_default_cli_builder(
///     "counter",
///     "abc",
///     "counter"))
/// ```
/// As for the `"abc"` part, it does not matter
/// which version the subcommand is getting here, as the
/// output of this script is a completion script, which
/// does not contain information about the version at all.
#[allow(unused_macros)]
macro_rules! build_subcommand {
    ($name:expr, $lib:ident, $implementor:ident) => (
        $lib::$implementor::build_cli(Runtime::get_default_cli_builder($name, "abc", $name))
    )
}

fn main() {
    // Make the `imag`-App...
    let app = Runtime::get_default_cli_builder(
        "imag",
        "abc",
        "imag");

    // and add all the subapps as subcommands.
    // TODO: This feels tedious, can we automate this?
    #[cfg(feature = "cc-imag-annotate")]
    let app = app.subcommand(build_subcommand!("annotate",    libimagannotatecmd, ImagAnnotate));
    #[cfg(feature = "cc-imag-diagnostics")]
    let app = app.subcommand(build_subcommand!("diagnostics", libimagdiagnosticscmd, ImagDiagnostics));
    #[cfg(feature = "cc-imag-edit")]
    let app = app.subcommand(build_subcommand!("edit",        libimageditcmd, ImagEdit));
    #[cfg(feature = "cc-imag-gps")]
    let app = app.subcommand(build_subcommand!("gps",         libimaggpscmd, ImagGps));
    #[cfg(feature = "cc-imag-grep")]
    let app = app.subcommand(build_subcommand!("grep",        libimaggrepcmd, ImagGrep));
    #[cfg(feature = "cc-imag-ids")]
    let app = app.subcommand(build_subcommand!("ids",         libimagidscmd, ImagIds));
    #[cfg(feature = "cc-imag-init")]
    let app = app.subcommand(build_subcommand!("init",        libimaginitcmd, ImagInit));
    #[cfg(feature = "cc-imag-link")]
    let app = app.subcommand(build_subcommand!("link",        libimaglinkcmd, ImagLink));
    #[cfg(feature = "cc-imag-mv")]
    let app = app.subcommand(build_subcommand!("mv",          libimagmvcmd, ImagMv));
    #[cfg(feature = "cc-imag-ref")]
    let app = app.subcommand(build_subcommand!("ref",         libimagrefcmd, ImagRef));
    #[cfg(feature = "cc-imag-store")]
    let app = app.subcommand(build_subcommand!("store",       libimagstorecmd, ImagStore));
    #[cfg(feature = "cc-imag-tag")]
    let app = app.subcommand(build_subcommand!("tag",         libimagtagcmd, ImagTag));
    #[cfg(feature = "cc-imag-view")]
    let app = app.subcommand(build_subcommand!("view",         libimagviewcmd, ImagView));
    #[cfg(feature = "cc-imag-bookmark")]
    let app = app.subcommand(build_subcommand!("bookmark",    libimagbookmarkfrontend, ImagBookmark));
    #[cfg(feature = "cc-imag-contact")]
    let app = app.subcommand(build_subcommand!("contact",     libimagcontactfrontend, ImagContact));
    #[cfg(feature = "cc-imag-diary")]
    let app = app.subcommand(build_subcommand!("diary",       libimagdiaryfrontend, ImagDiary));
    #[cfg(feature = "cc-imag-habit")]
    let app = app.subcommand(build_subcommand!("habit",       libimaghabitfrontend, ImagHabit));
    #[cfg(feature = "cc-imag-log")]
    let app = app.subcommand(build_subcommand!("log",         libimaglogfrontend, ImagLog));
    #[cfg(feature = "cc-imag-mail")]
    let app = app.subcommand(build_subcommand!("mail",        libimagmailfrontend, ImagMail));
    #[cfg(feature = "cc-imag-notes")]
    let app = app.subcommand(build_subcommand!("notes",       libimagnotesfrontend, ImagNotes));
    #[cfg(feature = "cc-imag-timetrack")]
    let app = app.subcommand(build_subcommand!("timetrack",   libimagtimetrackfrontend, ImagTimetrack));
    #[cfg(feature = "cc-imag-todo")]
    let app = app.subcommand(build_subcommand!("todo",        libimagtodofrontend, ImagTodo));

    let mut app = app;
    // Actually generates the completion files
    app.gen_completions("imag", Shell::Bash, "../../../target/");
    app.gen_completions("imag", Shell::Fish, "../../../target/");
    app.gen_completions("imag", Shell::Zsh,  "../../../target/");
}
