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
extern crate resiter;

extern crate libimagentrycategory;
extern crate libimagerror;
extern crate libimagrt;
extern crate libimagstore;
extern crate libimaginteraction;

use failure::Fallible as Result;
use clap::App;

use libimagerror::trace::MapErrTrace;
use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;

mod ui;

use std::io::Write;

use failure::err_msg;
use failure::Error;
use resiter::AndThen;
use resiter::IterInnerOkOrElse;

use libimagentrycategory::store::CategoryStore;
use libimagstore::iter::get::StoreIdGetIteratorExtension;
use libimagentrycategory::entry::EntryCategory;
use libimagentrycategory::category::Category;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagCategory {}
impl ImagApplication for ImagCategory {
    fn run(rt: Runtime) -> Result<()> {
        match rt.cli().subcommand_name().ok_or_else(|| err_msg("No subcommand called"))? {
            "set"               => set(&rt),
            "get"               => get(&rt),
            "list-category"     => list_category(&rt),
            "create-category"   => create_category(&rt),
            "delete-category"   => delete_category(&rt),
            "list-categories"   => list_categories(&rt),
            other               => {
                debug!("Unknown command");
                if rt.handle_unknown_subcommand("imag-category", other, rt.cli())?.success() {
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
        "Add a category to entries and manage categories"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}


fn set(rt: &Runtime) -> Result<()> {
    let scmd = rt.cli().subcommand_matches("set").unwrap(); // safed by main()
    let name = scmd.value_of("set-name").map(String::from).unwrap(); // safed by clap
    rt.ids::<crate::ui::PathProvider>()?
        .ok_or_else(|| err_msg("No ids supplied"))?
        .into_iter()
        .map(Ok)
        .into_get_iter(rt.store())
        .map_inner_ok_or_else(|| err_msg("Did not find one entry"))
        .and_then_ok(|mut e| e.set_category_checked(rt.store(), &name))
        .collect()
}

fn get(rt: &Runtime) -> Result<()> {
    let out = rt.stdout();
    let mut outlock = out.lock();
    rt.ids::<crate::ui::PathProvider>()?
        .ok_or_else(|| err_msg("No ids supplied"))?
        .into_iter()
        .map(Ok)
        .into_get_iter(rt.store())
        .map(|el| el.and_then(|o| o.ok_or_else(|| err_msg("Did not find one entry"))))
        .map(|entry| entry.and_then(|e| e.get_category()))
        .map(|name| name.and_then(|n| writeln!(outlock, "{}", n).map_err(Error::from)))
        .collect()
}

fn list_category(rt: &Runtime) -> Result<()> {
    let scmd = rt.cli().subcommand_matches("list-category").unwrap(); // safed by main()
    let name = scmd.value_of("list-category-name").map(String::from).unwrap(); // safed by clap

    if let Some(category) = rt.store().get_category_by_name(&name)? {
        let out         = rt.stdout();
        let mut outlock = out.lock();

        category
            .get_entries(rt.store())
            .map_err_trace_exit_unwrap()
            .map(|entry| writeln!(outlock, "{}", entry?.get_location()).map_err(Error::from))
            .collect()
    } else {
        Err(format_err!("No category named '{}'", name))
    }
}

fn create_category(rt: &Runtime) -> Result<()> {
    let scmd = rt.cli().subcommand_matches("create-category").unwrap(); // safed by main()
    let name = scmd.value_of("create-category-name").map(String::from).unwrap(); // safed by clap
    rt.store().create_category(&name).map(|_| ())
}

fn delete_category(rt: &Runtime) -> Result<()> {
    use libimaginteraction::ask::ask_bool;

    let scmd   = rt.cli().subcommand_matches("delete-category").unwrap(); // safed by main()
    let name   = scmd.value_of("delete-category-name").map(String::from).unwrap(); // safed by clap
    let ques   = format!("Do you really want to delete category '{}' and remove links to all categorized enties?", name);

    let mut input  = rt.stdin().ok_or_else(|| err_msg("No input stream. Cannot ask for permission"))?;
    let mut output = rt.stdout();
    let answer = ask_bool(&ques, Some(false), &mut input, &mut output)?;

    if answer {
        info!("Deleting category '{}'", name);
        rt.store().delete_category(&name).map(|_| ())
    } else {
        info!("Not doing anything");
        Ok(())
    }
}

fn list_categories(rt: &Runtime) -> Result<()> {
    let out         = rt.stdout();
    let mut outlock = out.lock();

    rt.store()
        .all_category_names()?
        .map(|name| name.and_then(|n| writeln!(outlock, "{}", n).map_err(Error::from)))
        .collect()
}

