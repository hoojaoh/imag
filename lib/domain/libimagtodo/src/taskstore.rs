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

use std::io::BufRead;
use std::result::Result as RResult;

use toml::Value;
use toml::map::Map;
use uuid::Uuid;

use task_hookrs::task::Task as TTask;
use task_hookrs::import::{import_task, import_tasks};
use failure::Fallible as Result;
use failure::ResultExt;
use failure::Error;
use failure::err_msg;

use libimagstore::store::{FileLockEntry, Store};
use libimagerror::errors::ErrorMsg as EM;

use crate::iter::TaskIdIterator;

/// Task struct containing a `FileLockEntry`
pub trait TaskStore<'a> {
    fn import_task_from_reader<R: BufRead>(&'a self, r: R) -> Result<(FileLockEntry<'a>, String, Uuid)>;
    fn get_task_from_import<R: BufRead>(&'a self, r: R) -> Result<RResult<FileLockEntry<'a>, String>>;
    fn get_task_from_string(&'a self, s: String) -> Result<RResult<FileLockEntry<'a>, String>>;
    fn get_task_from_uuid(&'a self, uuid: Uuid) -> Result<Option<FileLockEntry<'a>>>;
    fn retrieve_task_from_import<R: BufRead>(&'a self, r: R) -> Result<FileLockEntry<'a>>;
    fn retrieve_task_from_string(&'a self, s: String) -> Result<FileLockEntry<'a>>;
    fn delete_tasks_by_imports<R: BufRead>(&self, r: R) -> Result<()>;
    fn delete_task_by_uuid(&self, uuid: Uuid) -> Result<()>;
    fn all_tasks(&self) -> Result<TaskIdIterator>;
    fn new_from_twtask(&'a self, task: TTask) -> Result<FileLockEntry<'a>>;
}

impl<'a> TaskStore<'a> for Store {

    fn import_task_from_reader<R: BufRead>(&'a self, mut r: R) -> Result<(FileLockEntry<'a>, String, Uuid)> {
        let mut line = String::new();
        r.read_line(&mut line).context(EM::UTF8Error)?;
        import_task(&line.as_str())
            .context(err_msg("Error importing"))
            .map_err(Error::from)
            .and_then(|t| {
                let uuid = t.uuid().clone();
                self.new_from_twtask(t).map(|t| (t, line, uuid))
            })
    }

    /// Get a task from an import string. That is: read the imported string, get the UUID from it
    /// and try to load this UUID from store.
    ///
    /// Possible return values are:
    ///
    /// * Ok(Ok(Task))
    /// * Ok(Err(String)) - where the String is the String read from the `r` parameter
    /// * Err(_)          - where the error is an error that happened during evaluation
    ///
    fn get_task_from_import<R: BufRead>(&'a self, mut r: R) -> Result<RResult<FileLockEntry<'a>, String>> {
        let mut line = String::new();
        r.read_line(&mut line).context(EM::UTF8Error)?;
        self.get_task_from_string(line)
    }

    /// Get a task from a String. The String is expected to contain the JSON-representation of the
    /// Task to get from the store (only the UUID really matters in this case)
    ///
    /// For an explanation on the return values see `Task::get_from_import()`.
    fn get_task_from_string(&'a self, s: String) -> Result<RResult<FileLockEntry<'a>, String>> {
        import_task(s.as_str())
            .context(err_msg("Import error"))
            .map_err(Error::from)
            .map(|t| t.uuid().clone())
            .and_then(|uuid| self.get_task_from_uuid(uuid))
            .and_then(|o| match o {
                None    => Ok(Err(s)),
                Some(t) => Ok(Ok(t)),
            })
    }

    /// Get a task from an UUID.
    ///
    /// If there is no task with this UUID, this returns `Ok(None)`.
    fn get_task_from_uuid(&'a self, uuid: Uuid) -> Result<Option<FileLockEntry<'a>>> {
        crate::module_path::new_id(format!("taskwarrior/{}", uuid)).and_then(|store_id| self.get(store_id))
    }

    /// Same as Task::get_from_import() but uses Store::retrieve() rather than Store::get(), to
    /// implicitely create the task if it does not exist.
    fn retrieve_task_from_import<R: BufRead>(&'a self, mut r: R) -> Result<FileLockEntry<'a>> {
        let mut line = String::new();
        r.read_line(&mut line).context(EM::UTF8Error)?;
        self.retrieve_task_from_string(line)
    }

    /// Retrieve a task from a String. The String is expected to contain the JSON-representation of
    /// the Task to retrieve from the store (only the UUID really matters in this case)
    fn retrieve_task_from_string(&'a self, s: String) -> Result<FileLockEntry<'a>> {
        self.get_task_from_string(s)
            .and_then(|opt| match opt {
                Ok(task)    => Ok(task),
                Err(string) => import_task(string.as_str())
                    .context(err_msg("Import error"))
                    .map_err(Error::from)
                    .and_then(|t| self.new_from_twtask(t)),
            })
    }

    fn delete_tasks_by_imports<R: BufRead>(&self, r: R) -> Result<()> {
        use task_hookrs::status::TaskStatus;

        for (counter, res_ttask) in import_tasks(r).into_iter().enumerate() {
            let ttask = res_ttask.context(err_msg("Import error"))?;

            if counter % 2 == 1 {
                // Only every second task is needed, the first one is the
                // task before the change, and the second one after
                // the change. The (maybe modified) second one is
                // expected by taskwarrior.
                //
                // Taskwarrior does not have the concept of deleted tasks, but only modified
                // ones.
                //
                // Here we check if the status of a task is deleted and if yes, we delete it
                // from the store.
                if *ttask.status() == TaskStatus::Deleted {
                    let _ = self.delete_task_by_uuid(*ttask.uuid())?;
                    info!("Deleted task {}", *ttask.uuid());
                }
            }
        }
        Ok(())
    }

    fn delete_task_by_uuid(&self, uuid: Uuid) -> Result<()> {
        crate::module_path::new_id(format!("taskwarrior/{}", uuid)).and_then(|id| self.delete(id))
    }

    fn all_tasks(&self) -> Result<TaskIdIterator> {
        self.entries().map(|i| TaskIdIterator::new(i))
    }

    fn new_from_twtask(&'a self, task: TTask) -> Result<FileLockEntry<'a>> {
        use toml_query::read::TomlValueReadExt;
        use toml_query::set::TomlValueSetExt;

        let uuid     = task.uuid();
        crate::module_path::new_id(format!("taskwarrior/{}", uuid)).and_then(|id| {
            self.retrieve(id).and_then(|mut fle| {
                {
                    let hdr = fle.get_header_mut();
                    if hdr.read("todo")?.is_none() {
                        hdr.set("todo", Value::Table(Map::new()))?;
                    }

                    hdr.set("todo.uuid", Value::String(format!("{}",uuid)))?;
                }

                // If none of the errors above have returned the function, everything is fine
                Ok(fle)
            })
        })

    }

}

