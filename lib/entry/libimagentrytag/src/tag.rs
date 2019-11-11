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

use std::result::Result;

use failure::Error;

pub type Tag = String;
pub type TagSlice<'a> = &'a str;

/// validator which can be used by clap to validate that a string is a valid tag
pub fn is_tag(s: String) -> Result<(), String> {
    check_tag_string(&s)
}

pub fn is_tag_str(s: &str) -> Result<(), Error> {
    check_tag_string(s).map_err(|s| format_err!("{}", s))
}

fn check_tag_string(s: &str) -> Result<(), String> {
    trace!("Checking whether '{}' is a valid tag", s);

    let is_lower      = |s: &&str| s.chars().all(|c| c.is_lowercase());
    let no_whitespace = |s: &&str| s.chars().all(|c| !c.is_whitespace());
    let is_alphanum   = |s: &&str| s.chars().all(|c| c.is_alphanumeric());

    match (is_lower(&s), no_whitespace(&s), is_alphanum(&s)) {
        (true, true, true) => Ok(()),
        (false, false, false) => Err(format!("The string '{}' is not valid, because it is not all-lowercase, has whitespace and is not alphanumeric", s)),
        (false, false, _ )    => Err(format!("The string '{}' is not valid, because it is not all-lowercase and has whitespace", s)),
        (false, _, _ )        => Err(format!("The string '{}' is not valid, because it is not all-lowercase", s)),
        (_, false, _ )        => Err(format!("The string '{}' is not valid, because it has whitespace", s)),
        (_, _, false)         => Err(format!("The string '{}' is not valid, because it is not alphanumeric", s)),
    }
}

