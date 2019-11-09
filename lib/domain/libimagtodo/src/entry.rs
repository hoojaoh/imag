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

use libimagentryutil::isa::Is;
use libimagentryutil::isa::IsKindHeaderPathProvider;
use libimagstore::store::Entry;
use libimagutil::date::datetime_from_string;

use failure::Fallible as Result;
use failure::Error;
use failure::ResultExt;
use chrono::NaiveDateTime;
use toml_query::read::Partial;
use toml_query::read::TomlValueReadExt;
use toml_query::insert::TomlValueInsertExt;
use uuid::Uuid;

use crate::status::Status;
use crate::priority::Priority;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct TodoHeader {
    pub(crate) uuid:       Uuid,
    pub(crate) status:     Status,
    pub(crate) scheduled:  Option<String>,
    pub(crate) hidden:     Option<String>,
    pub(crate) due:        Option<String>,
    pub(crate) priority:   Option<Priority>,
}

impl<'a> Partial<'a> for TodoHeader {
    const LOCATION: &'static str = "todo";
    type Output                  = Self;
}

pub trait Todo {
    fn is_todo(&self)                                     -> Result<bool>;
    fn get_uuid(&self)                                    -> Result<Uuid>;
    fn get_status(&self)                                  -> Result<Status>;
    fn set_status(&mut self, status: Status)              -> Result<()>;
    fn get_scheduled(&self)                               -> Result<Option<NaiveDateTime>>;
    fn set_scheduled(&mut self, scheduled: NaiveDateTime) -> Result<()>;
    fn get_hidden(&self)                                  -> Result<Option<NaiveDateTime>>;
    fn set_hidden(&mut self, hidden: NaiveDateTime)       -> Result<()>;
    fn get_due(&self)                                     -> Result<Option<NaiveDateTime>>;
    fn set_due(&mut self, due: NaiveDateTime)             -> Result<()>;
    fn get_priority(&self)                                -> Result<Option<Priority>>;
    fn set_priority(&mut self, prio: Priority)            -> Result<()>;
}

provide_kindflag_path!(pub IsTodo, "todo.is_todo");

impl Todo for Entry {
    fn is_todo(&self) -> Result<bool> {
        self.is::<IsTodo>().context("Cannot check whether Entry is a todo").map_err(From::from)
    }

    fn get_uuid(&self) -> Result<Uuid> {
        get_header(self).map(|hdr| hdr.uuid)
    }

    fn get_status(&self) -> Result<Status> {
        get_header(self).map(|hdr| hdr.status)
    }

    fn set_status(&mut self, status: Status) -> Result<()> {
        self.get_header_mut().insert_serialized("todo.status", status)?;
        Ok(())
    }

    fn get_scheduled(&self) -> Result<Option<NaiveDateTime>> {
        get_optional_ndt(self, |hdr| hdr.scheduled)
    }

    fn set_scheduled(&mut self, scheduled: NaiveDateTime) -> Result<()> {
        self.get_header_mut().insert_serialized("todo.scheduled", scheduled)?;
        Ok(())
    }

    fn get_hidden(&self) -> Result<Option<NaiveDateTime>> {
        get_optional_ndt(self, |hdr| hdr.hidden)
    }

    fn set_hidden(&mut self, hidden: NaiveDateTime) -> Result<()> {
        self.get_header_mut().insert_serialized("todo.hidden", hidden)?;
        Ok(())
    }

    fn get_due(&self) -> Result<Option<NaiveDateTime>> {
        get_optional_ndt(self, |hdr| hdr.due)
    }

    fn set_due(&mut self, due: NaiveDateTime) -> Result<()> {
        self.get_header_mut().insert_serialized("todo.due", due)?;
        Ok(())
    }

    fn get_priority(&self) -> Result<Option<Priority>> {
        get_header(self).map(|hdr| hdr.priority)
    }

    fn set_priority(&mut self, priority: Priority) -> Result<()> {
        self.get_header_mut().insert_serialized("todo.priority", priority)?;
        Ok(())
    }

}

fn get_header(entry: &Entry) -> Result<TodoHeader> {
    entry.get_header()
        .read_partial::<TodoHeader>()?
        .ok_or_else(|| {
            format_err!("{} does not contain a TODO header", entry.get_location())
        })
}

fn get_optional_ndt<F>(entry: &Entry, extractor: F)
    -> Result<Option<NaiveDateTime>>
    where F: FnOnce(TodoHeader) -> Option<String>
{
    get_header(entry).map(extractor)?.map(datetime_from_string).transpose().map_err(Error::from)
}
