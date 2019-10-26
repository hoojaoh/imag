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
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
extern crate toml_query;
extern crate resiter;

extern crate libimagentryannotation;
extern crate libimagentryedit;
extern crate libimagerror;
extern crate libimagrt;
extern crate libimagstore;
extern crate libimagutil;
extern crate libimagentrylink;

use std::io::Write;

use failure::Error;
use failure::Fallible as Result;
use failure::ResultExt;
use failure::err_msg;
use resiter::IterInnerOkOrElse;
use resiter::AndThen;
use resiter::Map;
use toml_query::read::TomlValueReadTypeExt;
use clap::App;

use libimagentryannotation::annotateable::*;
use libimagentryannotation::annotation_fetcher::*;
use libimagentryedit::edit::*;
use libimagerror::errors::ErrorMsg as EM;
use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
use libimagstore::store::FileLockEntry;
use libimagstore::iter::get::StoreIdGetIteratorExtension;
use libimagentrylink::linkable::Linkable;
use libimagrt::iter::ReportTouchedResultEntry;

mod ui;

pub enum ImagAnnotate {}
impl ImagApplication for ImagAnnotate {
    fn run(rt: Runtime) -> Result<()> {
        match rt.cli().subcommand_name().ok_or_else(|| err_msg("No command called"))? {
            "add"    => add(&rt),
            "remove" => remove(&rt),
            "list"   => list(&rt),
            other    => {
                debug!("Unknown command");
                if rt.handle_unknown_subcommand("imag-annotation", other, rt.cli())?.success() {
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
        "Add annotations to entries"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

fn add(rt: &Runtime) -> Result<()> {
    let scmd = rt.cli().subcommand_matches("add").unwrap(); // safed by main()
    let mut ids = rt
        .ids::<crate::ui::PathProvider>()
        .context("No StoreId supplied")?
        .ok_or_else(|| err_msg("No ids supplied"))?
        .into_iter();

    if let Some(first) = ids.next() {
        let mut annotation = rt.store()
            .get(first.clone())?
            .ok_or_else(|| EM::EntryNotFound(first.local_display_string()))?
            .annotate(rt.store())?;

        annotation.edit_content(&rt)?;

        rt.report_touched(&first)?; // report first one first
        ids.map(Ok).into_get_iter(rt.store())
            .map_inner_ok_or_else(|| err_msg("Did not find one entry"))
            .and_then_ok(|mut entry| entry.add_link(&mut annotation).map(|_| entry))
            .map_report_touched(&rt)
            .map_ok(|_| ())
            .collect::<Result<Vec<_>>>()?;

        if !scmd.is_present("dont-print-name") {
            if let Some(annotation_id) = annotation
                .get_header()
                .read_string("annotation.name")?
            {
                writeln!(rt.stdout(), "Name of the annotation: {}", annotation_id)?;
            } else {
                Err(format_err!("Unnamed annotation: {:?}", annotation.get_location()))
                    .context("This is most likely a BUG, please report!")?;
            }
        }

        rt.report_touched(annotation.get_location())?;
    } else {
        debug!("No entries to annotate");
    }

    Ok(())
}

fn remove(rt: &Runtime) -> Result<()> {
    let scmd            = rt.cli().subcommand_matches("remove").unwrap(); // safed by main()
    let annotation_name = scmd.value_of("annotation_name").unwrap(); // safed by clap
    let delete          = scmd.is_present("delete-annotation");

    rt.ids::<crate::ui::PathProvider>()
        .context("No ids supplied")?
        .ok_or_else(|| err_msg("No ids supplied"))?
        .into_iter()
        .map(|id| {
            let mut entry = rt.store()
                .get(id.clone())?
                .ok_or_else(|| EM::EntryNotFound(id.local_display_string()))?;

            let annotation = entry.denotate(rt.store(), annotation_name)?;

            if delete {
                debug!("Deleting annotation object");
                if let Some(an) = annotation {
                    let loc = an.get_location().clone();
                    drop(an);

                    rt.store().delete(loc)?;
                } else {
                    warn!("Not having annotation object, cannot delete!");
                }
            } else {
                debug!("Not deleting annotation object");
            }

            rt.report_touched(entry.get_location()).map_err(Error::from)
        })
        .collect()
}

fn list(rt: &Runtime) -> Result<()> {
    let scmd      = rt.cli().subcommand_matches("list").unwrap(); // safed by clap
    let with_text = scmd.is_present("list-with-text");
    let ids = rt
        .ids::<crate::ui::PathProvider>()
        .context("No ids supplied")?
        .ok_or_else(|| err_msg("No ids supplied"))?;

    if ids.len() != 0 {
        ids.into_iter()
            .map(|id| -> Result<_> {
                let lds = id.local_display_string();
                Ok(rt.store()
                    .get(id)?
                    .ok_or_else(|| EM::EntryNotFound(lds))?
                    .annotations()?
                    .into_get_iter(rt.store())
                    .map(|el| el.and_then(|o| o.ok_or_else(|| format_err!("Cannot find entry"))))
                    .enumerate()
                    .map(|(i, entry)| entry.and_then(|e| list_annotation(&rt, i, &e, with_text).map(|_| e)))
                    .map_report_touched(&rt)
                    .map_ok(|_| ())
                    .collect())
            })
            .flatten()
            .collect()
    } else { // ids.len() == 0
        // show them all
        rt.store()
            .all_annotations()?
            .into_get_iter()
            .map(|el| el.and_then(|opt| opt.ok_or_else(|| format_err!("Cannot find entry"))))
            .enumerate()
            .map(|(i, entry)| entry.and_then(|e| list_annotation(&rt, i, &e, with_text).map(|_| e)))
            .map_report_touched(&rt)
            .map_ok(|_| ())
            .collect()
    }
}

fn list_annotation<'a>(rt: &Runtime, i: usize, a: &FileLockEntry<'a>, with_text: bool) -> Result<()> {
    if with_text {
        writeln!(rt.stdout(),
                 "--- {i: >5} | {id}\n{text}\n\n",
                 i = i,
                 id = a.get_location(),
                 text = a.get_content())
    } else {
        writeln!(rt.stdout(), "{: >5} | {}", i, a.get_location())
    }.map_err(Error::from)
}

