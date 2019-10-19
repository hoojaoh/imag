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
#[macro_use] extern crate failure;

extern crate libimagentrygps;
extern crate libimagrt;
extern crate libimagutil;
extern crate libimagerror;
extern crate libimagstore;

use std::io::Write;
use std::str::FromStr;

use failure::Error;
use failure::Fallible as Result;
use failure::err_msg;
use clap::App;

use libimagstore::storeid::StoreId;
use libimagentrygps::types::*;
use libimagentrygps::entry::*;
use libimagrt::application::ImagApplication;
use libimagrt::runtime::Runtime;

mod ui;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagGps {}
impl ImagApplication for ImagGps {
    fn run(rt: Runtime) -> Result<()> {
        match rt.cli().subcommand_name().ok_or_else(|| err_msg("No subcommand called"))? {
            "add"    => add(&rt),
            "remove" => remove(&rt),
            "get"    => get(&rt),
            other    => {
                debug!("Unknown command");
                if rt.handle_unknown_subcommand("imag-gps", other, rt.cli())
                    .map_err(Error::from)?
                    .success()
                {
                    Ok(())
                } else {
                    Err(format_err!("Subcommand failed"))
                }
            }
        }
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Add GPS coordinates to entries"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

fn rt_get_ids(rt: &Runtime) -> Result<Vec<StoreId>> {
    rt
        .ids::<crate::ui::PathProvider>()?
        .ok_or_else(|| err_msg("No ids supplied"))
}

fn add(rt: &Runtime) -> Result<()> {
    let c = {
        let parse = |value: &str| -> Result<(i64, i64, i64)> {
            debug!("Parsing '{}' into degree, minute and second", value);
            let ary = value.split('.')
                .map(|v| {debug!("Parsing = {}", v); v})
                .map(FromStr::from_str)
                .map(|elem| elem.or_else(|_| Err(err_msg("Error while converting number"))))
                .collect::<Result<Vec<i64>>>()?;

            let degree = ary.get(0).ok_or_else(|| err_msg("Degree missing. This value is required."))?;
            let minute = ary.get(1).ok_or_else(|| err_msg("Degree missing. This value is required."))?;
            let second = ary.get(2).unwrap_or(&0);

            Ok((*degree, *minute, *second))
        };

        let scmd = rt.cli().subcommand_matches("add").unwrap(); // safed by main()

        let long = parse(scmd.value_of("longitude").unwrap())?; // unwrap safed by clap
        let lati = parse(scmd.value_of("latitude").unwrap())?; // unwrap safed by clap

        let long = GPSValue::new(long.0, long.1, long.2);
        let lati = GPSValue::new(lati.0, lati.1, lati.2);

        Coordinates::new(long, lati)
    };

    rt_get_ids(&rt)?
        .into_iter()
        .map(|id| {
            rt.store()
                .get(id.clone())?
                .ok_or_else(|| format_err!("No such entry: {}", id))?
                .set_coordinates(c.clone())?;

            rt.report_touched(&id).map_err(Error::from)
        })
        .collect()
}

fn remove(rt: &Runtime) -> Result<()> {
    let print_removed = rt
        .cli()
        .subcommand_matches("remove")
        .unwrap()
        .is_present("print-removed"); // safed by main()

    rt_get_ids(&rt)?
        .into_iter()
        .map(|id| {
            let removed_value : Coordinates = rt
                .store()
                .get(id.clone())?
                .ok_or_else(|| format_err!("No such entry: {}", id))?
                .remove_coordinates()?
                .ok_or_else(|| format_err!("Entry had no coordinates: {}", id))??;

            if print_removed {
                writeln!(rt.stdout(), "{}", removed_value)?;
            }

            rt.report_touched(&id).map_err(Error::from)
        })
        .collect()
}

fn get(rt: &Runtime) -> Result<()> {
    let mut stdout = rt.stdout();

    rt_get_ids(&rt)?
        .into_iter()
        .map(|id| {
            let value = rt
                .store()
                .get(id.clone())?
                .ok_or_else(|| { // if we have Ok(None)
                    format_err!("No such entry: {}", id)
                })?
                .get_coordinates()?
                .ok_or_else(|| { // if we have Ok(None)
                    format_err!("Entry has no coordinates: {}", id)
                })?;

            writeln!(stdout, "{}", value)?;

            rt.report_touched(&id).map_err(Error::from)
        })
        .collect()
}

