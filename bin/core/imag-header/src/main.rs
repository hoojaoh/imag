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
extern crate filters;
extern crate failure;

extern crate libimagentryedit;
extern crate libimagerror;
#[macro_use] extern crate libimagrt;
extern crate libimagstore;
extern crate libimagutil;

use std::io::Write;
use std::str::FromStr;

use clap::ArgMatches;
use filters::filter::Filter;
use failure::Error;

use libimagerror::exit::ExitCode;
use libimagerror::io::ToExitCode;
use libimagerror::iter::TraceIterator;
use libimagerror::trace::MapErrTrace;
use libimagrt::runtime::Runtime;
use libimagrt::setup::generate_runtime_setup;
use libimagstore::iter::get::StoreIdGetIteratorExtension;
use libimagstore::store::FileLockEntry;
use libimagstore::storeid::StoreIdIterator;

use toml_query::read::TomlValueReadExt;

mod ui;

fn main() {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-header",
                                    &version,
                                    "Plumbing tool for reading/writing structured data in entries",
                                    ui::build_ui);

    let list_output_with_ids     = rt.cli().is_present("list-id");
    let list_output_with_ids_fmt = rt.cli().value_of("list-id-format");

    trace!("list_output_with_ids     = {:?}", list_output_with_ids );
    trace!("list_output_with_ids_fmt = {:?}", list_output_with_ids_fmt);

    let sids = rt
        .ids::<crate::ui::PathProvider>()
        .map_err_trace_exit_unwrap()
        .unwrap_or_else(|| {
            error!("No ids supplied");
            ::std::process::exit(1);
        })
        .into_iter();

    let iter = StoreIdIterator::new(Box::new(sids.into_iter().map(Ok)))
        .into_get_iter(rt.store())
        .trace_unwrap_exit()
        .filter_map(|x| x);

    let exit_code = match rt.cli().subcommand() {
        ("read", Some(mtch))   => read(&rt, mtch, iter),
        ("has", Some(mtch))    => has(&rt, mtch, iter),
        ("hasnt", Some(mtch))  => hasnt(&rt, mtch, iter),
        ("int", Some(mtch))    => int(&rt, mtch, iter),
        ("float", Some(mtch))  => float(&rt, mtch, iter),
        ("string", Some(mtch)) => string(&rt, mtch, iter),
        ("bool", Some(mtch))   => boolean(&rt, mtch, iter),
        (other, _mtchs) => {
            debug!("Unknown command");
            rt.handle_unknown_subcommand("imag-header", other, rt.cli())
                .map_err_trace_exit_unwrap()
                .code()
                .unwrap_or(1)
        },
    };

    ::std::process::exit(exit_code)
}

fn read<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> i32
    where I: Iterator<Item = FileLockEntry<'e>>
{
    debug!("Processing headers: reading value");
    let header_path = get_header_path(mtch, "header-value-path");
    let mut output = rt.stdout();

    iter.fold(0, |accu, entry| {
        trace!("Processing headers: working on {:?}", entry.get_location());
        entry.get_header()
            .read(header_path)
            .map_err(Error::from)
            .map_err_trace_exit_unwrap()
            .map(|value| {
                trace!("Processing headers: Got value {:?}", value);
                writeln!(output, "{}", value)
                    .to_exit_code()
                    .map(|_| accu)
                    .unwrap_or_else(ExitCode::code)
            })
            .unwrap_or_else(|| {
                // if value not present and configured
                error!("Value not present for entry {} at {}", entry.get_location(), header_path);
                1
            })
    })
}

fn has<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> i32
    where I: Iterator<Item = FileLockEntry<'e>>
{
    debug!("Processing headers: has value");
    let header_path = get_header_path(mtch, "header-value-path");
    let mut output = rt.stdout();

    iter.fold(0, |accu, entry| {
        trace!("Processing headers: working on {:?}", entry.get_location());
        let value = entry.get_header()
            .read(header_path)
            .map_err(Error::from)
            .map_err_trace_exit_unwrap()
            .is_some();

        writeln!(output, "{} - {}", entry.get_location(), value)
            .to_exit_code()
            .map(|_| if value { accu } else { 1 })
            .unwrap_or_else(ExitCode::code)
    })
}

fn hasnt<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> i32
    where I: Iterator<Item = FileLockEntry<'e>>
{
    debug!("Processing headers: hasnt value");
    let header_path = get_header_path(mtch, "header-value-path");
    let mut output = rt.stdout();

    iter.fold(0, |accu, entry| {
        trace!("Processing headers: working on {:?}", entry.get_location());
        let value = entry.get_header()
            .read(header_path)
            .map_err(Error::from)
            .map_err_trace_exit_unwrap()
            .is_none();

        writeln!(output, "{} - {}", entry.get_location(), value)
            .to_exit_code()
            .map(|_| if value { accu } else { 1 })
            .unwrap_or_else(ExitCode::code)
    })
}

macro_rules! implement_compare {
    { $mtch: ident, $path: expr, $t: ty, $compare: expr } => {{
        trace!("Getting value at {}, comparing as {}", $path, stringify!($t));
        if let Some(cmp) = $mtch.value_of($path).map(FromStr::from_str) {
            let cmp : $t = cmp.unwrap(); // safe by clap
            trace!("Getting value at {} = {}", $path, cmp);
            $compare(cmp)
        } else {
            true
        }
    }}
}

fn int<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> i32
    where I: Iterator<Item = FileLockEntry<'e>>
{
    debug!("Processing headers: int value");
    let header_path = get_header_path(mtch, "header-value-path");
    let mut output = rt.stdout();

    let filter = ::filters::ops::bool::Bool::new(true)
        .and(|i: &i64| -> bool {
            implement_compare!(mtch, "header-int-eq", i64, |cmp| *i == cmp)
        })
        .and(|i: &i64| -> bool {
            implement_compare!(mtch, "header-int-neq", i64, |cmp| *i != cmp)
        })
        .and(|i: &i64| -> bool {
            implement_compare!(mtch, "header-int-lt", i64, |cmp| *i < cmp)
        })
        .and(|i: &i64| -> bool {
            implement_compare!(mtch, "header-int-gt", i64, |cmp| *i > cmp)
        })
        .and(|i: &i64| -> bool {
            implement_compare!(mtch, "header-int-lte", i64, |cmp| *i <= cmp)
        })
        .and(|i: &i64| -> bool {
            implement_compare!(mtch, "header-int-gte", i64, |cmp| *i >= cmp)
        });

    iter.fold(0, |accu, entry| {
        trace!("Processing headers: working on {:?}", entry.get_location());
        if let Some(v) = entry.get_header()
            .read(header_path)
            .map_err(Error::from)
            .map_err_trace_exit_unwrap()
        {
            match v {
                ::toml::Value::Integer(i) => if filter.filter(&i) {
                    writeln!(output, "{} - {}", entry.get_location(), i)
                        .to_exit_code()
                        .map(|_| accu)
                        .unwrap_or_else(ExitCode::code)
                } else { 1 },
                _ => 1
            }
        } else { 1 }
    })
}

fn float<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> i32
    where I: Iterator<Item = FileLockEntry<'e>>
{
    debug!("Processing headers: float value");
    let header_path = get_header_path(mtch, "header-value-path");
    let mut output = rt.stdout();

    let filter = ::filters::ops::bool::Bool::new(true)
        .and(|i: &f64| -> bool {
            implement_compare!(mtch, "header-float-eq", f64, |cmp| *i == cmp)
        })
        .and(|i: &f64| -> bool {
            implement_compare!(mtch, "header-float-neq", f64, |cmp| *i != cmp)
        })
        .and(|i: &f64| -> bool {
            implement_compare!(mtch, "header-float-lt", f64, |cmp| *i < cmp)
        })
        .and(|i: &f64| -> bool {
            implement_compare!(mtch, "header-float-gt", f64, |cmp| *i > cmp)
        })
        .and(|i: &f64| -> bool {
            implement_compare!(mtch, "header-float-lte", f64, |cmp| *i <= cmp)
        })
        .and(|i: &f64| -> bool {
            implement_compare!(mtch, "header-float-gte", f64, |cmp| *i >= cmp)
        });

    iter.fold(0, |accu, entry| {
        trace!("Processing headers: working on {:?}", entry.get_location());
        if let Some(v) = entry.get_header()
            .read(header_path)
            .map_err(Error::from)
            .map_err_trace_exit_unwrap()
        {
            match v {
                ::toml::Value::Float(i) => if filter.filter(&i) {
                    writeln!(output, "{} - {}", entry.get_location(), i)
                        .to_exit_code()
                        .map(|_| accu)
                        .unwrap_or_else(ExitCode::code)
                } else { 1 },
                _ => 1
            }
        } else { 1 }
    })
}

fn string<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> i32
    where I: Iterator<Item = FileLockEntry<'e>>
{
    debug!("Processing headers: string value");
    let header_path = get_header_path(mtch, "header-value-path");
    let mut output = rt.stdout();

    let filter = ::filters::ops::bool::Bool::new(true)
        .and(|i: &String| -> bool {
            implement_compare!(mtch, "header-string-eq", String, |cmp| *i == cmp)
        })
        .and(|i: &String| -> bool {
            implement_compare!(mtch, "header-string-neq", String, |cmp| *i != cmp)
        });

    iter.fold(0, |accu, entry| {
        trace!("Processing headers: working on {:?}", entry.get_location());
        if let Some(v) = entry.get_header()
            .read(header_path)
            .map_err(Error::from)
            .map_err_trace_exit_unwrap()
        {
            match v {
                ::toml::Value::String(s) => if filter.filter(&s) {
                    writeln!(output, "{} - {}", entry.get_location(), s)
                        .to_exit_code()
                        .map(|_| accu)
                        .unwrap_or_else(ExitCode::code)
                } else { 1 },
                _ => 1
            }
        } else { 1 }
    })
}

fn boolean<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> i32
    where I: Iterator<Item = FileLockEntry<'e>>
{
    debug!("Processing headers: bool value");
    let header_path = get_header_path(mtch, "header-value-path");
    let mut output = rt.stdout();

    let filter = ::filters::ops::bool::Bool::new(true)
        .and(|i: &bool| -> bool { *i })
        .and(|i: &bool| -> bool { *i });

    iter.fold(0, |accu, entry| {
        trace!("Processing headers: working on {:?}", entry.get_location());
        if let Some(v) = entry.get_header()
            .read(header_path)
            .map_err(Error::from)
            .map_err_trace_exit_unwrap()
        {
            match v {
                ::toml::Value::Boolean(b) => if filter.filter(&b) {
                    writeln!(output, "{} - {}", entry.get_location(), b)
                        .to_exit_code()
                        .map(|_| accu)
                        .unwrap_or_else(ExitCode::code)
                } else { 1 },
                _ => 1
            }
        } else { 1 }
    })
}



// helpers
//
fn get_header_path<'a>(mtch: &'a ArgMatches<'a>, path: &'static str) -> &'a str {
    let header_path = mtch.value_of(path).unwrap(); // safe by clap
    debug!("Processing headers: header path = {}", header_path);
    header_path
}

