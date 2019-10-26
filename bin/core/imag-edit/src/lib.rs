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
extern crate resiter;

extern crate libimagentryedit;
extern crate libimagerror;
extern crate libimagrt;
extern crate libimagstore;
extern crate libimagutil;

use libimagentryedit::edit::Edit;
use libimagentryedit::edit::EditHeader;
use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
use libimagstore::iter::get::StoreIdGetIteratorExtension;

use failure::Fallible as Result;
use failure::err_msg;
use resiter::AndThen;
use resiter::IterInnerOkOrElse;
use clap::App;

mod ui;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagEdit {}
impl ImagApplication for ImagEdit {
    fn run(rt: Runtime) -> Result<()> {
        let edit_header = rt.cli().is_present("edit-header");
        let edit_header_only = rt.cli().is_present("edit-header-only");

        rt.ids::<crate::ui::PathProvider>()?
            .ok_or_else(|| err_msg("No ids supplied"))?
            .into_iter()
            .map(Ok)
            .into_get_iter(rt.store())
            .map_inner_ok_or_else(|| err_msg("Did not find one entry"))
            .inspect(|e| debug!("Editing = {:?}", e))
            .and_then_ok(|mut entry| {
                if edit_header {
                    entry.edit_header_and_content(&rt)
                } else if edit_header_only {
                    entry.edit_header(&rt)
                } else {
                    entry.edit_content(&rt)
                }
            })
            .collect()
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Edit store entries with $EDITOR"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
