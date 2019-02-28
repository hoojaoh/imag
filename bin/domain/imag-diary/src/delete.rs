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

use std::process::exit;

use chrono::naive::NaiveDateTime as NDT;

use libimagdiary::diaryid::DiaryId;
use libimagrt::runtime::Runtime;
use libimagtimeui::datetime::DateTime;
use libimagtimeui::parse::Parse;
use libimagutil::warn_exit::warn_exit;
use libimagstore::storeid::IntoStoreId;
use libimagerror::trace::MapErrTrace;
use libimagerror::exit::ExitUnwrap;

use crate::util::get_diary_name;

pub fn delete(rt: &Runtime) {
    use libimaginteraction::ask::ask_bool;

    let diaryname = get_diary_name(rt)
        .unwrap_or_else(|| warn_exit("No diary selected. Use either the configuration file or the commandline option", 1));

    let to_del_location = rt
        .cli()
        .subcommand_matches("delete")
        .unwrap()
        .value_of("datetime")
        .map(|dt| { debug!("DateTime = {:?}", dt); dt })
        .and_then(DateTime::parse)
        .map(Into::into)
        .ok_or_else(|| warn_exit("Not deleting entries: missing date/time specification", 1))
        .and_then(|dt: NDT| DiaryId::from_datetime(diaryname.clone(), dt).into_storeid())
        .and_then(|id| rt.store().retrieve(id))
        .map_err_trace_exit_unwrap()
        .get_location()
        .clone();

    let mut input = rt.stdin().unwrap_or_else(|| {
        error!("No input stream. Cannot ask for permission");
        exit(1);
    });

    let mut output = rt.stdout();

    if !ask_bool(&format!("Deleting {:?}", to_del_location), Some(true), &mut input, &mut output)
        .map_err_trace_exit_unwrap()
    {
        info!("Aborting delete action");
        return;
    }

    let _ = rt.report_touched(&to_del_location).unwrap_or_exit();

    let _ = rt
        .store()
        .delete(to_del_location)
        .map_err_trace_exit_unwrap();

    info!("Ok!");
}

