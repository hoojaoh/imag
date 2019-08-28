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
use libimagstore::store::Store;
use libimagstore::storeid::StoreIdIterator;
use libimagentrylink::linkable::Linkable;

use toml_query::read::TomlValueReadTypeExt;

use failure::Fallible as Result;
use failure::ResultExt;
use failure::Error;
use failure::err_msg;
use crate::store::CATEGORY_REGISTER_NAME_FIELD_PATH;
use crate::iter::CategoryEntryIterator;

provide_kindflag_path!(pub IsCategory, "category.is_category");

pub trait Category {
    fn is_category(&self) -> Result<bool>;
    fn get_name(&self)    -> Result<String>;
    fn get_entries<'a>(&self, store: &'a Store) -> Result<CategoryEntryIterator<'a>>;
}

impl Category for Entry {
    fn is_category(&self) -> Result<bool> {
        self.is::<IsCategory>()
    }

    fn get_name(&self) -> Result<String> {
        trace!("Getting category name of '{:?}'", self.get_location());
        self.get_header()
            .read_string(CATEGORY_REGISTER_NAME_FIELD_PATH)
            .context(format_err!("Failed to read header at '{}'", CATEGORY_REGISTER_NAME_FIELD_PATH))
            .map_err(Error::from)?
            .ok_or_else(|| err_msg("Category name missing"))
    }

    fn get_entries<'a>(&self, store: &'a Store) -> Result<CategoryEntryIterator<'a>> {
        trace!("Getting linked entries for category '{:?}'", self.get_location());
        let sit  = self.links()?.map(|l| l.get_store_id().clone()).map(Ok);
        let sit  = StoreIdIterator::new(Box::new(sit));
        let name = self.get_name()?;
        Ok(CategoryEntryIterator::new(store, sit, name))
    }
}

