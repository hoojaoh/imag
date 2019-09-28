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

#[macro_use] extern crate failure;
#[macro_use] extern crate log;
extern crate clap;
extern crate toml_query;
extern crate walkdir;

#[macro_use] extern crate libimagrt;
extern crate libimagcalendar;
extern crate libimagerror;
extern crate libimagstore;
extern crate libimagutil;

use std::path::PathBuf;

use failure::Error;
use failure::Fallible as Result;
use toml_query::read::Partial;
use toml_query::read::TomlValueReadExt;
use walkdir::DirEntry;
use walkdir::WalkDir;

use libimagcalendar::store::EventStore;
use libimagerror::exit::ExitUnwrap;
use libimagerror::iter::TraceIterator;
use libimagerror::trace::MapErrTrace;
use libimagrt::runtime::Runtime;
use libimagrt::setup::generate_runtime_setup;

mod ui;

fn main() {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-calendar",
                                    &version,
                                    "Calendar management tool",
                                    crate::ui::build_ui);


    if let Some(name) = rt.cli().subcommand_name() {
        debug!("Call {}", name);
        match name {
            "import" => import(&rt),
            other    => {
                warn!("Right now, only the 'import' command is available");
                debug!("Unknown command");
                let _ = rt.handle_unknown_subcommand("imag-calendar", other, rt.cli())
                    .map_err_trace_exit_unwrap()
                    .code()
                    .map(::std::process::exit);
            },
        }
    }
}

fn import(rt: &Runtime) {
    let scmd            = rt.cli().subcommand_matches("import").unwrap(); // safe by clap
    let collection_name = rt.cli().value_of("calendar-ref-collection-name").unwrap(); // default by clap
    let do_fail         = scmd.is_present("import-fail");
    let force_override  = scmd.is_present("import-force-override");
    let ref_config      = rt.config()
        .ok_or_else(|| format_err!("No configuration, cannot continue!"))
        .map_err_trace_exit_unwrap()
        .read_partial::<libimagentryref::reference::Config>()
        .map_err(Error::from)
        .map_err_trace_exit_unwrap()
        .ok_or_else(|| format_err!("Configuration missing: {}", libimagentryref::reference::Config::LOCATION))
        .map_err_trace_exit_unwrap();

    scmd.values_of("filesordirs")
        .unwrap() // save by clap
        .into_iter()
        .map(PathBuf::from)
        .map(|path| if path.is_dir() { // Find all files
            Box::new(WalkDir::new(path)
                .follow_links(false)
                .into_iter()
                .filter_entry(is_not_hidden)
                .filter_map(|r| match r {
                    Err(e) => Some(Err(Error::from(e))),
                    Ok(fe) => {
                        if fe.file_type().is_file() {
                            Some(Ok(fe.into_path()))
                        } else {
                            None // filter out directories
                        }
                    }
                })) as Box<dyn Iterator<Item = Result<PathBuf>>>
        } else { // is file, ensured by clap validator
            Box::new(std::iter::once(Ok(path)))
        })
        .flat_map(|it| it)   // From Iter<Iter<Result<PathBuf>>> to Iter<Result<PathBuf>>
        .trace_unwrap_exit() //... to Iter<PathBuf>
        .map(|path| {
            let v = rt.store().import_from_path(path, collection_name, &ref_config, force_override)?;
            Ok(v.into_iter()
                .filter_map(|result| if do_fail {
                    Some(result.map_err_trace_exit_unwrap())
                } else {
                    match result {
                        Err(e)  => { warn!("Error while importing: {}", e); None }
                        Ok(fle) => Some(fle),
                    }
                }))
        })
        .trace_unwrap_exit()
        .flat_map(|it| it)
        .for_each(|fle| rt.report_touched(fle.get_location()).unwrap_or_exit());
}

/// helper function to check whether a DirEntry points to something hidden (starting with dot)
fn is_not_hidden(entry: &DirEntry) -> bool {
    !entry.file_name().to_str().map(|s| s.starts_with(".")).unwrap_or(false)
}

