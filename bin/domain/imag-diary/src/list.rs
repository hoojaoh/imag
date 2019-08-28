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

use libimagdiary::diary::Diary;
use libimagrt::runtime::Runtime;
use libimagutil::warn_exit::warn_exit;
use libimagerror::trace::MapErrTrace;
use libimagerror::iter::TraceIterator;
use libimagerror::io::ToExitCode;
use libimagerror::exit::ExitUnwrap;
use libimagutil::debug_result::*;
use libimagdiary::diaryid::DiaryId;
use libimagdiary::diaryid::FromStoreId;
use libimagstore::storeid::IntoStoreId;

use failure::Fallible as Result;

use crate::util::get_diary_name;

pub fn list(rt: &Runtime) {
    let diaryname = get_diary_name(rt)
        .unwrap_or_else(|| warn_exit("No diary selected. Use either the configuration file or the commandline option", 1));

    let mut ids = Diary::entries(rt.store(), &diaryname)
        .map_dbg_str("Ok")
        .map_err_trace_exit_unwrap()
        .trace_unwrap_exit()
        .map(|id| DiaryId::from_storeid(&id))
        .collect::<Result<Vec<_>>>()
        .map_err_trace_exit_unwrap();

    ids.sort_by_key(|id| {
        [id.year() as u32, id.month(), id.day(), id.hour(), id.minute(), id.second()]
    });

    ids.into_iter()
        .map(IntoStoreId::into_storeid)
        .trace_unwrap_exit()
        .for_each(|id| {
            rt.report_touched(&id).unwrap_or_exit();

            if !rt.output_is_pipe() {
                writeln!(rt.stdout(), "{}", id).to_exit_code().unwrap_or_exit()
            }
        });
}

