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

#[macro_use] extern crate log;
#[macro_use] extern crate failure;
extern crate clap;
extern crate regex;
extern crate resiter;

extern crate libimagstore;
extern crate libimagrt;
extern crate libimagerror;

use std::io::Write;

use regex::Regex;
use clap::App;
use failure::Error;
use failure::Fallible as Result;
use failure::err_msg;
use resiter::AndThen;

use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
use libimagstore::store::Entry;
use libimagerror::iter::IterInnerOkOrElse;


mod ui;

struct Options {
    files_with_matches: bool,
    count: bool,
}

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagGrep {}
impl ImagApplication for ImagGrep {
    fn run(rt: Runtime) -> Result<()> {
        let opts = Options {
            files_with_matches    : rt.cli().is_present("files-with-matches"),
            count                 : rt.cli().is_present("count"),
        };

        let mut count : usize = 0;

        let pattern = rt
            .cli()
            .value_of("pattern")
            .map(Regex::new)
            .unwrap() // ensured by clap
            .map_err(|e| format_err!("Regex building error: {:?}", e))?;

        let overall_count = rt
            .store()
            .entries()?
            .into_get_iter()
            .map_inner_ok_or_else(|| err_msg("Entry from entries missing"))
            .and_then_ok(|entry| {
                if pattern.is_match(entry.get_content()) {
                    debug!("Matched: {}", entry.get_location());
                    show(&rt, &entry, &pattern, &opts, &mut count)
                } else {
                    debug!("Not matched: {}", entry.get_location());
                    Ok(())
                }
            })
            .collect::<Result<Vec<_>>>()?
            .len();

        if opts.count {
            writeln!(rt.stdout(), "{}", count)?;
        } else if !opts.files_with_matches {
            writeln!(rt.stdout(), "Processed {} files, {} matches, {} nonmatches",
                     overall_count,
                     count,
                     overall_count - count)?;
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
        "grep through entries text"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

}

fn show(rt: &Runtime, e: &Entry, re: &Regex, opts: &Options, count: &mut usize) -> Result<()> {
    if opts.files_with_matches {
        writeln!(rt.stdout(), "{}", e.get_location())?;
    } else if opts.count {
        *count += 1;
    } else {
        writeln!(rt.stdout(), "{}:", e.get_location())?;
        for capture in re.captures_iter(e.get_content()) {
            for mtch in capture.iter() {
                if let Some(m) = mtch {
                    writeln!(rt.stdout(), " '{}'", m.as_str())?;
                }
            }
        }

        writeln!(rt.stdout())?;
        *count += 1;
    }

    rt.report_touched(e.get_location()).map_err(Error::from)
}

