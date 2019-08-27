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
extern crate toml;
extern crate toml_query;
extern crate indicatif;
extern crate failure;
#[macro_use] extern crate log;

#[macro_use] extern crate libimagrt;
extern crate libimagerror;
extern crate libimagentrylink;
extern crate libimagstore;

use std::io::Write;

use libimagrt::runtime::Runtime;
use libimagrt::setup::generate_runtime_setup;
use libimagerror::trace::MapErrTrace;
use libimagerror::io::ToExitCode;
use libimagerror::exit::ExitUnwrap;
use libimagstore::store::FileLockEntry;
use libimagstore::storeid::StoreId;
use libimagentrylink::linkable::Linkable;

use toml::Value;
use toml_query::read::TomlValueReadExt;
use indicatif::{ProgressBar, ProgressStyle};
use failure::Fallible as Result;
use failure::Error;
use failure::err_msg;

use std::collections::BTreeMap;

mod ui;

#[derive(Debug)]
struct Diagnostic {
    pub id: StoreId,
    pub entry_store_version: String,
    pub header_sections: usize,
    pub bytecount_content: usize,
    pub overall_byte_size: usize,
    pub verified: bool,
    pub num_links: usize,
}

impl Diagnostic {

    fn for_entry<'a>(entry: &FileLockEntry<'a>) -> Result<Diagnostic> {
        Ok(Diagnostic {
            id: entry.get_location().clone(),
            entry_store_version: entry
                .get_header()
                .read("imag.version")
                .map(|opt| match opt {
                    Some(&Value::String(ref s)) => s.clone(),
                    Some(_) => "Non-String type in 'imag.version'".to_owned(),
                    None => "No version".to_owned(),
                })
                .unwrap_or("Error reading version".to_owned()),
            header_sections: match entry.get_header() {
                &Value::Table(ref map) => map.keys().count(),
                _ => 0
            },
            bytecount_content: entry.get_content().as_str().len(),
            overall_byte_size: entry.to_str()?.as_str().len(),
            verified: entry.verify().is_ok(),
            num_links: entry.links().map(Iterator::count).unwrap_or(0),
        })
    }
}

macro_rules! do_write {
    ($dest:ident, $pattern:tt) => {
        let _ = writeln!($dest, $pattern)
            .to_exit_code()
            .unwrap_or_exit();
    };

    ($dest:ident, $pattern:tt, $( $args:expr ),*) => {
        let _ = writeln!($dest, $pattern, $( $args ),*)
            .to_exit_code()
            .unwrap_or_exit();
    }
}

fn main() {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-diagnostics",
                                    &version,
                                    "Print diagnostics about imag and the imag store",
                                    ui::build_ui);

    let template    = get_config(&rt, "rt.progressbar_style");
    let tick_chars  = get_config(&rt, "rt.progressticker_chars");
    let verbose     = rt.cli().is_present("more-output");

    let style = if let Some(tick_chars) = tick_chars {
        ProgressStyle::default_spinner().tick_chars(&tick_chars)
    } else {
        ProgressStyle::default_spinner()
    };

    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(100);
    spinner.set_style(style);
    spinner.set_message("Accumulating data");

    let diags = rt.store()
        .entries()
        .map_err_trace_exit_unwrap()
        .into_get_iter()
        .map(|e| {
            e.map_err_trace_exit_unwrap()
                .ok_or_else(|| err_msg("Unable to get entry".to_owned()))
                .map_err_trace_exit_unwrap()
        })
        .map(|e| {
            let diag = Diagnostic::for_entry(&e);
            debug!("Diagnostic for '{:?}' = {:?}", e.get_location(), diag);
            drop(e);
            diag
        })
        .collect::<Result<Vec<_>>>()
        .map_err_trace_exit_unwrap();

    spinner.finish();
    let n                = diags.len();
    let progress         = ProgressBar::new(n as u64);
    let style            = if let Some(template) = template {
        ProgressStyle::default_bar().template(&template)
    } else {
        ProgressStyle::default_bar()
    };
    progress.set_style(style);
    progress.set_message("Calculating stats");

    let mut version_counts        : BTreeMap<String, usize> = BTreeMap::new();
    let mut sum_header_sections   = 0;
    let mut sum_bytecount_content = 0;
    let mut sum_overall_byte_size = 0;
    let mut max_overall_byte_size : Option<(usize, StoreId)> = None;
    let mut verified_count        = 0;
    let mut unverified_count      = 0;
    let mut unverified_entries    = vec![];
    let mut num_links    = 0;
    let mut max_links : Option<(usize, StoreId)> = None;

    for diag in diags.iter() {
        sum_header_sections     += diag.header_sections;
        sum_bytecount_content   += diag.bytecount_content;
        sum_overall_byte_size   += diag.overall_byte_size;
        match max_overall_byte_size {
            None => max_overall_byte_size = Some((diag.num_links, diag.id.clone())),
            Some((num, _)) => if num < diag.overall_byte_size {
                max_overall_byte_size = Some((diag.overall_byte_size, diag.id.clone()));
            }
        }

        let n = version_counts.get(&diag.entry_store_version).map(Clone::clone).unwrap_or(0);
        version_counts.insert(diag.entry_store_version.clone(), n+1);

        if diag.verified {
            verified_count += 1;
        } else {
            unverified_count += 1;
            if verbose {
                unverified_entries.push(diag.id.clone());
            }
        }

        num_links += diag.num_links;
        match max_links {
            None => max_links = Some((diag.num_links, diag.id.clone())),
            Some((num, _)) => if num < diag.num_links {
                max_links = Some((diag.num_links, diag.id.clone()));
            }
        }

        progress.inc(1);
    }

    progress.finish();

    let mut out = rt.stdout();

    do_write!(out, "imag version {}", { env!("CARGO_PKG_VERSION") });
    do_write!(out, "");
    do_write!(out, "{} entries", n);

    for (k, v) in version_counts {
        do_write!(out, "{} entries with store version '{}'", v, k);
    }
    if n != 0 {
        do_write!(out, "{} header sections in the average entry", sum_header_sections / n);
        do_write!(out, "{} average content bytecount", sum_bytecount_content / n);
        do_write!(out, "{} average overall bytecount", sum_overall_byte_size / n);

        if let Some((num, path)) = max_overall_byte_size {
            do_write!(out, "Largest Entry ({} bytes): {}", num, path.local_display_string());
        }

        do_write!(out, "{} average internal link count per entry", num_links / n);

        if let Some((num, path)) = max_links {
            do_write!(out, "Entry with most internal links ({}): {}",
                     num,
                     path.local_display_string());
        }
        do_write!(out, "{} verified entries", verified_count);
        do_write!(out, "{} unverified entries", unverified_count);
        if verbose {
            for unve in unverified_entries.iter() {
                do_write!(out, "Unverified: {}", unve);
            }
        }
    }
}

fn get_config(rt: &Runtime, s: &'static str) -> Option<String> {
    rt.config().and_then(|cfg| {
        cfg.read(s)
            .map_err(Error::from)
            .map_err_trace_exit_unwrap()
            .map(|opt| match opt {
                &Value::String(ref s) => s.to_owned(),
                _ => {
                    error!("Config type wrong: 'rt.progressbar_style' should be a string");
                    ::std::process::exit(1)
                }
            })
    })
}

