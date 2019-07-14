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

//! Functions to be used for clap::Arg::validator()
//! to validate arguments

use std::path::PathBuf;
use boolinator::Boolinator;

pub fn is_existing_path<A: AsRef<str>>(s: A) -> Result<(), String> {
    PathBuf::from(s.as_ref()).exists().as_result((), format!("Not a File or Directory: {}", s.as_ref()))
}

pub fn is_file<A: AsRef<str>>(s: A) -> Result<(), String> {
    PathBuf::from(s.as_ref()).is_file().as_result((), format!("Not a File: {}", s.as_ref()))
}

pub fn is_directory<A: AsRef<str>>(s: A) -> Result<(), String> {
    PathBuf::from(s.as_ref()).is_dir().as_result((), format!("Not a Directory: {}", s.as_ref()))
}

pub fn is_integer<A: AsRef<str>>(s: A) -> Result<(), String> {
    use std::str::FromStr;

    let i : Result<i64, _> = FromStr::from_str(s.as_ref());
    i.map(|_| ()).map_err(|_| format!("Not an integer: {}", s.as_ref()))
}

pub fn is_float<A: AsRef<str>>(s: A) -> Result<(), String> {
    use std::str::FromStr;

    let i : Result<f64, _> = FromStr::from_str(s.as_ref());
    i.map(|_| ()).map_err(|_| format!("Not an float: {}", s.as_ref()))
}

pub fn is_bool<A: AsRef<str>>(s: A) -> Result<(), String> {
    use std::str::FromStr;

    let i : Result<bool, _> = FromStr::from_str(s.as_ref());
    i.map(|_| ()).map_err(|_| format!("Not an bool: {}", s.as_ref()))
}

pub fn is_url<A: AsRef<str>>(s: A) -> Result<(), String> {
    use url::Url;
    Url::parse(s.as_ref()).map(|_| ()).map_err(|_| format!("Not a URL: {}", s.as_ref()))
}

pub fn is_datetime<A: AsRef<str>>(s: A) -> Result<(), String> {
    crate::date::datetime_from_string(s).map(|_| ()).map_err(|e| format!("{}", e))
}

pub fn is_date<A: AsRef<str>>(s: A) -> Result<(), String> {
    crate::date::date_from_string(s).map(|_| ()).map_err(|e| format!("{}", e))
}

