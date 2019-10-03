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

use vobject::icalendar::ICalendar;
use failure::Fallible as Result;
use failure::Error;

use libimagstore::store::FileLockEntry;
use libimagentryref::reference::fassade::RefFassade;
use libimagentryref::reference::Ref;
use libimagentryref::reference::Config;
use libimagentryref::hasher::default::DefaultHasher;

pub struct ParsedEventFLE<'a> {
    inner: FileLockEntry<'a>,
    data: ICalendar,
}

impl<'a> ParsedEventFLE<'a> {

    /// Because libimagcalendar only links to the actual calendar data, we need to read the data and
    /// parse it.
    /// With this function, a FileLockEntry can be parsed to a ParsedEventFileLockEntry
    /// (ParsedEventFLE).
    pub fn parse(fle: FileLockEntry<'a>, refconfig: &Config) -> Result<Self> {
        fle.as_ref_with_hasher::<DefaultHasher>()
            .get_path(refconfig)
            .and_then(|p| ::std::fs::read_to_string(p).map_err(Error::from))
            .and_then(|s| ICalendar::build(&s).map_err(Error::from))
            .map(|cal| ParsedEventFLE {
                inner: fle,
                data: cal,
            })
    }

    pub fn get_entry(&self) -> &FileLockEntry<'a> {
        &self.inner
    }

    pub fn get_data(&self) -> &ICalendar {
        &self.data
    }
}

