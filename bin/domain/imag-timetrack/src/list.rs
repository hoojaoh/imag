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

use chrono::NaiveDateTime;
use filters::filter::Filter;
use prettytable::Table;
use prettytable::Row;
use prettytable::Cell;
use kairos::parser::Parsed;
use kairos::parser::parse as kairos_parse;
use clap::ArgMatches;
use failure::Fallible as Result;
use failure::ResultExt;
use failure::err_msg;
use failure::Error;

use libimagerror::trace::trace_error;
use libimagerror::trace::MapErrTrace;
use libimagerror::iter::TraceIterator;
use libimagerror::exit::ExitUnwrap;
use libimagstore::store::FileLockEntry;
use libimagtimetrack::store::TimeTrackStore;
use libimagtimetrack::timetracking::TimeTracking;

use libimagrt::runtime::Runtime;

pub fn list(rt: &Runtime) -> i32 {
    let (_, cmd) = rt.cli().subcommand();
    let cmd = cmd.unwrap(); // checked in main()

    let gettime = |cmd: &ArgMatches, name| {
        match cmd.value_of(name).map(kairos_parse) {
            Some(Ok(Parsed::TimeType(tt))) => match tt.calculate() {
                Ok(tt) => {
                    let dt = tt.get_moment().unwrap_or_else(|| {
                        error!("Failed to get date from '{}'", cmd.value_of(name).unwrap());
                        ::std::process::exit(1)
                    });

                    Some(dt.clone())
                },
                Err(e) => {
                    error!("Failed to calculate date from '{}': {:?}",
                           cmd.value_of(name).unwrap(), e);
                    ::std::process::exit(1)
                },
            },
            Some(Ok(Parsed::Iterator(_))) => {
                error!("Expected single point in time, got '{}', which yields a list of dates", cmd.value_of(name).unwrap());
                ::std::process::exit(1)
            },
            Some(Err(e)) => {
                let e = e;
                trace_error(&e);
                ::std::process::exit(1)
            }
            None => None,
        }
    };

    let start = gettime(&cmd, "start-time");
    let end   = gettime(&cmd, "end-time");

    let list_not_ended = cmd.is_present("list-not-ended");
    let show_duration  = cmd.is_present("show-duration");

    list_impl(rt, start, end, list_not_ended, show_duration)
}

pub fn list_impl(rt: &Runtime,
                 start: Option<NaiveDateTime>,
                 end: Option<NaiveDateTime>,
                 list_not_ended: bool,
                 show_duration: bool)
    -> i32
{

    let start_time_filter = |timetracking: &FileLockEntry| {
        start.map(|s| match timetracking.get_start_datetime() {
            Ok(Some(dt)) => dt >= s,
            Ok(None)     => {
                warn!("Funny things are happening: Timetracking has no start time");
                false
            }
            Err(e) => {
                trace_error(&e);
                false
            }
        })
        .unwrap_or(true)
    };

    let end_time_filter = |timetracking: &FileLockEntry| {
        end.map(|s| match timetracking.get_end_datetime() {
            Ok(Some(dt)) => dt <= s,
            Ok(None)     => list_not_ended,
            Err(e)       => {
                trace_error(&e);
                false
            }
        })
        .unwrap_or(true)
    };

    let filter = start_time_filter.and(end_time_filter);

    let mut table = Table::new();
    let title_row = if !show_duration {
        Row::new(["Tag", "Start", "End"].iter().map(|s| Cell::new(s)).collect())
    } else {
        Row::new(["Tag", "Start", "End", "Duration"].iter().map(|s| Cell::new(s)).collect())
    };
    table.set_titles(title_row);

    let mut table_empty = true;

    let table = rt.store()
        .get_timetrackings()
        .map_err_trace_exit_unwrap()
        .trace_unwrap()
        .filter(|e| filter.filter(e))
        .fold(Ok(table), |acc: Result<_>, e| {
            acc.and_then(|mut tab: Table| {
                debug!("Processing {:?}", e.get_location());

                let tag   = e.get_timetrack_tag()?;
                debug!(" -> tag = {:?}", tag);

                let start = e.get_start_datetime()?;
                debug!(" -> start = {:?}", start);

                let end   = e.get_end_datetime()?;
                debug!(" -> end = {:?}", end);

                let v = match (start, end) {
                    (None, _)          => {
                        let mut v = vec![String::from(tag.as_str()), String::from(""), String::from("")];
                        if show_duration {
                            v.push(String::from(""));
                        }
                        v
                    },
                    (Some(s), None)    => {
                        let mut v = vec![
                            String::from(tag.as_str()),
                            format!("{}", s),
                            String::from(""),
                        ];

                        if show_duration {
                            v.push(String::from(""));
                        }

                        v
                    },
                    (Some(s), Some(e)) => {
                        let mut v = vec![
                            String::from(tag.as_str()),
                            format!("{}", s),
                            format!("{}", e),
                        ];

                        if show_duration {
                            let duration = e - s;
                            let dur = format!("{days} Days, {hours} Hours, {minutes} Minutes, {seconds} Seconds",
                                    days    = duration.num_days(),
                                    hours   = duration.num_hours(),
                                    minutes = duration.num_minutes(),
                                    seconds = duration.num_seconds());

                            v.push(dur);
                        }

                        v
                    },
                };

                let cells : Vec<Cell> = v
                    .into_iter()
                    .map(|s| Cell::new(&s))
                    .collect();
                tab.add_row(Row::new(cells));

                rt.report_touched(e.get_location()).unwrap_or_exit();

                table_empty = false;
                Ok(tab)
            })
        })
        .map_err_trace_exit_unwrap();

    if !table_empty {
        table.print(&mut rt.stdout())
            .context(err_msg("Failed to print table"))
            .map_err(Error::from)
            .map(|_| 0)
            .map_err_trace()
            .unwrap_or(1)
    } else {
        0
    }
}

