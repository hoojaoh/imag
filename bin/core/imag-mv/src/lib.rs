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

#[macro_use] extern crate log;
#[macro_use] extern crate failure;
extern crate clap;

extern crate libimagrt;
extern crate libimagstore;
extern crate libimagerror;
extern crate libimagentrylink;

mod ui;

use std::path::PathBuf;

use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
use libimagstore::storeid::StoreId;
use libimagstore::store::Store;
use libimagstore::store::FileLockEntry;
use libimagentrylink::linkable::Linkable;
use libimagstore::iter::get::StoreIdGetIteratorExtension;
use libimagerror::iter::IterInnerOkOrElse;

use failure::Fallible as Result;
use failure::err_msg;
use clap::App;


/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagMv {}
impl ImagApplication for ImagMv {
    fn run(rt: Runtime) -> Result<()> {
        let sourcename = rt
            .cli()
            .value_of("source")
            .map(PathBuf::from)
            .map(StoreId::new)
            .unwrap()?; // unwrap safe by clap

        let destname = rt
            .cli()
            .value_of("dest")
            .map(PathBuf::from)
            .map(StoreId::new)
            .unwrap()?; // unwrap safe by clap

        // remove links to entry, and re-add them later
        let mut linked_entries = rt.store()
            .get(sourcename.clone())?
            .ok_or_else(|| format_err!("Entry does not exist: {}", sourcename))?
            .links()?
            .map(|link| link.get_store_id().clone())
            .map(Ok)
            .into_get_iter(rt.store())
            .map_inner_ok_or_else(|| err_msg("Linked entry does not exist"))
            .collect::<Result<Vec<_>>>()?;

        { // remove links to linked entries from source
            let mut entry = rt
                .store()
                .get(sourcename.clone())?
                .ok_or_else(|| err_msg("Source Entry does not exist"))?;

            for link in linked_entries.iter_mut() {
                entry.remove_link(link)?;
            }
        }

        if let Err(e) = rt.store().move_by_id(sourcename.clone(), destname.clone()) {
            debug!("Re-adding links to source entry because moving failed");
            relink(rt.store(), sourcename.clone(), &mut linked_entries)?;

            return Err(e);
        }

        rt.report_touched(&destname)?;

        // re-add links to moved entry
        relink(rt.store(), destname, &mut linked_entries)?;

        info!("Ok.");
        Ok(())
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Move things around in the store"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}



fn relink<'a>(store: &'a Store, target: StoreId, linked_entries: &mut Vec<FileLockEntry<'a>>) -> Result<()> {
    let mut entry = store
        .get(target)?
        .ok_or_else(|| err_msg("Funny things happened: Entry moved to destination did not fail, but entry does not exist"))?;

    for mut link in linked_entries {
        let _ = entry.add_link(&mut link)?;
    }

    Ok(())
}

