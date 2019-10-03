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
use failure::Fallible as Result;
use vobject::icalendar::Event;
use libimagerror::trace::MapErrTrace;

pub fn event_is_before<'a>(event: &Event<'a>, before_spec: &NaiveDateTime) -> bool {
    let uid = || event.uid()
        .map(|uid| uid.into_raw())
        .unwrap_or_else(|| String::from("<No UID>"));

    let dtend_is_before_spec : Result<bool> = event.dtend()
        .map(|dtend| {
            let datetime = try_to_parse_datetime(dtend.raw())?;
            let result = datetime < *before_spec;
            trace!("{} < {} => {}", datetime, before_spec, result);
            Ok(result)
        })
        .unwrap_or_else(|| Err({
            format_err!("Entry with UID {} has no end time, cannot determine whether to list it",
                        uid())
        }));

    let dtstamp_is_before_spec : Result<bool> = event.dtstamp()
        .map(|dtstamp| {
            let datetime = try_to_parse_datetime(dtstamp.raw())?;
            let result = datetime < *before_spec;
            trace!("{} < {} => {}", datetime, before_spec, result);
            Ok(result)
        })
        .unwrap_or_else(|| Err({
            format_err!("Entry with UID {} has no timestamp, cannot determine whether to list it",
                        uid())
        }));

    trace!("dtend_is_before_spec   = {:?}", dtend_is_before_spec);
    trace!("dtstamp_is_before_spec = {:?}", dtstamp_is_before_spec);

    match (dtend_is_before_spec, dtstamp_is_before_spec) {
        (Ok(b), _)  => return b,
        (_, Ok(b))  => return b,
        (Err(e), _) => return Err(e).map_err_trace_exit_unwrap()
    }
}

fn try_to_parse_datetime(s: &str) -> Result<NaiveDateTime> {
    const FORMATS : &[&'static str] = &[
        "%Y%m%dT%H%M%S",
        "%Y%m%dT%H%M%SZ"
    ];

    ::libimagutil::date::try_to_parse_datetime_from_string(s, FORMATS.iter())
        .ok_or_else(|| format_err!("Cannot parse datetime: {}", s))
}

