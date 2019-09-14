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
#[macro_use] extern crate is_match;
#[macro_use] extern crate log;
extern crate toml;
extern crate toml_query;
extern crate itertools;
extern crate failure;
extern crate textwrap;

extern crate libimaglog;
extern crate libimagrt;
extern crate libimagstore;
extern crate libimagerror;
extern crate libimagdiary;

use std::io::Write;
use std::io::Cursor;
use std::result::Result as RResult;
use std::str::FromStr;

use failure::Error;
use failure::err_msg;
use failure::Fallible as Result;

use libimagrt::application::ImagApplication;
use libimagrt::runtime::Runtime;
use libimagerror::trace::MapErrTrace;
use libimagerror::io::ToExitCode;
use libimagerror::exit::ExitUnwrap;
use libimagerror::iter::TraceIterator;
use libimagerror::exit::ExitCode;
use libimagdiary::diary::Diary;
use libimagdiary::diaryid::DiaryId;
use libimaglog::log::Log;
use libimagstore::iter::get::StoreIdGetIteratorExtension;
use libimagstore::store::FileLockEntry;

use clap::App;

mod ui;

use toml::Value;
use itertools::Itertools;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagLog {}
impl ImagApplication for ImagLog {
    fn run(rt: Runtime) -> Result<()> {
        if let Some(scmd) = rt.cli().subcommand_name() {
            match scmd {
                "show" => show(&rt),
                other    => {
                    debug!("Unknown command");
                    let _ = rt.handle_unknown_subcommand("imag-log", other, rt.cli())
                        .map_err_trace_exit_unwrap()
                        .code()
                        .map(::std::process::exit);
                },
            }
        } else {
            let text       = get_log_text(&rt);
            let diary_name = rt.cli()
                .value_of("diaryname")
                .map(String::from)
                .unwrap_or_else(|| get_diary_name(&rt));

            debug!("Writing to '{}': {}", diary_name, text);

            rt
                .store()
                .new_entry_now(&diary_name)
                .map(|mut fle| {
                    fle.make_log_entry().map_err_trace_exit_unwrap();
                    *fle.get_content_mut() = text;
                    fle
                })
                .map(|fle| rt.report_touched(fle.get_location()).unwrap_or_exit())
                .map_err_trace_exit_unwrap();

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
        "Overlay to imag-diary to 'log' single lines of text"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

fn show(rt: &Runtime) {
    use std::borrow::Cow;

    use libimagdiary::iter::DiaryEntryIterator;
    use libimagdiary::entry::DiaryEntry;

    let scmd = rt.cli().subcommand_matches("show").unwrap(); // safed by main()
    let iters : Vec<DiaryEntryIterator> = match scmd.values_of("show-name") {
        Some(values) => values
            .map(|diary_name| Diary::entries(rt.store(), diary_name).map_err_trace_exit_unwrap())
            .collect(),

        None => if scmd.is_present("show-all") {
            debug!("Showing for all diaries");
            rt.store()
                .diary_names()
                .map_err_trace_exit_unwrap()
                .map(|diary_name| {
                    let diary_name = diary_name.map_err_trace_exit_unwrap();
                    debug!("Getting entries for Diary: {}", diary_name);
                    let entries = Diary::entries(rt.store(), &diary_name).map_err_trace_exit_unwrap();
                    let diary_name = Cow::from(diary_name);
                    (entries, diary_name)
                })
                .unique_by(|tpl| tpl.1.clone())
                .map(|tpl| tpl.0)
                .collect()
        } else {
            // showing default logs
            vec![Diary::entries(rt.store(), &get_diary_name(rt)).map_err_trace_exit_unwrap()]
        }
    };

    let mut do_wrap = if scmd.is_present("show-wrap") {
        Some(80)
    } else {
        None
    };
    let do_remove_newlines = scmd.is_present("show-skipnewlines");

    if let Some(wrap_value) = scmd.value_of("show-wrap") {
        do_wrap = Some(usize::from_str(wrap_value).map_err(Error::from).map_err_trace_exit_unwrap());
    }

    let mut output = rt.stdout();

    iters.into_iter()
        .flatten()
        .into_get_iter(rt.store())
        .trace_unwrap_exit()
        .filter_map(|opt| {
            if opt.is_none() {
                warn!("Failed to retrieve an entry from an existing store id");
            }

            opt
        })
        .filter(|e| e.is_log().map_err_trace_exit_unwrap())
        .map(|entry| (entry.diary_id().map_err_trace_exit_unwrap(), entry))
        .sorted_by_key(|tpl| tpl.0.get_date_representation())
        .map(|tpl| { debug!("Found entry: {:?}", tpl.1); tpl })
        .map(|(id, entry)| {
            if let Some(wrap_limit) = do_wrap {
                // assume a capacity here:
                // diaryname + year + month + day + hour + minute + delimiters + whitespace
                // 10 + 4 + 2 + 2 + 2 + 2 + 6 + 4 = 32
                // plus text, which we assume to be 120 characters... lets allocate 256 bytes.
                let mut buffer = Cursor::new(Vec::with_capacity(256));
                do_write_to(&mut buffer, id, &entry, do_remove_newlines).unwrap_or_exit();
                let buffer = String::from_utf8(buffer.into_inner())
                    .map_err(Error::from)
                    .map_err_trace_exit_unwrap();

                // now lets wrap
                for line in ::textwrap::wrap(&buffer, wrap_limit).iter() {
                    writeln!(&mut output, "{}", line).to_exit_code()?;
                }
            } else {
                do_write_to(&mut output, id, &entry, do_remove_newlines).unwrap_or_exit();
            }

            rt
                .report_touched(entry.get_location())
                .unwrap_or_exit();
            Ok(())
        })
        .collect::<RResult<Vec<()>, ExitCode>>()
        .unwrap_or_exit();
}

fn get_diary_name(rt: &Runtime) -> String {
    use toml_query::read::TomlValueReadExt;
    use toml_query::read::TomlValueReadTypeExt;

    let cfg = rt
        .config()
        .ok_or_else(|| err_msg("Configuration not present, cannot continue"))
        .map_err_trace_exit_unwrap();

    let current_log = cfg
        .read_string("log.default")
        .map_err(Error::from)
        .map_err_trace_exit_unwrap()
        .ok_or_else(|| err_msg("Configuration missing: 'log.default'"))
        .map_err_trace_exit_unwrap();

    if cfg
        .read("log.logs")
        .map_err(Error::from)
        .map_err_trace_exit_unwrap()
        .ok_or_else(|| err_msg("Configuration missing: 'log.logs'"))
        .map_err_trace_exit_unwrap()
        .as_array()
        .ok_or_else(|| err_msg("Configuration 'log.logs' is not an Array"))
        .map_err_trace_exit_unwrap()
        .iter()
        .map(|e| if !is_match!(e, &Value::String(_)) {
            error!("Configuration 'log.logs' is not an Array<String>!");
            ::std::process::exit(1)
        } else {
            e
        })
        .map(Value::as_str)
        .map(Option::unwrap) // safe by map from above
        .map(String::from)
        .find(|log| log == &current_log)
        .is_none()
    {
        error!("'log.logs' does not contain 'log.default'");
        ::std::process::exit(1)
    } else {
        current_log
    }
}

fn get_log_text(rt: &Runtime) -> String {
    rt.cli()
        .values_of("text")
        .unwrap() // safe by clap
        .enumerate()
        .fold(String::with_capacity(500), |mut acc, (n, e)| {
            if n != 0 {
                acc.push_str(" ");
            }
            acc.push_str(e);
            acc
        })
}

fn do_write_to<'a>(sink: &mut dyn Write, id: DiaryId, entry: &FileLockEntry<'a>, do_remove_newlines: bool) -> RResult<(), ExitCode> {
    let text = if do_remove_newlines {
        entry.get_content().trim_end().replace("\n", "")
    } else {
        entry.get_content().trim_end().to_string()
    };

    writeln!(sink,
            "{dname: >10} - {y: >4}-{m:0>2}-{d:0>2}T{H:0>2}:{M:0>2} - {text}",
             dname = id.diary_name(),
             y = id.year(),
             m = id.month(),
             d = id.day(),
             H = id.hour(),
             M = id.minute(),
             text = text)
        .to_exit_code()
}

