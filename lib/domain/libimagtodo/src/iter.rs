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
use failure::Error;
use filters::failable::filter::FailableFilter;

use libimagstore::store::FileLockEntry;
use libimagstore::store::Entry;

use crate::entry::Todo;
use crate::status::Status;
use crate::priority::Priority;

/// Iterator adaptor which filters an Iterator<Item = FileLockEntry> so that only todos are left
pub struct OnlyTodos<'a>(Box<dyn Iterator<Item = FileLockEntry<'a>>>);

impl<'a> OnlyTodos<'a> {
    pub fn new(it: Box<dyn Iterator<Item = FileLockEntry<'a>>>) -> Self {
        OnlyTodos(it)
    }
}

impl<'a> Iterator for OnlyTodos<'a> {
    type Item = Result<FileLockEntry<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.0.next() {
            match next.is_todo() {
                Ok(true)  => return Some(Ok(next)),
                Ok(false) => continue,
                Err(e)    => return Some(Err(e)),
            }
        }

        None
    }
}

/// Helper filter type
///
/// Can be used to filter an Iterator<Item = FileLockEntry> of Todos by status
///
pub struct StatusFilter(Status);

impl FailableFilter<Entry> for StatusFilter {
    type Error = Error;

    fn filter(&self, entry: &Entry) -> RResult<bool, Self::Error> {
        Ok(entry.get_status()? == self.0)
    }
}

/// Helper filter type
///
/// Can be used to filter an Iterator<Item = FileLockEntry> of Todos for scheduled todos
///
pub struct IsScheduledFilter;

impl FailableFilter<Entry> for IsScheduledFilter {
    type Error = Error;

    fn filter(&self, entry: &Entry) -> RResult<bool, Self::Error> {
        entry.get_scheduled().map(|s| s.is_some())
    }
}

/// Helper filter type
///
/// Can be used to filter an Iterator<Item = FileLockEntry> of Todos for hidden todos
///
pub struct IsHiddenFilter;

impl FailableFilter<Entry> for IsHiddenFilter {
    type Error = Error;

    fn filter(&self, entry: &Entry) -> RResult<bool, Self::Error> {
        entry.get_hidden().map(|s| s.is_some())
    }
}


/// Helper filter type
///
/// Can be used to filter an Iterator<Item = FileLockEntry> of Todos for due todos
///
pub struct IsDueFilter;

impl FailableFilter<Entry> for IsDueFilter {
    type Error = Error;

    fn filter(&self, entry: &Entry) -> RResult<bool, Self::Error> {
        entry.get_due().map(|s| s.is_some())
    }
}


/// Helper filter type
///
/// Can be used to filter an Iterator<Item = FileLockEntry> of Todos for priority
///
/// # Warning
///
/// If no priority is set for the entry, this filters out the entry
///
pub struct PriorityFilter(Priority);

impl FailableFilter<Entry> for PriorityFilter {
    type Error = Error;

    fn filter(&self, entry: &Entry) -> RResult<bool, Self::Error> {
        Ok(entry.get_priority()?.map(|p| p == self.0).unwrap_or(false))
    }
}

