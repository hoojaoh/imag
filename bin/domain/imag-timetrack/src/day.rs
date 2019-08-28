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

use std::io::Write;
use std::str::FromStr;

use filters::filter::Filter;
use chrono::NaiveDateTime;
use failure::Error;

use libimagerror::trace::trace_error;
use libimagerror::trace::MapErrTrace;
use libimagerror::iter::TraceIterator;
use libimagerror::io::ToExitCode;
use libimagerror::exit::ExitUnwrap;
use libimagstore::store::FileLockEntry;
use libimagtimetrack::store::TimeTrackStore;
use libimagtimetrack::timetracking::TimeTracking;
use libimagtimetrack::tag::TimeTrackingTag;
use libimagtimetrack::iter::filter::*;

use libimagrt::runtime::Runtime;


pub fn day(rt: &Runtime) -> i32 {
    let (_, cmd) = rt.cli().subcommand();
    let cmd = cmd.unwrap(); // checked in main()

    let filter = {
        let start = match cmd.value_of("start").map(NaiveDateTime::from_str) {
            None    => ::chrono::offset::Local::today().and_hms(0, 0, 0).naive_local(),
            Some(s) => match s.map_err(Error::from) {
                Ok(dt) => dt,
                Err(e) => {
                    trace_error(&e);
                    return 1
                }
            }
        };

        let end = match cmd.value_of("end").map(NaiveDateTime::from_str) {
            None    => ::chrono::offset::Local::today().and_hms(23, 59, 59).naive_local(),
            Some(s) => match s.map_err(Error::from) {
                Ok(dt) => dt,
                Err(e) => {
                    trace_error(&e);
                    return 1
                }
            }
        };

        let tags = cmd
            .values_of("tags")
            .map(|ts| ts.map(String::from).map(TimeTrackingTag::from).collect());

        let start_time_filter = has_start_time_where(move |dt: &NaiveDateTime| {
            start <= *dt
        });

        let end_time_filter = has_end_time_where(move |dt: &NaiveDateTime| {
            end >= *dt
        });

        let tags_filter = move |fle: &FileLockEntry| {
            match tags {
                Some(ref tags) => has_one_of_tags(&tags).filter(fle),
                None => true,
            }
        };

        tags_filter.and(start_time_filter).and(end_time_filter)
    };

    rt.store()
        .get_timetrackings()
        .map_err_trace_exit_unwrap()
        .trace_unwrap()
        .filter(|e| filter.filter(e))
        .map(|e| -> Result<_, Error> {
            debug!("Processing {:?}", e.get_location());

            let tag   = e.get_timetrack_tag()?;
            debug!(" -> tag = {:?}", tag);

            let start = e.get_start_datetime()?;
            debug!(" -> start = {:?}", start);

            let end   = e.get_end_datetime()?;
            debug!(" -> end = {:?}", end);

            rt.report_touched(e.get_location()).unwrap_or_exit();

            Ok((tag, start, end))
        })
        .trace_unwrap_exit()
        .map(|(tag, start, end)| {
            match (start, end) {
                (None, _)          => writeln!(rt.stdout(), "{} has no start time.", tag),
                (Some(s), None)    => writeln!(rt.stdout(), "{} | {} - ...", tag, s),
                (Some(s), Some(e)) => writeln!(rt.stdout(), "{} | {} - {}", tag, s, e),
            }
            .to_exit_code()
        })
        .collect::<Result<Vec<()>, _>>()
        .map(|_| 0)
        .unwrap_or_else(|e| e.code())
}

