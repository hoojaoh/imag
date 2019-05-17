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
extern crate failure;

extern crate libimagerror;
#[macro_use] extern crate libimagrt;
extern crate libimagstore;

use std::io::Write;

use failure::Error;
use failure::err_msg;

use libimagerror::trace::MapErrTrace;
use libimagerror::iter::TraceIterator;
use libimagrt::setup::generate_runtime_setup;
use libimagstore::iter::get::StoreIdGetIteratorExtension;

mod ui;

fn main() {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-markdown",
                                    &version,
                                    "Print one or more imag entries after processing them with a markdown parser",
                                    ui::build_ui);

    let out = rt.stdout();
    let mut outlock = out.lock();

    rt.ids::<crate::ui::PathProvider>()
        .map_err_trace_exit_unwrap()
        .into_iter()
        .map(Ok)
        .into_get_iter(rt.store())
        .trace_unwrap_exit()
        .map(|ofle| ofle.ok_or_else(|| {
            err_msg("Entry does not exist but is in store. This is a BUG, please report!")
        }))
        .trace_unwrap_exit()
        .map(|fle| libimagentrymarkdown::html::to_html(fle.get_content()))
        .trace_unwrap_exit()
        .for_each(|html| {
            writeln!(outlock, "{}", html).map_err(Error::from).map_err_trace_exit_unwrap();
        })
}

