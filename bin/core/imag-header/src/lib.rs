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
#[macro_use] extern crate failure;
extern crate toml;
extern crate toml_query;
extern crate filters;
extern crate resiter;

extern crate libimagentryedit;
extern crate libimagerror;
extern crate libimagrt;
extern crate libimagstore;
extern crate libimagutil;

use std::io::Write;
use std::str::FromStr;
use std::string::ToString;

use clap::{App, ArgMatches};
use filters::filter::Filter;
use toml::Value;
use failure::Error;
use failure::Fallible as Result;
use failure::err_msg;
use resiter::FilterMap;
use resiter::AndThen;
use resiter::IterInnerOkOrElse;

use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
use libimagstore::iter::get::StoreIdGetIteratorExtension;
use libimagstore::store::FileLockEntry;

use toml_query::read::TomlValueReadExt;
use toml_query::read::TomlValueReadTypeExt;

mod ui;

const EPS_CMP: f64 = 1e-10;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagHeader {}
impl ImagApplication for ImagHeader {
    fn run(rt: Runtime) -> Result<()> {
        let list_output_with_ids     = rt.cli().is_present("list-id");
        let list_output_with_ids_fmt = rt.cli().value_of("list-id-format");

        trace!("list_output_with_ids     = {:?}", list_output_with_ids );
        trace!("list_output_with_ids_fmt = {:?}", list_output_with_ids_fmt);

        let iter = rt
            .ids::<crate::ui::PathProvider>()?
            .ok_or_else(|| err_msg("No ids supplied"))?
            .into_iter()
            .map(Ok)
            .into_get_iter(rt.store())
            .map_inner_ok_or_else(|| err_msg("Did not find one entry"));

        match rt.cli().subcommand() {
            ("read", Some(mtch))   => read(&rt, mtch, iter),
            ("has", Some(mtch))    => has(&rt, mtch, iter),
            ("hasnt", Some(mtch))  => hasnt(&rt, mtch, iter),
            ("int", Some(mtch))    => int(&rt, mtch, iter),
            ("float", Some(mtch))  => float(&rt, mtch, iter),
            ("string", Some(mtch)) => string(&rt, mtch, iter),
            ("bool", Some(mtch))   => boolean(&rt, mtch, iter),
            (other, _mtchs) => {
                debug!("Unknown command");
                if rt.handle_unknown_subcommand("imag-header", other, rt.cli())
                    .map_err(Error::from)?
                    .success()
                {
                    Ok(())
                } else {
                    Err(format_err!("Subcommand failed"))
                }
            },
        }
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Plumbing tool for reading/writing structured data in entries"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

fn read<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> Result<()>
    where I: Iterator<Item = Result<FileLockEntry<'e>>>
{
    debug!("Processing headers: reading value");
    let header_path = get_header_path(mtch, "header-value-path");
    let mut output = rt.stdout();
    trace!("Got output: {:?}", output);

    iter.and_then_ok(|entry| {
        trace!("Processing headers: working on {:?}", entry.get_location());
        entry.get_header()
            .read(header_path)?
            .ok_or_else(|| format_err!("Value not present for entry {} at {}", entry.get_location(), header_path))
            .and_then(|value| {
                trace!("Processing headers: Got value {:?}", value);

                let string_representation = match value {
                    Value::String(s)  => Some(s.to_owned()),
                    Value::Integer(i) => Some(i.to_string()),
                    Value::Float(f)   => Some(f.to_string()),
                    Value::Boolean(b) => Some(b.to_string()),
                    _ => None,
                };

                if let Some(repr) = string_representation {
                    writeln!(output, "{}", repr)?;
                } else {
                    writeln!(output, "{}", value)?;
                }
                Ok(())
            })
    })
    .collect::<Result<()>>()
}

fn has<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> Result<()>
    where I: Iterator<Item = Result<FileLockEntry<'e>>>
{
    debug!("Processing headers: has value");
    let header_path = get_header_path(mtch, "header-value-path");
    let mut output = rt.stdout();

    iter.and_then_ok(|entry| {
        trace!("Processing headers: working on {:?}", entry.get_location());
        if let Some(_) = entry.get_header().read(header_path)?  {
            if !rt.output_is_pipe() {
                writeln!(output, "{}", entry.get_location())?;
            }
            rt.report_touched(entry.get_location()).map_err(Error::from)
        } else {
            Ok(())
        }
    })
    .collect::<Result<()>>()
}

fn hasnt<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> Result<()>
    where I: Iterator<Item = Result<FileLockEntry<'e>>>
{
    debug!("Processing headers: hasnt value");
    let header_path = get_header_path(mtch, "header-value-path");
    let mut output = rt.stdout();

    iter.and_then_ok(|entry| {
        trace!("Processing headers: working on {:?}", entry.get_location());
        if let Some(_) = entry.get_header().read(header_path)? {
            Ok(())
        } else {
            if !rt.output_is_pipe() {
                writeln!(output, "{}", entry.get_location())?;
            }
            rt.report_touched(entry.get_location()).map_err(Error::from)
        }
    })
    .collect()
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

fn int<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> Result<()>
    where I: Iterator<Item = Result<FileLockEntry<'e>>>
{
    debug!("Processing headers: int value");
    let header_path = get_header_path(mtch, "header-value-path");

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

    iter.and_then_ok(|entry| {
        if let Some(hdr) = entry.get_header().read_int(header_path)?  {
            Ok((filter.filter(&hdr), entry))
        } else {
            Ok((false, entry))
        }
    })
    .filter_map_ok(|(b, e)| if b { Some(e) } else { None })
    .and_then_ok(|entry| rt.report_touched(entry.get_location()).map_err(Error::from))
    .collect()
}

fn float<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> Result<()>
    where I: Iterator<Item = Result<FileLockEntry<'e>>>
{
    debug!("Processing headers: float value");
    let header_path = get_header_path(mtch, "header-value-path");

    let filter = ::filters::ops::bool::Bool::new(true)
        .and(|i: &f64| -> bool {
            implement_compare!(mtch, "header-float-eq", f64, |cmp: f64| (*i - cmp).abs() < EPS_CMP)
        })
        .and(|i: &f64| -> bool {
            implement_compare!(mtch, "header-float-neq", f64, |cmp: f64| (*i - cmp).abs() > EPS_CMP)
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

    iter.and_then_ok(|entry| {
        if let Some(hdr) = entry.get_header().read_float(header_path)? {
            Ok((filter.filter(&hdr), entry))
        } else {
            Ok((false, entry))
        }
    })
    .filter_map_ok(|(b, e)| if b { Some(e) } else { None })
    .and_then_ok(|entry| rt.report_touched(entry.get_location()).map_err(Error::from))
    .collect()
}

fn string<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> Result<()>
    where I: Iterator<Item = Result<FileLockEntry<'e>>>
{
    debug!("Processing headers: string value");
    let header_path = get_header_path(mtch, "header-value-path");

    let filter = ::filters::ops::bool::Bool::new(true)
        .and(|i: &String| -> bool {
            implement_compare!(mtch, "header-string-eq", String, |cmp| *i == cmp)
        })
        .and(|i: &String| -> bool {
            implement_compare!(mtch, "header-string-neq", String, |cmp| *i != cmp)
        });

    iter.and_then_ok(|entry| {
        if let Some(hdr) = entry.get_header().read_string(header_path)?  {
            Ok((filter.filter(&hdr), entry))
        } else {
            Ok((false, entry))
        }
    })
    .filter_map_ok(|(b, e)| if b { Some(e) } else { None })
    .and_then_ok(|entry| rt.report_touched(entry.get_location()).map_err(Error::from))
    .collect()
}

fn boolean<'a, 'e, I>(rt: &Runtime, mtch: &ArgMatches<'a>, iter: I) -> Result<()>
    where I: Iterator<Item = Result<FileLockEntry<'e>>>
{
    debug!("Processing headers: bool value");
    let header_path = get_header_path(mtch, "header-value-path");

    let filter = ::filters::ops::bool::Bool::new(true)
        .and(|i: &bool| -> bool { *i })
        .and(|i: &bool| -> bool { *i });

    iter.and_then_ok(|entry| {
        if let Some(hdr) = entry.get_header().read_bool(header_path)?  {
            Ok((filter.filter(&hdr), entry))
        } else {
            Ok((false, entry))
        }
    })
    .filter_map_ok(|(b, e)| if b { Some(e) } else { None })
    .and_then_ok(|entry| rt.report_touched(entry.get_location()).map_err(Error::from))
    .collect()
}



// helpers
//
fn get_header_path<'a>(mtch: &'a ArgMatches<'a>, path: &'static str) -> &'a str {
    let header_path = mtch.value_of(path).unwrap(); // safe by clap
    debug!("Processing headers: header path = {}", header_path);
    header_path
}

