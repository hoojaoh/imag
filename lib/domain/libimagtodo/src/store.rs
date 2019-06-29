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

use std::result::Result as RResult;

use failure::Fallible as Result;
use chrono::NaiveDateTime;
use uuid::Uuid;
use toml_query::insert::TomlValueInsertExt;

use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;
use libimagstore::iter::Entries;
use libimagutil::date::datetime_to_string;
use libimagentryutil::isa::Is;

use crate::status::Status;
use crate::priority::Priority;
use crate::entry::TodoHeader;
use crate::entry::IsTodo;

pub trait TodoStore<'a> {
    fn create_todo(&'a self,
                   status: Status,
                   scheduled: Option<NaiveDateTime>,
                   hidden: Option<NaiveDateTime>,
                   due: Option<NaiveDateTime>,
                   prio: Option<Priority>,
                   check_sanity: bool) -> Result<FileLockEntry<'a>>;

    fn get_todo_by_uuid(&'a self, uuid: &Uuid) -> Result<Option<FileLockEntry<'a>>>;

    fn get_todos(&self) -> Result<Entries>;
}

impl<'a> TodoStore<'a> for Store {

    /// Create a new todo entry
    ///
    /// # Warning
    ///
    /// If check_sanity is set to false, this does not sanity-check the scheduled/hidden/due dates.
    /// This might result in unintended behaviour (hidden after due date, scheduled before hidden
    /// date... etc)
    ///
    /// An user of this function might want to use `date_sanity_check()` to perform sanity checks
    /// before calling TodoStore::create_todo() and show the Err(String) as a warning to user in an
    /// interactive way.
    fn create_todo(&'a self,
                   status: Status,
                   scheduled: Option<NaiveDateTime>,
                   hidden: Option<NaiveDateTime>,
                   due: Option<NaiveDateTime>,
                   prio: Option<Priority>,
                   check_sanity: bool) -> Result<FileLockEntry<'a>>
    {
        if check_sanity {
            trace!("Checking sanity before creating todo");
            if let Err(s) = date_sanity_check(scheduled.as_ref(), hidden.as_ref(), due.as_ref()) {
                trace!("Not sane.");
                return Err(format_err!("{}", s))
            }
        }

        let uuid = Uuid::new_v4();
        let uuid_s = format!("{}", uuid.to_hyphenated_ref()); // TODO: not how it is supposed to be
        debug!("Created new UUID for todo = {}", uuid_s);

        let mut entry = crate::module_path::new_id(uuid_s).and_then(|id| self.create(id))?;

        let header = TodoHeader {
            uuid,
            status,
            scheduled: scheduled.as_ref().map(datetime_to_string),
            hidden: hidden.as_ref().map(datetime_to_string),
            due: due.as_ref().map(datetime_to_string),
            priority: prio
        };

        debug!("Created header for todo: {:?}", header);

        let _ = entry.get_header_mut().insert_serialized("todo", header)?;
        let _ = entry.set_isflag::<IsTodo>()?;

        Ok(entry)
    }

    fn get_todo_by_uuid(&'a self, uuid: &Uuid) -> Result<Option<FileLockEntry<'a>>> {
        let uuid_s = format!("{}", uuid.to_hyphenated_ref()); // TODO: not how it is supposed to be
        debug!("Created new UUID for todo = {}", uuid_s);
        let id = crate::module_path::new_id(uuid_s)?;
        self.get(id)
    }

    /// Get all todos using Store::entries()
    fn get_todos(&self) -> Result<Entries> {
        self.entries().and_then(|es| es.in_collection("todo"))
    }
}

/// Perform a sanity check on the scheduled/hidden/due dates
///
/// This function returns a String as error, which can be shown as a warning to the user or as an
/// error.
pub fn date_sanity_check(scheduled: Option<&NaiveDateTime>,
                         hidden: Option<&NaiveDateTime>,
                         due: Option<&NaiveDateTime>)
    -> RResult<(), String>
{
    if let (Some(sched), Some(hid)) = (scheduled.as_ref(), hidden.as_ref()) {
        if sched > hid {
            return Err(format!("Scheduled date after hidden date: {s}, {h}",
                               s = sched,
                               h = hid))
        }
    }

    if let (Some(hid), Some(due)) = (hidden.as_ref(), due.as_ref()) {
        if hid > due {
            return Err(format!("Hidden date after due date: {h}, {d}",
                               h = hid,
                               d = due))
        }
    }

    if let (Some(sched), Some(due)) = (scheduled.as_ref(), due.as_ref()) {
        if sched > due {
            return Err(format!("Scheduled date after due date: {s}, {d}",
                               s = sched,
                               d = due))
        }
    }

    Ok(())
}

