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
#[macro_use] extern crate failure;
extern crate resiter;

extern crate libimagnotes;
extern crate libimagrt;
extern crate libimagentryedit;
extern crate libimagerror;
extern crate libimagutil;
extern crate libimagstore;

use std::io::Write;

use itertools::Itertools;
use clap::App;
use failure::Error;
use failure::err_msg;
use failure::Fallible as Result;
use resiter::IterInnerOkOrElse;

use libimagentryedit::edit::Edit;
use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
use libimagstore::iter::get::StoreIdGetIteratorExtension;
use libimagnotes::note::Note;
use libimagnotes::notestore::*;
use libimagutil::warn_result::WarnResult;


mod ui;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagNotes {}
impl ImagApplication for ImagNotes {
    fn run(rt: Runtime) -> Result<()> {
        match rt.cli().subcommand_name().ok_or_else(|| err_msg("No subcommand called"))? {
            "create" => create(&rt),
            "delete" => delete(&rt),
            "edit"  => edit(&rt),
            "list"  => list(&rt),
            other    => {
                debug!("Unknown command");
                if rt.handle_unknown_subcommand("imag-notes", other, rt.cli())?.success() {
                    Ok(())
                } else {
                    Err(err_msg("Failed to handle unknown subcommand"))
                }
            },
        }
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

fn create(rt: &Runtime) -> Result<()> {
    let name = name_from_cli(rt, "create");
    let mut note = rt.store().new_note(name.clone(), String::new())?;

    if rt.cli().subcommand_matches("create").unwrap().is_present("edit") {
        note.edit_content(rt)?
    }

    rt.report_touched(note.get_location()).map_err(Error::from)
}

fn delete(rt: &Runtime) -> Result<()> {
    rt.store().delete_note(name_from_cli(rt, "delete")).map(|_| ())
}

fn edit(rt: &Runtime) -> Result<()> {
    let name = name_from_cli(rt, "edit");
    rt
        .store()
        .get_note(name.clone())?
        .ok_or_else(|| format_err!("Name '{}' not found", name))
        .and_then(|mut note| {
            note.edit_content(rt).map_warn_err_str("Editing failed")?;
            rt.report_touched(note.get_location()).map_err(Error::from)
        })
}

fn list(rt: &Runtime) -> Result<()> {
    use std::cmp::Ordering;

    rt
        .store()
        .all_notes()?
        .into_get_iter(rt.store())
        .map_inner_ok_or_else(|| err_msg("Did not find one entry"))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .sorted_by(|a, b| match (a.get_name(), b.get_name()) {
            (Ok(a), Ok(b)) => a.cmp(&b),
            _ => Ordering::Greater,
        })
        .map(|note| {
            let name = note.get_name().map_err(Error::from)?;
            writeln!(rt.stdout(), "{}", name)?;
            rt.report_touched(note.get_location()).map_err(Error::from)
        })
        .collect::<Result<Vec<_>>>()
        .map(|_| ())
}

