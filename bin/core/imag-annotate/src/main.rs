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

extern crate libimagentryannotation;
extern crate libimagentryedit;
extern crate libimagerror;
#[macro_use] extern crate libimagrt;
extern crate libimagstore;
extern crate libimagutil;
extern crate libimagentrylink;

use std::io::Write;

use failure::Error;
use toml_query::read::TomlValueReadTypeExt;

use libimagentryannotation::annotateable::*;
use libimagentryannotation::annotation_fetcher::*;
use libimagentryedit::edit::*;
use libimagerror::trace::MapErrTrace;
use libimagerror::exit::ExitUnwrap;
use libimagerror::io::ToExitCode;
use libimagerror::errors::ErrorMsg as EM;
use libimagerror::iter::TraceIterator;
use libimagrt::runtime::Runtime;
use libimagrt::setup::generate_runtime_setup;
use libimagstore::store::FileLockEntry;
use libimagstore::iter::get::StoreIdGetIteratorExtension;
use libimagentrylink::linkable::Linkable;

mod ui;

fn main() {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-annotation",
                                    &version,
                                    "Add annotations to entries",
                                    ui::build_ui);

    if let Some(name) = rt.cli().subcommand_name() {
        match name {
            "add"    => add(&rt),
            "remove" => remove(&rt),
            "list"   => list(&rt),
            other    => {
                debug!("Unknown command");
                let _ = rt.handle_unknown_subcommand("imag-annotation", other, rt.cli())
                    .map_err_trace_exit_unwrap()
                    .code()
                    .map(::std::process::exit);
            },
        }
    }
}

fn add(rt: &Runtime) {
    let scmd = rt.cli().subcommand_matches("add").unwrap(); // safed by main()
    let mut ids = rt
        .ids::<crate::ui::PathProvider>()
        .map_err_trace_exit_unwrap()
        .unwrap_or_else(|| {
            error!("No StoreId supplied");
            ::std::process::exit(1);
        })
        .into_iter();

    if let Some(first) = ids.next() {
        let mut annotation = rt.store()
            .get(first.clone())
            .map_err_trace_exit_unwrap()
            .ok_or_else(|| EM::EntryNotFound(first.local_display_string()))
            .map_err(Error::from)
            .map_err_trace_exit_unwrap()
            .annotate(rt.store())
            .map_err_trace_exit_unwrap();

        annotation.edit_content(&rt).map_err_trace_exit_unwrap();

        for id in ids {
            let mut entry = rt.store().get(id.clone())
                .map_err_trace_exit_unwrap()
                .ok_or_else(|| format_err!("Not found: {}", id.local_display_string()))
                .map_err_trace_exit_unwrap();

            entry.add_link(&mut annotation).map_err_trace_exit_unwrap();
        }

        if !scmd.is_present("dont-print-name") {
            if let Some(annotation_id) = annotation
                .get_header()
                .read_string("annotation.name")
                .map_err(Error::from)
                .map_err_trace_exit_unwrap()
            {
                writeln!(rt.stdout(), "Name of the annotation: {}", annotation_id)
                    .to_exit_code()
                    .unwrap_or_exit();
            } else {
                error!("Unnamed annotation: {:?}", annotation.get_location());
                error!("This is most likely a BUG, please report!");
            }
        }
    } else {
        debug!("No entries to annotate");
    }
}

fn remove(rt: &Runtime) {
    let scmd = rt.cli().subcommand_matches("remove").unwrap(); // safed by main()
    let annotation_name = scmd.value_of("annotation_name").unwrap(); // safed by clap
    let delete = scmd.is_present("delete-annotation");
    let ids = rt
        .ids::<crate::ui::PathProvider>()
        .map_err_trace_exit_unwrap()
        .unwrap_or_else(|| {
            error!("No ids supplied");
            ::std::process::exit(1);
        })
        .into_iter();

    ids.for_each(|id| {
        let mut entry = rt.store()
            .get(id.clone())
            .map_err_trace_exit_unwrap()
            .ok_or_else(|| EM::EntryNotFound(id.local_display_string()))
            .map_err(Error::from)
            .map_err_trace_exit_unwrap();

        let annotation = entry
            .denotate(rt.store(), annotation_name)
            .map_err_trace_exit_unwrap();

        if delete {
            debug!("Deleting annotation object");
            if let Some(an) = annotation {
                let loc = an.get_location().clone();
                drop(an);

                rt
                    .store()
                    .delete(loc)
                    .map_err_trace_exit_unwrap();
            } else {
                warn!("Not having annotation object, cannot delete!");
            }
        } else {
            debug!("Not deleting annotation object");
        }
    })

}

fn list(rt: &Runtime) {
    let scmd = rt.cli().subcommand_matches("list").unwrap(); // safed by clap
    let with_text = scmd.is_present("list-with-text");
    let ids = rt
        .ids::<crate::ui::PathProvider>()
        .map_err_trace_exit_unwrap()
        .unwrap_or_else(|| {
            error!("No ids supplied");
            ::std::process::exit(1);
        })
        .into_iter();

    if ids.len() != 0 {
        ids
            .for_each(|id| {
                rt
                    .store()
                    .get(id.clone())
                    .map_err_trace_exit_unwrap()
                    .ok_or_else(|| EM::EntryNotFound(id.local_display_string()))
                    .map_err(Error::from)
                    .map_err_trace_exit_unwrap()
                    .annotations()
                    .map_err_trace_exit_unwrap()
                    .into_get_iter(rt.store())
                    .trace_unwrap_exit()
                    .map(|opt| opt.ok_or_else(|| format_err!("Cannot find entry")))
                    .trace_unwrap_exit()
                    .enumerate()
                    .for_each(|(i, entry)| list_annotation(&rt, i, entry, with_text));
            });
    } else { // ids.len() == 0
        // show them all
        rt.store()
            .all_annotations()
            .map_err_trace_exit_unwrap()
            .into_get_iter()
            .trace_unwrap_exit()
            .map(|opt| opt.ok_or_else(|| format_err!("Cannot find entry")))
            .trace_unwrap_exit()
            .enumerate()
            .for_each(|(i, entry)| list_annotation(&rt, i, entry, with_text));
    }
}

fn list_annotation<'a>(rt: &Runtime, i: usize, a: FileLockEntry<'a>, with_text: bool) {
    if with_text {
        writeln!(rt.stdout(),
                 "--- {i: >5} | {id}\n{text}\n\n",
                 i = i,
                 id = a.get_location(),
                 text = a.get_content())
    } else {
        writeln!(rt.stdout(), "{: >5} | {}", i, a.get_location())
    }
    .to_exit_code()
    .unwrap_or_exit();
}

