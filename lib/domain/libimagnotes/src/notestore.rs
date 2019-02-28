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

use toml::Value;

use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;

use toml_query::insert::TomlValueInsertExt;
use failure::Fallible as Result;

use crate::iter::*;

pub trait NoteStore<'a> {
    fn new_note(&'a self, name: String, text: String) -> Result<FileLockEntry<'a>>;
    fn delete_note(&'a self, name: String) -> Result<()>;
    fn retrieve_note(&'a self, name: String) -> Result<FileLockEntry<'a>>;
    fn get_note(&'a self, name: String) -> Result<Option<FileLockEntry<'a>>>;
    fn all_notes(&'a self) -> Result<NoteIterator>;
}


impl<'a> NoteStore<'a> for Store {

    fn new_note(&'a self, name: String, text: String) -> Result<FileLockEntry<'a>> {
        use std::ops::DerefMut;

        debug!("Creating new Note: '{}'", name);
        let fle = {
            let mut lockentry = crate::module_path::new_id(name.clone()).and_then(|id| self.create(id))?;

            {
                let entry  = lockentry.deref_mut();
                entry.get_header_mut().insert("note.name", Value::String(name))?;
                *entry.get_content_mut() = text;
            }

            lockentry
        };

        Ok(fle)
    }

    fn delete_note(&'a self, name: String) -> Result<()> {
        crate::module_path::new_id(name).and_then(|id| self.delete(id))
    }

    fn retrieve_note(&'a self, name: String) -> Result<FileLockEntry<'a>> {
        crate::module_path::new_id(name).and_then(|id| self.retrieve(id))
    }

    fn get_note(&'a self, name: String) -> Result<Option<FileLockEntry<'a>>> {
        crate::module_path::new_id(name).and_then(|id| self.get(id))
    }

    fn all_notes(&'a self) -> Result<NoteIterator> {
        self.entries().map(|it| it.into_storeid_iter()).map(NoteIterator::new)
    }

}

