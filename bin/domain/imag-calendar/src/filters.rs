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

/// Generate a filter function to filter shown events
///
/// If `do_filter` is false, this filter returns true all the time.
///
/// If an event is in the past, relative to the `today` parameter, the function returns false,
/// else it returns true.
///
/// # Details
///
/// The date of the event is determined by using the "dtend" or "dtstamp" members of the event
/// object. These fields are parsed to NaiveDateTime objects and then compared to the `today` object.
///
/// If an parsing error happens in the "dtend" parsing step, "dtstamp" is used. If this results also
/// in a parsing error, the first error is returned.
///
pub fn filter_past(do_filter: bool, today: NaiveDateTime) -> impl FnOnce(&Event) -> Result<bool> {
    move |pe| if do_filter {
        let uid = || pe.uid()
            .map(|uid| uid.into_raw())
            .unwrap_or_else(|| String::from("<No UID>"));

        let dtend_is_pre_today : Result<bool> = pe.dtend()
            .map(|dtend| Ok(try_to_parse_datetime(dtend.raw())? < today))
            .unwrap_or_else(|| Err({
                format_err!("Entry with UID {} has no end time, cannot determine whether to list it",
                            uid())
            }));

        let dtstamp_is_pre_today : Result<bool> = pe.dtstamp()
            .map(|dtstamp| Ok(try_to_parse_datetime(dtstamp.raw())? < today))
            .unwrap_or_else(|| Err({
                format_err!("Entry with UID {} has no timestamp, cannot determine whether to list it",
                            uid())
            }));

        trace!("dtend_is_pre_today   = {:?}", dtend_is_pre_today);
        trace!("dtstamp_is_pre_today = {:?}", dtstamp_is_pre_today);

        match (dtend_is_pre_today, dtstamp_is_pre_today) {
            (Ok(b), _)  => return Ok(!b),
            (_, Ok(b))  => return Ok(!b),
            (Err(e), _) => return Err(e)
        }
    } else {
        Ok(true)
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

