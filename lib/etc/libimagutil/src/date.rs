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

use chrono::NaiveDate;
use chrono::NaiveDateTime;
use chrono::format::ParseError;

pub const NAIVE_DATE_STRING_FORMAT : &str = "%Y-%m-%d";

pub fn date_to_string(ndt: NaiveDate) -> String {
    ndt.format(NAIVE_DATE_STRING_FORMAT).to_string()
}

pub fn date_from_string<S>(s: S) -> Result<NaiveDate, ParseError>
    where S: AsRef<str>
{
    NaiveDate::parse_from_str(s.as_ref(), NAIVE_DATE_STRING_FORMAT)
}

pub const NAIVE_DATETIME_STRING_FORMAT : &str = "%Y-%m-%d %H:%M:%S";

pub fn datetime_to_string(ndt: &NaiveDateTime) -> String {
    ndt.format(NAIVE_DATETIME_STRING_FORMAT).to_string()
}

pub fn datetime_from_string<S>(s: S) -> Result<NaiveDateTime, ParseError>
    where S: AsRef<str>
{
    NaiveDateTime::parse_from_str(s.as_ref(), NAIVE_DATETIME_STRING_FORMAT)
}

/// Try to parse `s` with all formats from `fmts`
///
/// The function returns an `Option<NaiveDateTime>`, so that the user of the function can generate
/// the appropriate error message themselves.
///
pub fn try_to_parse_datetime_from_string<'a, S, Formats, Format>(s: S, fmts: Formats) -> Option<NaiveDateTime>
    where S: AsRef<str>,
          Formats: Iterator<Item = Format>,
          Format: AsRef<str>
{
    fmts.fold(None, |a, f| a.or_else(|| NaiveDateTime::parse_from_str(s.as_ref(), f.as_ref()).ok()))
}

#[test]
fn test_try_to_parse_datetime_from_string() {
    let formats = vec![
        "%Y%m%dT%H%M%S",
        "%Y%m%dT%H%M%SZ"
    ];

    let text = "20190730T160527Z";

    assert!(try_to_parse_datetime_from_string(text, formats.iter()).is_some())
}

