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

use std::path::PathBuf;

use libimagstore::store::Store;
use libimagstore::store::FileLockEntry;
use libimagstore::iter::Entries;
use libimagentrylink::linkable::Linkable;

use failure::Fallible as Result;
use failure::Error;
use failure::err_msg;
use failure::ResultExt;

pub struct Wiki<'a, 'b>(&'a Store, &'b str);

/// An interface for accessing, creating and deleting "wiki pages"
///
/// Wiki pages are normal entries with some details added.
///
///
/// # Details
///
/// Entries are automatically linked to the "index" page when created and retrieved.
///
impl<'a, 'b> Wiki<'a, 'b> {

    pub(crate) fn new(store: &'a Store, name: &'b str) -> Wiki<'a, 'b> {
        Wiki(store, name)
    }

    pub(crate) fn create_index_page(&self) -> Result<FileLockEntry<'a>> {
        let path = PathBuf::from(format!("{}/index", self.1));
        let sid  = crate::module_path::new_id(path)?;

        self.0.create(sid)
    }

    pub(crate) fn get_index_page(&self) -> Result<FileLockEntry<'a>> {
        let path = PathBuf::from(format!("{}/index", self.1));
        let sid  = crate::module_path::new_id(path)?;

        self.0
            .get(sid)
            .context("Cannot get ID from store")
            .map_err(Error::from)?
            .ok_or_else(|| Error::from(err_msg("Missing index")))
    }

    pub fn get_entry<EN: AsRef<str>>(&self, entry_name: EN) -> Result<Option<FileLockEntry<'a>>> {
        let path = PathBuf::from(format!("{}/{}", self.1, entry_name.as_ref()));
        let sid  = crate::module_path::new_id(path)?;
        self.0.get(sid)
    }

    pub fn create_entry<EN: AsRef<str>>(&self, entry_name: EN) -> Result<FileLockEntry<'a>> {
        let path = PathBuf::from(format!("{}/{}", self.1, entry_name.as_ref()));
        let sid  = crate::module_path::new_id(path)?;
        let mut index = self
            .get_entry("index")?
            .ok_or_else(|| err_msg("Missing index page"))?;
        let mut entry = self.0.create(sid)?;

        entry.add_link(&mut index).map(|_| entry)
    }

    pub fn retrieve_entry<EN: AsRef<str>>(&self, entry_name: EN) -> Result<FileLockEntry<'a>> {
        let path = PathBuf::from(format!("{}/{}", self.1, entry_name.as_ref()));
        let sid  = crate::module_path::new_id(path)?;
        let mut index = self
            .get_entry("index")?
            .ok_or_else(|| err_msg("Missing index page"))?;
        let mut entry = self.0.retrieve(sid)?;

        entry.add_link(&mut index).map(|_| entry)
    }

    pub fn all_ids(&self) -> Result<Entries<'a>> {
        self.0.entries()?.in_collection("wiki")
    }

    pub fn delete_entry<EN: AsRef<str>>(&self, entry_name: EN) -> Result<()> {
        let path = PathBuf::from(format!("{}/{}", self.1, entry_name.as_ref()));
        let sid  = crate::module_path::new_id(path)?;
        self.0.delete(sid)
    }
}

