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
use std::process::exit;
use std::str::FromStr;


use failure::err_msg;
use failure::Fallible as Result;
use clap::App;

use libimagstore::storeid::StoreId;
use libimagentrygps::types::*;
use libimagentrygps::entry::*;
use libimagrt::application::ImagApplication;
use libimagrt::runtime::Runtime;
use libimagerror::trace::MapErrTrace;
use libimagerror::exit::ExitUnwrap;
use libimagerror::io::ToExitCode;

mod ui;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagGps {}
impl ImagApplication for ImagGps {
    fn run(rt: Runtime) -> Result<()> {
        if let Some(name) = rt.cli().subcommand_name() {
            match name {
                "add"    => add(&rt),
                "remove" => remove(&rt),
                "get"    => get(&rt),
                other    => {
                    debug!("Unknown command");
                    let _ = rt.handle_unknown_subcommand("imag-gps", other, rt.cli())
                        .map_err_trace_exit_unwrap()
                        .code()
                        .map(::std::process::exit);
                }
            }
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
        "Add GPS coordinates to entries"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

fn rt_get_ids(rt: &Runtime) -> Vec<StoreId> {
    rt
        .ids::<crate::ui::PathProvider>()
        .map_err_trace_exit_unwrap()
        .unwrap_or_else(|| {
            error!("No ids supplied");
            ::std::process::exit(1);
        })
}

fn add(rt: &Runtime) {
    let c = {
        let parse = |value: &str| -> (i64, i64, i64) {
            debug!("Parsing '{}' into degree, minute and second", value);
            let ary = value.split('.')
                .map(|v| {debug!("Parsing = {}", v); v})
                .map(FromStr::from_str)
                .map(|elem| {
                    elem.or_else(|_| Err(err_msg("Error while converting number")))
                        .map_err_trace_exit_unwrap()
                })
                .collect::<Vec<i64>>();

            let degree = ary.get(0).unwrap_or_else(|| {
                error!("Degree missing. This value is required.");
                exit(1)
            });
            let minute = ary.get(1).unwrap_or_else(|| {
                error!("Degree missing. This value is required.");
                exit(1)
            });
            let second = ary.get(2).unwrap_or(&0);

            (*degree, *minute, *second)
        };

        let scmd = rt.cli().subcommand_matches("add").unwrap(); // safed by main()

        let long = parse(scmd.value_of("longitude").unwrap()); // unwrap safed by clap
        let lati = parse(scmd.value_of("latitude").unwrap()); // unwrap safed by clap

        let long = GPSValue::new(long.0, long.1, long.2);
        let lati = GPSValue::new(lati.0, lati.1, lati.2);

        Coordinates::new(long, lati)
    };

    rt_get_ids(&rt)
        .into_iter()
        .for_each(|id| {
            rt.store()
                .get(id.clone())
                .map_err_trace_exit_unwrap()
                .unwrap_or_else(|| { // if we have Ok(None)
                    error!("No such entry: {}", id);
                    exit(1)
                })
                .set_coordinates(c.clone())
                .map_err_trace_exit_unwrap();

            rt.report_touched(&id).unwrap_or_exit();
        });
}

fn remove(rt: &Runtime) {
    let print_removed = rt
        .cli()
        .subcommand_matches("remove")
        .unwrap()
        .is_present("print-removed"); // safed by main()

    rt_get_ids(&rt)
        .into_iter()
        .for_each(|id| {
            let removed_value = rt
                .store()
                .get(id.clone())
                .map_err_trace_exit_unwrap()
                .unwrap_or_else(|| { // if we have Ok(None)
                    error!("No such entry: {}", id);
                    exit(1)
                })
                .remove_coordinates()
                .map_err_trace_exit_unwrap() // The delete action failed
                .unwrap_or_else(|| { // if we have Ok(None)
                    error!("Entry had no coordinates: {}", id);
                    exit(1)
                })
                .map_err_trace_exit_unwrap(); // The parsing of the deleted values failed

            if print_removed {
                writeln!(rt.stdout(), "{}", removed_value).to_exit_code().unwrap_or_exit();
            }

            rt.report_touched(&id).unwrap_or_exit();
        });
}

fn get(rt: &Runtime) {
    let mut stdout = rt.stdout();

    rt_get_ids(&rt)
        .into_iter()
        .for_each(|id| {
            let value = rt
                .store()
                .get(id.clone())
                .map_err_trace_exit_unwrap()
                .unwrap_or_else(|| { // if we have Ok(None)
                    error!("No such entry: {}", id);
                    exit(1)
                })
                .get_coordinates()
                .map_err_trace_exit_unwrap() // The get action failed
                .unwrap_or_else(|| { // if we have Ok(None)
                    error!("Entry has no coordinates: {}", id);
                    exit(1)
                });

            writeln!(stdout, "{}", value).to_exit_code().unwrap_or_exit();

            rt.report_touched(&id).unwrap_or_exit();
        })

}

