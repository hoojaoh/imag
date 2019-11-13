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
extern crate chrono;
extern crate filters;
extern crate kairos;
#[macro_use] extern crate log;
#[macro_use] extern crate failure;
extern crate resiter;

#[cfg(feature = "import-taskwarrior")]
extern crate task_hookrs;

#[cfg(feature = "import-taskwarrior")]
extern crate uuid;

#[cfg(feature = "import-taskwarrior")]
extern crate libimagentrytag;

#[cfg(feature = "import-taskwarrior")]
extern crate libimagentrylink;

extern crate libimagrt;
extern crate libimagstore;
extern crate libimagerror;
extern crate libimagentryedit;
extern crate libimagtodo;
extern crate libimagutil;
extern crate libimagentryview;

use std::io::Write;
use std::result::Result as RResult;

use clap::ArgMatches;
use chrono::NaiveDateTime;
use failure::Error;
use failure::Fallible as Result;
use failure::err_msg;
use clap::App;
use resiter::AndThen;
use resiter::IterInnerOkOrElse;

use libimagentryedit::edit::Edit;
use libimagentryview::viewer::ViewFromIter;
use libimagentryview::viewer::Viewer;
use libimagrt::application::ImagApplication;
use libimagrt::runtime::Runtime;
use libimagstore::iter::get::*;
use libimagstore::store::Entry;
use libimagstore::store::FileLockEntry;
use libimagtodo::entry::Todo;
use libimagtodo::priority::Priority;
use libimagtodo::status::Status;
use libimagtodo::store::TodoStore;
use libimagutil::date::datetime_to_string;

mod ui;
mod import;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagTodo {}
impl ImagApplication for ImagTodo {
    fn run(rt: Runtime) -> Result<()> {
        match rt.cli().subcommand_name() {
            Some("create")      => create(&rt),
            Some("show")        => show(&rt),
            Some("mark")           => mark(&rt),
            Some("pending") | None => list_todos(&rt, &StatusMatcher::new().is(Status::Pending), false),
            Some("list")           => list(&rt),
            Some("import")         => import::import(&rt),
            Some(other)         => {
                debug!("Unknown command");
                if rt.handle_unknown_subcommand("imag-todo", other, rt.cli())?.success() {
                    Ok(())
                } else {
                    Err(err_msg("Failed to handle unknown subcommand"))
                }
            }
        } // end match scmd
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Interface with taskwarrior"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

/// A black- and whitelist for matching statuses of todo entries
///
/// The blacklist is checked first, followed by the whitelist.
/// In case the whitelist is empty, the StatusMatcher works with a
/// blacklist-only approach.
#[derive(Debug)]
pub struct StatusMatcher {
    is: Vec<Status>,
    is_not: Vec<Status>,
}

impl StatusMatcher {
    pub fn new() -> Self {
        StatusMatcher {
            is: Vec::new(),
            is_not: Vec::new(),
        }
    }

    pub fn is(mut self, s: Status) -> Self {
        self.add_is(s);
        self
    }

    pub fn add_is(&mut self, s: Status) {
        self.is.push(s);
    }

    pub fn is_not(mut self, s: Status) -> Self {
        self.add_is_not(s);
        self
    }

    pub fn add_is_not(&mut self, s: Status) {
        self.is_not.push(s);
    }

    pub fn matches(&self, todo: Status) -> bool {
        if self.is_not.iter().find(|t| **t == todo).is_some() {
            // On blacklist
            false
        } else if self.is.len() < 1 || self.is.iter().find(|t| **t == todo).is_some() {
            // No whitelist or on whitelist
            true
        } else {
            // Not on blacklist, but whitelist exists and not on it either
            false
        }
    }
}

fn create(rt: &Runtime) -> Result<()> {
    debug!("Creating todo");
    let scmd = rt.cli().subcommand().1.unwrap(); // safe by clap

    let scheduled: Option<NaiveDateTime> = get_datetime_arg(&scmd, "create-scheduled")?;
    let hidden: Option<NaiveDateTime>    = get_datetime_arg(&scmd, "create-hidden")?;
    let due: Option<NaiveDateTime>       = get_datetime_arg(&scmd, "create-due")?;
    let prio: Option<Priority>           = scmd.value_of("create-prio").map(prio_from_str).transpose()?;
    let status: Status                   = scmd.value_of("create-status").map(Status::from_str).unwrap()?;
    let edit                             = scmd.is_present("create-edit");
    let text                             = scmd.value_of("text").unwrap();

    trace!("Creating todo with these variables:");
    trace!("scheduled = {:?}", scheduled);
    trace!("hidden    = {:?}", hidden);
    trace!("due       = {:?}", due);
    trace!("prio      = {:?}", prio);
    trace!("status    = {:?}", status);
    trace!("edit      = {}", edit);
    trace!("text      = {:?}", text);

    let mut entry = rt.store().create_todo(status, scheduled, hidden, due, prio, true)?;
    debug!("Created: todo {}", entry.get_uuid()?);

    debug!("Setting content");
    *entry.get_content_mut() = text.to_string();

    if edit {
        debug!("Editing content");
        entry.edit_content(&rt)?;
    }

    rt.report_touched(entry.get_location()).map_err(Error::from)
}

fn mark(rt: &Runtime) -> Result<()> {
    fn mark_todos_as(rt: &Runtime, status: Status) -> Result<()> {
        rt.ids::<crate::ui::PathProvider>()?
            .ok_or_else(|| err_msg("No ids supplied"))?
            .into_iter()
            .map(Ok)
            .into_get_iter(rt.store())
            .map_inner_ok_or_else(|| err_msg("Did not find one entry"))
            .and_then_ok(|e| rt.report_touched(e.get_location()).map_err(Error::from).map(|_| e))
            .and_then_ok(|mut e| e.set_status(status.clone()))
            .collect()
    }

    let scmd = rt.cli().subcommand().1.unwrap();
    match scmd.subcommand_name() {
        Some("done")    => mark_todos_as(rt, Status::Done),
        Some("delete")  => mark_todos_as(rt, Status::Deleted),
        Some("pending") => mark_todos_as(rt, Status::Pending),
        Some(other)     => Err(format_err!("Unknown mark type selected: {}", other)),
        None            => Err(format_err!("No mark type selected, doing nothing!")),
    }
}

/// Generic todo listing function
///
/// Supports filtering of todos by status using the passed in StatusMatcher
fn list_todos(rt: &Runtime, matcher: &StatusMatcher, show_hidden: bool) -> Result<()> {
    use filters::failable::filter::FailableFilter;
    debug!("Listing todos with status filter {:?}", matcher);

    let now = {
        let now = chrono::offset::Local::now();
        NaiveDateTime::new(now.date().naive_local(), now.time())
    };

    let filter_hidden = |todo: &FileLockEntry<'_>| -> Result<bool> {
        Ok(todo.get_hidden()?.map(|hid| hid > now).unwrap_or(true))
    };

    struct TodoViewer {
        details: bool,
    }
    impl Viewer for TodoViewer {
        fn view_entry<W>(&self, entry: &Entry, sink: &mut W) -> RResult<(), libimagentryview::error::Error>
            where W: Write
        {
            use libimagentryview::error::Error as E;

            if !entry.is_todo().map_err(E::from)? {
                return Err(format_err!("Not a Todo: {}", entry.get_location())).map_err(E::from);
            }

            let uuid     = entry.get_uuid().map_err(E::from)?;
            let status   = entry.get_status().map_err(E::from)?;
            let status   = status.as_str();
            let first_line = entry.get_content()
                .lines()
                .next()
                .unwrap_or("<empty description>");

            if !self.details {
                writeln!(sink, "{uuid} - {status} : {first_line}",
                         uuid = uuid,
                         status = status,
                         first_line = first_line)
            } else {
                let sched    = get_dt_str(entry.get_scheduled(), "Not scheduled")?;
                let hidden   = get_dt_str(entry.get_hidden(), "Not hidden")?;
                let due      = get_dt_str(entry.get_due(), "No due")?;
                let priority = entry.get_priority().map_err(E::from)?.map(|p| p.as_str().to_string())
                    .unwrap_or("No prio".to_string());

                writeln!(sink, "{uuid} - {status} - {sched} - {hidden} - {due} - {prio}: {first_line}",
                         uuid = uuid,
                         status = status,
                         sched = sched,
                         hidden = hidden,
                         due = due,
                         prio = priority,
                         first_line = first_line)
            }
            .map_err(libimagentryview::error::Error::from)
        }
    }

    let viewer = TodoViewer { details: false };

    rt.store()
        .get_todos()?
        .into_get_iter()
        .map_inner_ok_or_else(|| err_msg("Did not find one entry"))
        .filter_map(|r| {
            match r.and_then(|e| e.get_status().map(|s| (s, e))) {
                Err(e) => Some(Err(e)),
                Ok((st, e)) => if matcher.matches(st) {
                    Some(Ok(e))
                } else {
                    None
                }
            }
        })
        .and_then_ok(|entry| {
            if !rt.output_is_pipe() && (show_hidden || filter_hidden.filter(&entry)?) {
                viewer.view_entry(&entry, &mut rt.stdout())?;
            }

            rt.report_touched(entry.get_location()).map_err(Error::from)
        })
        .collect()
}

/// Generic todo items list function
///
/// This sets up filtes based on the command line and prints out a list of todos
fn list(rt: &Runtime) -> Result<()> {
    debug!("Listing todo");
    let scmd      = rt.cli().subcommand().1;
    let table     = scmd.map(|s| s.is_present("list-table")).unwrap_or(true);
    let hidden    = scmd.map(|s| s.is_present("list-hidden")).unwrap_or(false);
    let done      = scmd.map(|s| s.is_present("list-done")).unwrap_or(false);
    let nopending = scmd.map(|s| s.is_present("list-nopending")).unwrap_or(true);

    trace!("table     = {}", table);
    trace!("hidden    = {}", hidden);
    trace!("done      = {}", done);
    trace!("nopending = {}", nopending);

    let mut matcher = StatusMatcher::new();
    if !done { matcher.add_is_not(Status::Done); }
    if nopending { matcher.add_is_not(Status::Pending); }

    // TODO: Support printing as ASCII table
    list_todos(rt, &matcher, hidden)
}

fn show(rt: &Runtime) -> Result<()> {
    #[derive(Default)]
    struct TodoShow;
    impl Viewer for TodoShow {

        fn view_entry<W>(&self, entry: &Entry, sink: &mut W) -> RResult<(), libimagentryview::error::Error>
            where W: Write
        {
            use libimagentryview::error::Error as E;

            if !entry.is_todo().map_err(E::from)? {
                return Err(format_err!("Not a Todo: {}", entry.get_location())).map_err(E::from);
            }

            let uuid     = entry.get_uuid().map_err(E::from)?;
            let status   = entry.get_status().map_err(E::from)?;
            let status   = status.as_str();
            let text     = entry.get_content();
            let sched    = get_dt_str(entry.get_scheduled(), "Not scheduled")?;
            let hidden   = get_dt_str(entry.get_hidden(), "Not hidden")?;
            let due      = get_dt_str(entry.get_due(), "No due")?;
            let priority = entry.get_priority().map_err(E::from)?.map(|p| p.as_str().to_string())
                .unwrap_or("No prio".to_string());

            writeln!(sink, "{uuid}\nStatus: {status}\nPriority: {prio}\nScheduled: {sched}\nHidden: {hidden}\nDue: {due}\n\n{text}",
                     uuid   = uuid,
                     status = status,
                     sched  = sched,
                     hidden = hidden,
                     due    = due,
                     prio   = priority,
                     text   = text)
                .map_err(Error::from)
                .map_err(libimagentryview::error::Error::from)
        }
    }

    rt.ids::<crate::ui::PathProvider>()?
        .ok_or_else(|| err_msg("No ids supplied"))?
        .into_iter()
        .map(Ok)
        .into_get_iter(rt.store())
        .map_inner_ok_or_else(|| err_msg("Did not find one entry"))
        .and_then_ok(|e| rt.report_touched(e.get_location()).map_err(Error::from).map(|_| e))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .view::<TodoShow, _>(&mut rt.stdout())
        .map_err(Error::from)
}

//
// utility functions
//

fn get_datetime_arg(scmd: &ArgMatches, argname: &'static str) -> Result<Option<NaiveDateTime>> {
    use kairos::timetype::TimeType;
    use kairos::parser;

    match scmd.value_of(argname) {
        None => Ok(None),
        Some(v) => match parser::parse(v)? {
            parser::Parsed::TimeType(TimeType::Moment(moment)) => Ok(Some(moment)),
            parser::Parsed::TimeType(other) => {
                Err(format_err!("You did not pass a date, but a {}", other.name()))
            },
            parser::Parsed::Iterator(_) => {
                Err(format_err!("Argument {} results in a list of dates, but we need a single date.", v))
            }
        }
    }
}

fn prio_from_str<S: AsRef<str>>(s: S) -> Result<Priority> {
    match s.as_ref() {
        "h" => Ok(Priority::High),
        "m" => Ok(Priority::Medium),
        "l" => Ok(Priority::Low),
        other => Err(format_err!("Unsupported Priority: '{}'", other)),
    }
}

fn get_dt_str(d: Result<Option<NaiveDateTime>>, s: &str) -> RResult<String, libimagentryview::error::Error> {
    Ok(d.map_err(libimagentryview::error::Error::from)?
       .map(|v| datetime_to_string(&v))
       .unwrap_or(s.to_string()))
}

