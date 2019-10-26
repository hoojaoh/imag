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

#![forbid(unsafe_code)]

#![deny(
    non_camel_case_types,
    non_snake_case,
    path_statements,
    trivial_numeric_casts,
    unstable_features,
    unused_allocation,
    unused_import_braces,
    unused_imports,
    unused_must_use,
    unused_mut,
    unused_qualifications,
    while_true,
)]

extern crate clap;
#[macro_use] extern crate log;
extern crate itertools;
extern crate failure;

extern crate libimagnotes;
extern crate libimagrt;
extern crate libimagentryedit;
extern crate libimagerror;
extern crate libimagutil;
extern crate libimagstore;

use std::io::Write;
use std::process::exit;

use itertools::Itertools;
use clap::App;
use failure::Fallible as Result;

use libimagentryedit::edit::Edit;
use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
use libimagstore::iter::get::StoreIdGetIteratorExtension;
use libimagnotes::note::Note;
use libimagnotes::notestore::*;
use libimagerror::trace::MapErrTrace;
use libimagerror::exit::ExitUnwrap;
use libimagerror::io::ToExitCode;
use libimagerror::iter::TraceIterator;
use libimagutil::info_result::*;
use libimagutil::warn_result::WarnResult;


mod ui;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagNotes {}
impl ImagApplication for ImagNotes {
    fn run(rt: Runtime) -> Result<()> {
        if let Some(name) = rt.cli().subcommand_name() {

            debug!("Call: {}", name);
            match name {
                "create" => create(&rt),
                "delete" => delete(&rt),
                "edit"   => edit(&rt),
                "list"   => list(&rt),
                other    => {
                    debug!("Unknown command");
                    let _ = rt.handle_unknown_subcommand("imag-notes", other, rt.cli())
                        .map_err_trace_exit_unwrap()
                        .code()
                        .map(::std::process::exit);
                },
            };
        }
        
        Ok(())
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Note taking helper"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

fn name_from_cli(rt: &Runtime, subcmd: &str) -> String {
    rt.cli().subcommand_matches(subcmd).unwrap().value_of("name").map(String::from).unwrap()
}

fn create(rt: &Runtime) {
    let name = name_from_cli(rt, "create");
    let mut note = rt
        .store()
        .new_note(name.clone(), String::new())
        .map_err_trace_exit_unwrap();

    if rt.cli().subcommand_matches("create").unwrap().is_present("edit") {
        note
            .edit_content(rt)
            .map_warn_err_str("Editing failed")
            .map_err_trace_exit_unwrap();
    }

    rt.report_touched(note.get_location()).unwrap_or_exit();
}

fn delete(rt: &Runtime) {
    rt.store()
        .delete_note(name_from_cli(rt, "delete"))
        .map_info_str("Ok")
        .map_err_trace_exit_unwrap();
}

fn edit(rt: &Runtime) {
    let name = name_from_cli(rt, "edit");
    rt
        .store()
        .get_note(name.clone())
        .map_err_trace_exit_unwrap()
        .map(|mut note| {
            note
                .edit_content(rt)
                .map_warn_err_str("Editing failed")
                .map_err_trace_exit_unwrap();

            rt.report_touched(note.get_location()).unwrap_or_exit();
        })
        .unwrap_or_else(|| {
            error!("Cannot find note with name '{}'", name);
        });
}

fn list(rt: &Runtime) {
    use std::cmp::Ordering;

    rt
        .store()
        .all_notes()
        .map_err_trace_exit_unwrap()
        .into_get_iter(rt.store())
        .trace_unwrap_exit()
        .map(|opt| opt.unwrap_or_else(|| {
            error!("Fatal: Nonexistent entry where entry should exist");
            exit(1)
        }))
        .sorted_by(|note_a, note_b| if let (Ok(a), Ok(b)) = (note_a.get_name(), note_b.get_name()) {
            a.cmp(&b)
        } else {
            Ordering::Greater
        })
        .for_each(|note| {
            let name = note.get_name().map_err_trace_exit_unwrap();
            writeln!(rt.stdout(), "{}", name)
                .to_exit_code()
                .unwrap_or_exit();

            rt.report_touched(note.get_location()).unwrap_or_exit();
        });
}

