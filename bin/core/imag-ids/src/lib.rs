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

#[cfg(test)]
extern crate env_logger;

extern crate libimagerror;
extern crate libimagstore;
extern crate libimagrt;

use std::io::Write;

use failure::Fallible as Result;
use failure::err_msg;
use failure::Error;
use resiter::Map;
use resiter::AndThen;
use clap::App;

use libimagstore::storeid::StoreId;
use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;

mod ui;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagIds {}
impl ImagApplication for ImagIds {
    fn run(rt: Runtime) -> Result<()> {
        let print_storepath = rt.cli().is_present("print-storepath");

        let mut stdout = rt.stdout();
        trace!("Got output: {:?}", stdout);

        let mut process = |iter: &mut dyn Iterator<Item = Result<StoreId>>| -> Result<()> {
            iter.map_ok(|id| if print_storepath {
                (Some(rt.store().path()), id)
            } else {
                (None, id)
            }).and_then_ok(|(storepath, id)| {
                if !rt.output_is_pipe() {
                    let id = id.to_str()?;
                    trace!("Writing to {:?}", stdout);

                    if let Some(store) = storepath {
                        writeln!(stdout, "{}/{}", store.display(), id)?;
                    } else {
                        writeln!(stdout, "{}", id)?;
                    }
                }

                rt.report_touched(&id).map_err(Error::from)
            })
            .collect::<Result<()>>()
        };

        if rt.ids_from_stdin() {
            debug!("Fetching IDs from stdin...");
            let mut iter = rt.ids::<crate::ui::PathProvider>()?
                .ok_or_else(|| err_msg("No ids supplied"))?
                .into_iter()
                .map(Ok);

            process(&mut iter)
        } else {
            process(&mut rt.store().entries()?)
        }
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "print all ids"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
