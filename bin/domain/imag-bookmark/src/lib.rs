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
extern crate resiter;

extern crate libimagbookmark;
extern crate libimagrt;
extern crate libimagerror;
extern crate libimagutil;
extern crate libimagentrylink;

use std::io::Write;
use std::ops::DerefMut;

use toml_query::read::TomlValueReadTypeExt;
use failure::Error;
use failure::err_msg;
use failure::Fallible as Result;
use resiter::AndThen;
use clap::App;

use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
use libimagbookmark::collection::BookmarkCollection;
use libimagbookmark::collection::BookmarkCollectionStore;
use libimagbookmark::link::Link as BookmarkLink;
use libimagentrylink::linkable::Linkable;

mod ui;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagBookmark {}
impl ImagApplication for ImagBookmark {
    fn run(rt: Runtime) -> Result<()> {
        match rt.cli().subcommand_name().ok_or_else(|| err_msg("No subcommand called"))? {
            "add"        => add(&rt),
            "collection" => collection(&rt),
            "list"       => list(&rt),
            "remove"     => remove(&rt),
            other        => {
                debug!("Unknown command");
                if rt.handle_unknown_subcommand("imag-bookmark", other, rt.cli())?.success() {
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
        "Bookmark collection tool"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

fn add(rt: &Runtime) -> Result<()> {
    let scmd = rt.cli().subcommand_matches("add").unwrap();
    let coll = get_collection_name(rt, "add", "collection")?;

    let mut collection = BookmarkCollectionStore::get(rt.store(), &coll)?
        .ok_or_else(|| format_err!("No bookmark collection '{}' found", coll))?;

    rt.report_touched(collection.get_location())?;

    scmd.values_of("urls")
        .unwrap()
        .into_iter()
        .map(|url| {
            let new_ids = BookmarkCollection::add_link(collection.deref_mut(), rt.store(), BookmarkLink::from(url))?;
            rt.report_all_touched(new_ids.into_iter()).map_err(Error::from)
        })
        .collect()
}

fn collection(rt: &Runtime) -> Result<()> {
    let scmd = rt.cli().subcommand_matches("collection").unwrap();

    if scmd.is_present("add") { // adding a new collection
        let name = scmd.value_of("add").unwrap();
        let id   = BookmarkCollectionStore::new(rt.store(), &name)?;
        rt.report_touched(id.get_location())?;
        info!("Created: {}", name);
    }

    if scmd.is_present("remove") { // remove a collection
        let name = scmd.value_of("remove").unwrap();

        { // remove all links
            BookmarkCollectionStore::get(rt.store(), &name)?
                .ok_or_else(|| format_err!("Collection does not exist: {}", name))?
                .unlink(rt.store())?;
        }

        BookmarkCollectionStore::delete(rt.store(), &name)?;
        info!("Deleted: {}", name);
    }

    Ok(())
}

fn list(rt: &Runtime) -> Result<()> {
    let coll = get_collection_name(rt, "list", "collection")?;

    let collection = BookmarkCollectionStore::get(rt.store(), &coll)?
        .ok_or_else(|| format_err!("No bookmark collection '{}' found", coll))?;

    rt.report_touched(collection.get_location())?;

    let mut i = 0; // poor mans enumerate()

    collection
        .get_links(rt.store())?
        .and_then_ok(|link| {
            let r = writeln!(rt.stdout(), "{: >3}: {}", i, link).map_err(Error::from);
            i += 1;
            r
        })
        .collect()
}

fn remove(rt: &Runtime) -> Result<()> {
    let scmd = rt.cli().subcommand_matches("remove").unwrap();
    let coll = get_collection_name(rt, "list", "collection")?;

    let mut collection = BookmarkCollectionStore::get(rt.store(), &coll)?
        .ok_or_else(|| format_err!("No bookmark collection '{}' found", coll))?;

    rt.report_touched(collection.get_location())?;

    scmd.values_of("urls")
        .unwrap()
        .into_iter()
        .map(|url| {
            let removed_links = BookmarkCollection::remove_link(collection.deref_mut(), rt.store(), BookmarkLink::from(url))?;
            rt.report_all_touched(removed_links.into_iter()).map_err(Error::from)
        })
        .collect()
}


fn get_collection_name(rt: &Runtime,
                       subcommand_name: &str,
                       collection_argument_name: &str)
    -> Result<String>
{
    if let Some(cn) = rt.cli()
        .subcommand_matches(subcommand_name)
        .and_then(|scmd| scmd.value_of(collection_argument_name).map(String::from))
    {
        return Ok(cn)
    } else {
        rt.config().ok_or_else(|| err_msg("No configuration availablew"))
            .and_then(|cfg| {
                cfg.read_string("bookmark.default_collection")?
                    .ok_or_else(|| err_msg("Missing config: 'bookmark.default_collection'."))
            })
    }
}

