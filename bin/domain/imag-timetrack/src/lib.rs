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

#[macro_use]
extern crate log;

extern crate clap;
extern crate chrono;
extern crate filters;
extern crate itertools;
extern crate prettytable;
extern crate kairos;
extern crate failure;

extern crate libimagerror;
extern crate libimagstore;
extern crate libimagrt;
extern crate libimagtimetrack;
extern crate libimagutil;

mod cont;
mod day;
mod list;
mod month;
mod shell;
mod start;
mod stop;
mod track;
mod ui;
mod week;
mod year;

use crate::cont::cont;
use crate::day::day;
use crate::list::{list, list_impl};
use crate::month::month;
use crate::shell::shell;
use crate::start::start;
use crate::stop::stop;
use crate::track::track;
use crate::week::week;
use crate::year::year;

use clap::App;
use failure::Fallible as Result;

use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
use libimagerror::trace::MapErrTrace;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagTimetrack {}
impl ImagApplication for ImagTimetrack {
    fn run(rt: Runtime) -> Result<()> {
        let command = rt.cli().subcommand_name();
        let retval  = if let Some(command) = command {
            debug!("Call: {}", command);
            match command {
                "continue" => cont(&rt),
                "day"      => day(&rt),
                "list"     => list(&rt),
                "month"    => month(&rt),
                "shell"    => shell(&rt),
                "start"    => start(&rt),
                "stop"     => stop(&rt),
                "track"    => track(&rt),
                "week"     => week(&rt),
                "year"     => year(&rt),
                other      => {
                    debug!("Unknown command");
                    rt.handle_unknown_subcommand("imag-timetrack", other, rt.cli())
                        .map_err_trace_exit_unwrap()
                        .code()
                        .unwrap_or(0)
                },
            }
        } else {
            let start = ::chrono::offset::Local::today().naive_local().and_hms(0, 0, 0);
            let end   = ::chrono::offset::Local::today().naive_local().and_hms(23, 59, 59);
            list_impl(&rt, Some(start), Some(end), false, false)
        };

        ::std::process::exit(retval);
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Time tracking module"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
