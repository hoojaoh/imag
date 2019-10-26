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
extern crate failure;

extern crate libimagerror;
extern crate libimagrt;
extern crate libimagstore;

use failure::Fallible as Result;
use failure::err_msg;
use clap::App;

use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
use libimagrt::iter::ReportTouchedResultEntry;
use libimagstore::iter::create::StoreIdCreateIteratorExtension;
use libimagstore::iter::retrieve::StoreIdRetrieveIteratorExtension;

mod ui;



pub enum ImagCreate {}
impl ImagApplication for ImagCreate {
    fn run(rt: Runtime) -> Result<()> {
        let force = rt.cli().is_present("force");

        let ids = rt.ids::<crate::ui::PathProvider>()?
            .ok_or_else(|| err_msg("No ids supplied"))?
            .into_iter()
            .map(Ok);

        if force {
            ids.into_retrieve_iter(rt.store()).map_report_touched(&rt).collect::<Result<Vec<_>>>()
        } else {
            ids.into_create_iter(rt.store()).map_report_touched(&rt).collect::<Result<Vec<_>>>()
        }.map(|_| ())
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Plumbing tool to create entries"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

