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
use failure::err_msg;
use toml_query::insert::TomlValueInsertExt;
use uuid::Uuid;

use libimagstore::store::Store;
use libimagstore::store::FileLockEntry;
use libimagentryutil::isa::Is;
use libimagutil::date::datetime_to_string;

use crate::priority::Priority;
use crate::status::Status;
use crate::entry::IsTodo;
use crate::entry::TodoHeader;
use crate::store::date_sanity_check;


pub struct TodoBuilder {
    uuid: Option<Uuid>,
    status: Option<Status>,
    scheduled: Option<NaiveDateTime>,
    hidden: Option<NaiveDateTime>,
    due: Option<NaiveDateTime>,
    prio: Option<Priority>,
    check_sanity: bool,
}

impl TodoBuilder {
    pub(crate) fn new() -> Self {
        TodoBuilder {
            uuid:      None,
            status:    None,
            scheduled: None,
            hidden:    None,
            due:       None,
            prio:      None,
            check_sanity: true,
        }
    }

    pub fn build<'a>(self, store: &'a Store) -> Result<FileLockEntry<'a>> {
        let uuid   = self.uuid.ok_or_else(|| err_msg("Uuid missing"))?;
        let status = self.status.ok_or_else(|| err_msg("Status missing"))?;

        if self.check_sanity {
            trace!("Checking sanity before creating todo");
            if let Err(s) = date_sanity_check(self.scheduled.as_ref(), self.hidden.as_ref(), self.due.as_ref()) {
                trace!("Not sane.");
                return Err(format_err!("{}", s))
            }
        }

        let uuid_s = format!("{}", uuid.to_hyphenated_ref()); // TODO: not how it is supposed to be
        debug!("Created new UUID for todo = {}", uuid_s);

        let mut entry = crate::module_path::new_id(uuid_s).and_then(|id| store.create(id))?;

        let header = TodoHeader {
            uuid,
            status,
            scheduled: self.scheduled.as_ref().map(datetime_to_string),
            hidden: self.hidden.as_ref().map(datetime_to_string),
            due: self.due.as_ref().map(datetime_to_string),
            priority: self.prio
        };

        debug!("Created header for todo: {:?}", header);

        let _ = entry.get_header_mut().insert_serialized("todo", header)?;
        let _ = entry.set_isflag::<IsTodo>()?;

        Ok(entry)
    }

    pub fn with_uuid(mut self, uuid: Option<Uuid>) -> Self {
        self.uuid = uuid;
        self
    }

    pub fn with_status(mut self, status: Option<Status>) -> Self {
        self.status = status;
        self
    }

    pub fn with_scheduled(mut self, scheduled: Option<NaiveDateTime>) -> Self {
        self.scheduled = scheduled;
        self
    }

    pub fn with_hidden(mut self, hidden: Option<NaiveDateTime>) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn with_due(mut self, due: Option<NaiveDateTime>) -> Self {
        self.due = due;
        self
    }

    pub fn with_prio(mut self, prio: Option<Priority>) -> Self {
        self.prio = prio;
        self
    }

    pub fn with_check_sanity(mut self, b: bool) -> Self {
        self.check_sanity = b;
        self
    }

}
