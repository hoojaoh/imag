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

use std::env;
use std::process::Command;

use filters::filter::Filter;

use libimagerror::exit::ExitUnwrap;
use libimagerror::iter::TraceIterator;
use libimagerror::trace::MapErrTrace;
use libimagerror::trace::trace_error;
use libimagrt::runtime::Runtime;
use libimagtimetrack::iter::filter::has_one_of_tags;
use libimagtimetrack::store::TimeTrackStore;
use libimagtimetrack::tag::TimeTrackingTag;
use libimagtimetrack::timetracking::TimeTracking;
use libimagutil::warn_result::*;

pub fn shell(rt: &Runtime) -> i32 {
    let (_, cmd) = rt.cli().subcommand();
    let cmd = cmd.unwrap(); // checked in main()

    let start = ::chrono::offset::Local::now().naive_local();
    let tags = cmd.values_of("tags")
        .unwrap() // enforced by clap
        .map(String::from)
        .map(TimeTrackingTag::from)
        .collect::<Vec<_>>();

    let mut shellcmd = {
        let mkshell = |s: String| {
            let mut cmd = Command::new(s);
            cmd.stdin(::std::process::Stdio::inherit());
            cmd.stdout(::std::process::Stdio::inherit());
            cmd.stderr(::std::process::Stdio::inherit());
            cmd
        };

        if let Some(s) = cmd.value_of("shell") {
            mkshell(s.to_owned())
        } else {
            env::var("SHELL")
                .map(mkshell)
                .map_err(|e| match e {
                    env::VarError::NotPresent => {
                        error!("No $SHELL variable in environment, cannot work!");
                        ::std::process::exit(1)
                    },
                    env::VarError::NotUnicode(_) => {
                        error!("$SHELL variable is not unicode, cannot work!");
                        ::std::process::exit(1)
                    }
                })
                .unwrap()
        }
    };

    for tag in tags.iter() {
        match rt.store().create_timetracking_at(&start, tag) {
            Err(e) => trace_error(&e),
            Ok(entry) => {
                rt.report_touched(entry.get_location()).unwrap_or_exit();
            }
        }
    }

    let exit_code = match shellcmd.status() {
        Ok(estat) => estat.code().unwrap_or(0),
        Err(e) => {
            error!("Error starting shell: {:?}", e);
            ::std::process::exit(2)
        },
    };

    let stop = ::chrono::offset::Local::now().naive_local();
    let filter = has_one_of_tags(&tags);
    rt.store()
        .get_timetrackings()
        .map_warn_err_str("Getting timetrackings failed")
        .map_err_trace_exit_unwrap()
        .trace_unwrap()
        .filter(|e| filter.filter(e))
        .for_each(|mut elem| if let Err(e) = elem.set_end_datetime(stop.clone()) {
            trace_error(&e)
        } else {
            debug!("Setting end time worked: {:?}", elem);
            rt.report_touched(elem.get_location()).unwrap_or_exit();
        });

    ::std::process::exit(exit_code)
}


