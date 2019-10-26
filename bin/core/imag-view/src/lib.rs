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
#[macro_use] extern crate failure;
extern crate resiter;

extern crate libimagentryview;
extern crate libimagerror;
extern crate libimagrt;
extern crate libimagstore;
extern crate libimagutil;

use std::str::FromStr;
use std::collections::BTreeMap;
use std::io::Write;
use std::process::Command;

use handlebars::Handlebars;
use toml_query::read::TomlValueReadTypeExt;
use failure::Error;
use failure::err_msg;
use failure::Fallible as Result;
use resiter::AndThen;
use resiter::IterInnerOkOrElse;
use clap::App;

use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
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
            .ids::<::ui::PathProvider>()?
            .ok_or_else(|| err_msg("No ids supplied"))?
            .into_iter()
            .map(Ok)
            .into_get_iter(rt.store())
            .map_inner_ok_or_else(|| err_msg("Entry not found, please report this as a bug"));

        if rt.cli().is_present("in") {
            let files = entries
                .and_then_ok(|entry| {
                    let tmpfile = create_tempfile_for(&entry, view_header, hide_content)?;
                    rt.report_touched(entry.get_location())?;
                    Ok(tmpfile)
                })
                .collect::<Result<Vec<_>>>()?;

            let mut command = {
                let viewer = rt
                    .cli()
                    .value_of("in")
                    .ok_or_else(|| err_msg("No viewer given"))?;

                let config = rt
                    .config()
                    .ok_or_else(|| err_msg("No configuration, cannot continue"))?;

                let query = format!("view.viewers.{}", viewer);

                let viewer_template = config
                    .read_string(&query)?
                    .ok_or_else(|| format_err!("Cannot find '{}' in config", query))?;

                let mut handlebars = Handlebars::new();
                handlebars.register_escape_fn(::handlebars::no_escape);

                handlebars.register_template_string("template", viewer_template)?;

                let mut data = BTreeMap::new();

                let file_paths = files
                    .iter()
                    .map(|&(_, ref path)| path.clone())
                    .collect::<Vec<String>>()
                    .join(" ");

                data.insert("entries", file_paths);

                let call = handlebars .render("template", &data)?;
                let mut elems = call.split_whitespace();
                let command_string = elems.next().ok_or_else(|| err_msg("No command"))?;
                let mut cmd = Command::new(command_string);

                for arg in elems {
                    cmd.arg(arg);
                }

                cmd
            };

            debug!("Calling: {:?}", command);

            if !command
                .status()?
                .success()
            {
                return Err(err_msg("Failed to execute command"))
            }

            drop(files);
            Ok(())
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

                let mut i = 0; // poor mans enumerate()

                entries.and_then_ok(|entry| {
                    if i != 0 {
                        if let Some(s) = seperator.as_ref() {
                            writeln!(outlock, "{}", s)?;
                        }
                    }

                    viewer.view_entry(&entry, &mut outlock)?;

                    i += 1;
                    rt.report_touched(entry.get_location()).map_err(Error::from)
                })
                .collect()
            } else {
                let mut viewer = StdoutViewer::new(view_header, !hide_content);

                if rt.cli().occurrences_of("autowrap") != 0 {
                    let width = rt.cli().value_of("autowrap").unwrap(); // ensured by clap
                    let width = usize::from_str(width).map_err(|_| {
                        format_err!("Failed to parse argument to number: autowrap = {:?}",
                               rt.cli().value_of("autowrap").map(String::from))
                    })?;

                    // Copying this value over, so that the seperator has the right len as well
                    sep_width = width;

                    viewer.wrap_at(width);
                }

                let seperator = basesep.map(|s| build_seperator(s, sep_width));
                let mut i = 0; // poor mans enumerate()
                entries.and_then_ok(|entry| {
                    if i != 0 {
                        if let Some(s) = seperator.as_ref() {
                            writeln!(outlock, "{}", s)?;
                        }
                    }

                    viewer.view_entry(&entry, &mut outlock)?;

                    i += 1;
                    rt.report_touched(entry.get_location()).map_err(Error::from)
                })
                .collect()
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
        "View entries (readonly)"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

fn create_tempfile_for<'a>(entry: &FileLockEntry<'a>, view_header: bool, hide_content: bool)
    -> Result<(tempfile::NamedTempFile, String)>
{
    let mut tmpfile = tempfile::NamedTempFile::new()?;

    if view_header {
        let hdr = toml::ser::to_string_pretty(entry.get_header())?;
        let _ = tmpfile.write(format!("---\n{}---\n", hdr).as_bytes())?;
    }

    if !hide_content {
        let _ = tmpfile.write(entry.get_content().as_bytes())?;
    }

    let file_path = tmpfile
        .path()
        .to_str()
        .map(String::from)
        .ok_or_else(|| Error::from(err_msg("Cannot build path")))?;

    Ok((tmpfile, file_path))
}

