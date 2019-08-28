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
extern crate toml;
extern crate toml_query;
#[macro_use] extern crate failure;

extern crate libimagbookmark;
#[macro_use] extern crate libimagrt;
extern crate libimagerror;
extern crate libimagutil;
extern crate libimagentrylink;

use std::io::Write;
use std::process::exit;
use std::ops::DerefMut;

use toml_query::read::TomlValueReadTypeExt;
use failure::Error;

use libimagrt::runtime::Runtime;
use libimagrt::setup::generate_runtime_setup;
use libimagbookmark::collection::BookmarkCollection;
use libimagbookmark::collection::BookmarkCollectionStore;
use libimagbookmark::link::Link as BookmarkLink;
use libimagerror::trace::{MapErrTrace, trace_error};
use libimagerror::io::ToExitCode;
use libimagerror::exit::ExitUnwrap;
use libimagutil::debug_result::DebugResult;
use libimagentrylink::linkable::Linkable;


mod ui;

use crate::ui::build_ui;

fn main() {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-bookmark",
                                    &version,
                                    "Bookmark collection tool",
                                    build_ui);

    if let Some(name) = rt.cli().subcommand_name() {
        debug!("Call {}", name);
        match name {
            "add"        => add(&rt),
            "collection" => collection(&rt),
            "list"       => list(&rt),
            "remove"     => remove(&rt),
            other        => {
                debug!("Unknown command");
                let _ = rt.handle_unknown_subcommand("imag-bookmark", other, rt.cli())
                    .map_err_trace_exit_unwrap()
                    .code()
                    .map(::std::process::exit);
            },
        }
    }
}

fn add(rt: &Runtime) {
    let scmd = rt.cli().subcommand_matches("add").unwrap();
    let coll = get_collection_name(rt, "add", "collection");

    let mut collection = BookmarkCollectionStore::get(rt.store(), &coll)
        .map_err_trace_exit_unwrap()
        .ok_or_else(|| format_err!("No bookmark collection '{}' found", coll))
        .map_err_trace_exit_unwrap();

    rt.report_touched(collection.get_location()).unwrap_or_exit();

    for url in scmd.values_of("urls").unwrap() { // unwrap saved by clap
        let new_ids = BookmarkCollection::add_link(collection.deref_mut(), rt.store(), BookmarkLink::from(url))
            .map_err_trace_exit_unwrap();

        rt.report_all_touched(new_ids.into_iter()).unwrap_or_exit();
    }

    info!("Ready");
}

fn collection(rt: &Runtime) {
    let scmd = rt.cli().subcommand_matches("collection").unwrap();

    if scmd.is_present("add") { // adding a new collection
        let name = scmd.value_of("add").unwrap();
        if let Ok(id) = BookmarkCollectionStore::new(rt.store(), &name) {
            rt.report_touched(id.get_location()).unwrap_or_exit();
            info!("Created: {}", name);
        } else {
            warn!("Creating collection {} failed", name);
            exit(1);
        }
    }

    if scmd.is_present("remove") { // remove a collection
        let name = scmd.value_of("remove").unwrap();

        { // remove all links
            BookmarkCollectionStore::get(rt.store(), &name)
                .map_err_trace_exit_unwrap()
                .ok_or_else(|| format_err!("Collection does not exist: {}", name))
                .map_err_trace_exit_unwrap()
                .unlink(rt.store())
                .map_err_trace_exit_unwrap();
        }

        if BookmarkCollectionStore::delete(rt.store(), &name).is_ok() {
            info!("Deleted: {}", name);
        } else {
            warn!("Deleting collection {} failed", name);
            exit(1);
        }
    }
}

fn list(rt: &Runtime) {
    let coll = get_collection_name(rt, "list", "collection");

    let collection = BookmarkCollectionStore::get(rt.store(), &coll)
        .map_err_trace_exit_unwrap()
        .ok_or_else(|| format_err!("No bookmark collection '{}' found", coll))
        .map_err_trace_exit_unwrap();

    rt.report_touched(collection.get_location()).unwrap_or_exit();

    collection
        .get_links(rt.store())
        .map_dbg_str("Listing...")
        .map_err_trace_exit_unwrap()
        .enumerate()
        .for_each(|(i, link)| match link {
            Ok(link) => writeln!(rt.stdout(), "{: >3}: {}", i, link).to_exit_code().unwrap_or_exit(),
            Err(e)   => trace_error(&e)
        });
    debug!("... ready with listing");
}

fn remove(rt: &Runtime) {
    let scmd = rt.cli().subcommand_matches("remove").unwrap();
    let coll = get_collection_name(rt, "list", "collection");

    let mut collection = BookmarkCollectionStore::get(rt.store(), &coll)
        .map_err_trace_exit_unwrap()
        .ok_or_else(|| format_err!("No bookmark collection '{}' found", coll))
        .map_err_trace_exit_unwrap();

    rt.report_touched(collection.get_location()).unwrap_or_exit();

    for url in scmd.values_of("urls").unwrap() { // enforced by clap
        let removed_links = BookmarkCollection::remove_link(collection.deref_mut(), rt.store(), BookmarkLink::from(url))
            .map_err_trace_exit_unwrap();

        rt.report_all_touched(removed_links.into_iter()).unwrap_or_exit();
    }

    info!("Ready");
}


fn get_collection_name(rt: &Runtime,
                       subcommand_name: &str,
                       collection_argument_name: &str)
    -> String
{
    rt.cli()
        .subcommand_matches(subcommand_name)
        .and_then(|scmd| scmd.value_of(collection_argument_name).map(String::from))
        .unwrap_or_else(|| {
            rt.config()
                .map(|cfg| {
                    cfg.read_string("bookmark.default_collection")
                        .map_err(Error::from)
                        .map_err_trace_exit_unwrap()
                        .ok_or_else(|| {
                            error!("Missing config: 'bookmark.default_collection'. Set or use commandline to specify.");
                            exit(1)
                        })
                        .unwrap()
                        .clone()
                })
                .unwrap_or_else(|| {
                    error!("Failed to read configuration");
                    exit(1)
                })
        })
}

