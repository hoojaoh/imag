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
extern crate handlebars;
extern crate chrono;
extern crate kairos;

extern crate libimagrt;
extern crate libimagcalendar;
extern crate libimagerror;
extern crate libimagstore;
extern crate libimagutil;

use std::path::PathBuf;
use std::result::Result as RResult;
use std::io::Write;

use failure::Error;
use failure::err_msg;
use failure::Fallible as Result;
use toml_query::read::Partial;
use toml_query::read::TomlValueReadExt;
use walkdir::DirEntry;
use walkdir::WalkDir;
use vobject::icalendar::Event;
use clap::App;

use libimagcalendar::store::EventStore;
use libimagerror::io::ToExitCode;
use libimagerror::exit::ExitUnwrap;
use libimagerror::iter::TraceIterator;
use libimagerror::trace::MapErrTrace;
use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;

mod filters;
mod ui;
mod util;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binary crates to execute business logic or to
/// build a CLI completion.
pub enum ImagCalendar {}
impl ImagApplication for ImagCalendar {
    fn run(rt: Runtime) -> Result<()> {
        if let Some(name) = rt.cli().subcommand_name() {
            debug!("Call {}", name);
            match name {
                "import" => import(&rt),
                "list"   => list(&rt),
                "show"   => show(&rt),
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

        Ok(())
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Calendar management tool"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
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

    // sanity check
    debug!("Doing sanity check on config, to see whether the configuration required for importing is there");
    if ref_config.get(collection_name).is_none() {
        error!("Configuration missing: {}.{}", libimagentryref::reference::Config::LOCATION, collection_name);
        ::std::process::exit(1);
    }

    debug!("Starting import...");
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
                            let path = fe.into_path();
                            trace!("Found file: {}", path.display());
                            Some(Ok(path))
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
            trace!("Importing {}", path.display());
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

fn list(rt: &Runtime) {
    use util::*;

    let scmd        = rt.cli().subcommand_matches("list").unwrap(); // safe by clap
    let list_format = get_event_print_format("calendar.list_format", rt, &scmd)
        .map_err_trace_exit_unwrap();

    let do_filter_past   = !scmd.is_present("list-past");
    let do_filter_before = scmd.value_of("list-before");
    let do_filter_after  = scmd.value_of("list-after");

    let ref_config      = rt.config()
        .ok_or_else(|| format_err!("No configuration, cannot continue!"))
        .map_err_trace_exit_unwrap()
        .read_partial::<libimagentryref::reference::Config>()
        .map_err(Error::from)
        .map_err_trace_exit_unwrap()
        .ok_or_else(|| format_err!("Configuration missing: {}", libimagentryref::reference::Config::LOCATION))
        .map_err_trace_exit_unwrap();


    debug!("List format: {:?}", list_format);
    debug!("Ref config : {:?}", ref_config);
    let today = ::chrono::Local::now().naive_local();

    let event_filter = |e: &'_ Event| { // what a crazy hack to make the compiler happy
        debug!("Filtering event: {:?}", e);

        // generate a function `filter_past` which filters out the past or not
        let allow_all_past_events = |event| if do_filter_past {
            filters::event_is_before(event, &today)
        } else {
            true
        };

        let do_filter_before = do_filter_before.map(|spec| kairos_parse(spec).map_err_trace_exit_unwrap());

        let allow_events_before_date = |event| do_filter_before.as_ref().map(|spec| {
            filters::event_is_before(event, spec)
        }).unwrap_or(true);


        let do_filter_after = do_filter_after.map(|spec| kairos_parse(spec).map_err_trace_exit_unwrap());

        let allow_events_after_date = |event| do_filter_after.as_ref().map(|spec| {
            filters::event_is_after(event, spec)
        }).unwrap_or(true);

        allow_all_past_events(e) && allow_events_before_date(e) && allow_events_after_date(e)
    };

    let mut listed_events = 0;

    rt.store()
        .all_events()
        .map_err_trace_exit_unwrap()
        .trace_unwrap_exit()
        .map(|sid| rt.store().get(sid))
        .trace_unwrap_exit()
        .map(|oe| oe.ok_or_else(|| err_msg("Missing entry while calling all_events()")))
        .trace_unwrap_exit()
        .map(|ev| ParsedEventFLE::parse(ev, &ref_config))
        .trace_unwrap_exit()
        .for_each(|parsed_entry| {
            parsed_entry
                .get_data()
                .events()
                .filter_map(RResult::ok)
                .filter(event_filter)
                .for_each(|event| {
                    listed_events = listed_events + 1;
                    let data      = build_data_object_for_handlebars(listed_events, &event);

                    let rendered = list_format
                        .render("format", &data)
                        .map_err(Error::from)
                        .map_err_trace_exit_unwrap();

                    writeln!(rt.stdout(), "{}", rendered).to_exit_code().unwrap_or_exit()
                });

            rt.report_touched(parsed_entry.get_entry().get_location()).unwrap_or_exit();
        });
}

fn show(rt: &Runtime) {
    let scmd        = rt.cli().subcommand_matches("show").unwrap(); // safe by clap
    let ref_config  = rt.config()
        .ok_or_else(|| format_err!("No configuration, cannot continue!"))
        .map_err_trace_exit_unwrap()
        .read_partial::<libimagentryref::reference::Config>()
        .map_err(Error::from)
        .map_err_trace_exit_unwrap()
        .ok_or_else(|| format_err!("Configuration missing: {}", libimagentryref::reference::Config::LOCATION))
        .map_err_trace_exit_unwrap();

    let list_format = util::get_event_print_format("calendar.show_format", rt, &scmd)
        .map_err_trace_exit_unwrap();

    let mut shown_events = 0;

    scmd.values_of("show-ids")
        .unwrap() // safe by clap
        .into_iter()
        .filter_map(|id| {
            util::find_event_by_id(rt.store(), id, &ref_config)
                .map(|entry| { debug!("Found => {:?}", entry); entry })
                .map_err_trace_exit_unwrap()
                .map(|parsed| (parsed, id))
        })
        .for_each(|(parsed_entry, id)| {
            parsed_entry
                .get_data()
                .events()
                .filter_map(RResult::ok)
                .filter(|pent| {
                    let relevant = pent.uid().map(|uid| uid.raw().starts_with(id)).unwrap_or(false);
                    debug!("Relevant {} => {}", parsed_entry.get_entry().get_location(), relevant);
                    relevant
                })
                .for_each(|event| {
                    shown_events = shown_events + 1;
                    let data     = util::build_data_object_for_handlebars(shown_events, &event);

                    let rendered = list_format
                        .render("format", &data)
                        .map_err(Error::from)
                        .map_err_trace_exit_unwrap();

                    writeln!(rt.stdout(), "{}", rendered).to_exit_code().unwrap_or_exit()
                });

            rt.report_touched(parsed_entry.get_entry().get_location()).unwrap_or_exit();
        });
}

/// helper function to check whether a DirEntry points to something hidden (starting with dot)
fn is_not_hidden(entry: &DirEntry) -> bool {
    !entry.file_name().to_str().map(|s| s.starts_with(".")).unwrap_or(false)
}

