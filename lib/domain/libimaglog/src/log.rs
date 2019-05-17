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

use libimagdiary::entry::DiaryEntry;
use libimagstore::store::Entry;
use libimagentryutil::isa::Is;
use libimagentryutil::isa::IsKindHeaderPathProvider;

use failure::Fallible as Result;
use failure::Error;
use failure::ResultExt;


use toml::Value;
use toml_query::insert::TomlValueInsertExt;

pub trait Log : DiaryEntry {
    fn is_log(&self) -> Result<bool>;
    fn make_log_entry(&mut self) -> Result<()>;
}

provide_kindflag_path!(pub IsLog, "log.is_log");

impl Log for Entry {
    fn is_log(&self) -> Result<bool> {
        self.is::<IsLog>().context("Cannot check whether Entry is a Log").map_err(From::from)
    }

    fn make_log_entry(&mut self) -> Result<()> {
        self.get_header_mut()
            .insert("log.is_log", Value::Boolean(true))
            .context("Cannot insert 'log.is_log' into header of entry")
            .map_err(Error::from)
            .map(|_| ())
    }

}

