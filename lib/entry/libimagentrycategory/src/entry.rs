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

use toml_query::insert::TomlValueInsertExt;
use toml_query::read::TomlValueReadExt;
use toml_query::read::TomlValueReadTypeExt;
use toml::Value;

use libimagstore::store::Entry;
use libimagentrylink::linker::Linkable;
use libimagerror::errors::ErrorMsg as EM;

use failure::Fallible as Result;
use failure::ResultExt;
use failure::Error;
use failure::err_msg;
use crate::store::CategoryStore;

pub trait EntryCategory {

    fn set_category(&mut self, s: &str) -> Result<()>;

    fn set_category_checked(&mut self, register: &CategoryStore, s: &str) -> Result<()>;

    fn get_category(&self) -> Result<String>;

    fn has_category(&self) -> Result<bool>;

    fn remove_category(&mut self) -> Result<()>;

}

impl EntryCategory for Entry {

    fn set_category(&mut self, s: &str) -> Result<()> {
        trace!("Setting category '{}' UNCHECKED", s);
        self.get_header_mut()
            .insert(&String::from("category.value"), Value::String(s.to_string()))
            .context(format_err!("Failed to insert header at 'category.value' of '{}'", self.get_location()))
            .context(EM::EntryHeaderWriteError)
            .map_err(Error::from)
            .map(|_| ())
    }

    /// Check whether a category exists before setting it.
    ///
    /// This function should be used by default over EntryCategory::set_category()!
    fn set_category_checked(&mut self, register: &CategoryStore, s: &str) -> Result<()> {
        trace!("Setting category '{}' checked", s);
        let mut category = register
            .get_category_by_name(s)?
            .ok_or_else(|| Error::from(err_msg("Category does not exist")))?;

        let _ = self.set_category(s)?;
        let _ = self.add_link(&mut category)?;

        Ok(())
    }

    fn get_category(&self) -> Result<String> {
        trace!("Getting category from '{}'", self.get_location());
        self.get_header()
            .read_string("category.value")?
            .ok_or_else(|| Error::from(err_msg("Category name missing")))
    }

    fn has_category(&self) -> Result<bool> {
        trace!("Has category? '{}'", self.get_location());
        self.get_header()
            .read("category.value")
            .context(format_err!("Failed to read header at 'category.value' of '{}'", self.get_location()))
            .context(EM::EntryHeaderReadError)
            .map_err(Error::from)
            .map(|x| x.is_some())
    }

    /// Remove the category setting
    ///
    /// # Warning
    ///
    /// This does _only_ remove the category setting in the header. This does _not_ remove the
    /// internal link to the category entry, nor does it remove the category from the store.
    fn remove_category(&mut self) -> Result<()> {
        use toml_query::delete::TomlValueDeleteExt;

        self.get_header_mut()
            .delete("category.value")
            .context(format_err!("Failed to delete header at 'category.value' of '{}'", self.get_location()))
            .context(EM::EntryHeaderWriteError)
            .map_err(Error::from)
            .map(|_| ())
    }

}
