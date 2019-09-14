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
extern crate handlebars;
extern crate tempfile;
extern crate toml;
extern crate toml_query;
extern crate failure;

extern crate libimagentryview;
extern crate libimagerror;
extern crate libimagrt;
extern crate libimagstore;
extern crate libimagutil;

use std::str::FromStr;
use std::collections::BTreeMap;
use std::io::Write;
use std::process::Command;
use std::process::exit;

use handlebars::Handlebars;
use toml_query::read::TomlValueReadTypeExt;
use failure::Error;
use failure::err_msg;
use failure::Fallible as Result;
use clap::App;

use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
use libimagerror::trace::MapErrTrace;
use libimagerror::iter::TraceIterator;
use libimagerror::io::ToExitCode;
use libimagerror::exit::ExitUnwrap;
use libimagentryview::builtin::stdout::StdoutViewer;
use libimagentryview::builtin::md::MarkdownViewer;
use libimagentryview::viewer::Viewer;
use libimagstore::iter::get::StoreIdGetIteratorExtension;
use libimagstore::store::FileLockEntry;

mod ui;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagView {}
impl ImagApplication for ImagView {
    fn run(rt: Runtime) -> Result<()> {
        let view_header  = rt.cli().is_present("view-header");
        let hide_content = rt.cli().is_present("not-view-content");
        let entries      = rt
            .ids::<::ui::PathProvider>()
            .map_err_trace_exit_unwrap()
            .unwrap_or_else(|| {
                error!("No ids supplied");
                ::std::process::exit(1);
            })
            .into_iter()
            .map(Ok)
            .into_get_iter(rt.store())
            .trace_unwrap_exit()
            .map(|e| {
                e.ok_or_else(|| err_msg("Entry not found"))
                    .map_err(Error::from)
                    .map_err_trace_exit_unwrap()
            });

        if rt.cli().is_present("in") {
            let files = entries
                .map(|entry| {
                    let tmpfile = create_tempfile_for(&entry, view_header, hide_content);
                    rt.report_touched(entry.get_location()).unwrap_or_exit();
                    tmpfile
                })
                .collect::<Vec<_>>();

            let mut command = {
                let viewer = rt
                    .cli()
                    .value_of("in")
                    .ok_or_else(|| Error::from(err_msg("No viewer given")))
                    .map_err_trace_exit_unwrap();

                let config = rt
                    .config()
                    .ok_or_else(|| Error::from(err_msg("No configuration, cannot continue")))
                    .map_err_trace_exit_unwrap();

                let query = format!("view.viewers.{}", viewer);

                let viewer_template = config
                    .read_string(&query)
                    .map_err(Error::from)
                    .map_err_trace_exit_unwrap()
                    .unwrap_or_else(|| {
                        error!("Cannot find '{}' in config", query);
                        exit(1)
                    });

                let mut handlebars = Handlebars::new();
                handlebars.register_escape_fn(::handlebars::no_escape);

                let _ = handlebars
                    .register_template_string("template", viewer_template)
                    .map_err(Error::from)
                    .map_err_trace_exit_unwrap();

                let mut data = BTreeMap::new();

                let file_paths = files
                    .iter()
                    .map(|&(_, ref path)| path.clone())
                    .collect::<Vec<String>>()
                    .join(" ");

                data.insert("entries", file_paths);

                let call = handlebars
                    .render("template", &data)
                    .map_err(Error::from)
                    .map_err_trace_exit_unwrap();
                let mut elems = call.split_whitespace();
                let command_string = elems
                    .next()
                    .ok_or_else(|| Error::from(err_msg("No command")))
                    .map_err_trace_exit_unwrap();
                let mut cmd = Command::new(command_string);

                for arg in elems {
                    cmd.arg(arg);
                }

                cmd
            };

            debug!("Calling: {:?}", command);

            if !command
                .status()
                .map_err(Error::from)
                .map_err_trace_exit_unwrap()
                .success()
            {
                exit(1)
            }

            drop(files);
        } else {
            let out         = rt.stdout();
            let mut outlock = out.lock();

            let basesep = if rt.cli().occurrences_of("seperator") != 0 { // checker for default value
                rt.cli().value_of("seperator").map(String::from)
            } else {
                None
            };

            let mut sep_width = 80; // base width, automatically overridden by wrap width

            // Helper to build the seperator with a base string `sep` and a `width`
            let build_seperator = |sep: String, width: usize| -> String {
                sep.repeat(width / sep.len())
            };

            if rt.cli().is_present("compile-md") {
                let viewer    = MarkdownViewer::new(&rt);
                let seperator = basesep.map(|s| build_seperator(s, sep_width));

                entries
                    .enumerate()
                    .for_each(|(n, entry)| {
                        if n != 0 {
                            seperator
                                .as_ref()
                                .map(|s| writeln!(outlock, "{}", s).to_exit_code().unwrap_or_exit());
                        }

                        if let Err(e) = viewer.view_entry(&entry, &mut outlock) {
                            handle_error(e);
                        }

                        rt.report_touched(entry.get_location()).unwrap_or_exit();
                    });
            } else {
                let mut viewer = StdoutViewer::new(view_header, !hide_content);

                if rt.cli().occurrences_of("autowrap") != 0 {
                    let width = rt.cli().value_of("autowrap").unwrap(); // ensured by clap
                    let width = usize::from_str(width).unwrap_or_else(|e| {
                        error!("Failed to parse argument to number: autowrap = {:?}",
                               rt.cli().value_of("autowrap").map(String::from));
                        error!("-> {:?}", e);
                        ::std::process::exit(1)
                    });

                    // Copying this value over, so that the seperator has the right len as well
                    sep_width = width;

                    viewer.wrap_at(width);
                }

                let seperator = basesep.map(|s| build_seperator(s, sep_width));
                entries
                    .enumerate()
                    .for_each(|(n, entry)| {
                        if n != 0 {
                            seperator
                                .as_ref()
                                .map(|s| writeln!(outlock, "{}", s).to_exit_code().unwrap_or_exit());
                        }

                        if let Err(e) = viewer.view_entry(&entry, &mut outlock) {
                            handle_error(e);
                        }

                        rt.report_touched(entry.get_location()).unwrap_or_exit();
                    });
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
        "View entries (readonly)"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

fn create_tempfile_for<'a>(entry: &FileLockEntry<'a>, view_header: bool, hide_content: bool)
    -> (tempfile::NamedTempFile, String)
{
    let mut tmpfile = tempfile::NamedTempFile::new()
        .map_err(Error::from)
        .map_err_trace_exit_unwrap();

    if view_header {
        let hdr = toml::ser::to_string_pretty(entry.get_header())
            .map_err(Error::from)
            .map_err_trace_exit_unwrap();
        let _ = tmpfile.write(format!("---\n{}---\n", hdr).as_bytes())
            .map_err(Error::from)
            .map_err_trace_exit_unwrap();
    }

    if !hide_content {
        let _ = tmpfile.write(entry.get_content().as_bytes())
            .map_err(Error::from)
            .map_err_trace_exit_unwrap();
    }

    let file_path = tmpfile
        .path()
        .to_str()
        .map(String::from)
        .ok_or_else(|| Error::from(err_msg("Cannot build path")))
        .map_err_trace_exit_unwrap();

    (tmpfile, file_path)
}

fn handle_error(e: ::libimagentryview::error::Error) {
    use libimagentryview::error::Error;
    match e {
        Error::Io(e)    => Err(e).to_exit_code().unwrap_or_exit(),
        Error::Other(e) => Err(e).map_err_trace_exit_unwrap()
    }
}
