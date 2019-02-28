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

use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;
use libimagstore::storeid::StoreId;

use failure::Fallible as Result;

use crate::wiki::Wiki;

pub trait WikiStore {

    fn get_wiki<'a, 'b>(&'a self, name: &'b str) -> Result<Option<Wiki<'a, 'b>>>;

    fn create_wiki<'a, 'b>(&'a self, name: &'b str)
        -> Result<(Wiki<'a, 'b>, FileLockEntry<'a>)>;

    fn retrieve_wiki<'a, 'b>(&'a self, name: &'b str)
        -> Result<(Wiki<'a, 'b>, FileLockEntry<'a>)>;

}

impl WikiStore for Store {

    /// get a wiki by its name
    fn get_wiki<'a, 'b>(&'a self, name: &'b str) -> Result<Option<Wiki<'a, 'b>>> {
        if self.exists(wiki_path(name.as_ref())?)? {
            debug!("Building Wiki object");
            Ok(Some(Wiki::new(self, name)))
        } else {
            debug!("Cannot build wiki object: Wiki does not exist");
            Ok(None)
        }
    }

    /// Create a wiki.
    ///
    /// # Returns
    ///
    /// Returns the Wiki object.
    ///
    /// Ob success, an empty Wiki entry with the name `index` is created inside the wiki. Later, new
    /// entries are automatically linked to this entry.
    ///
    fn create_wiki<'a, 'b>(&'a self, name: &'b str) -> Result<(Wiki<'a, 'b>, FileLockEntry<'a>)> {
        debug!("Trying to get wiki '{}'", name);

        let wiki = Wiki::new(self, name);
        let index = wiki.create_index_page()?;
        Ok((wiki, index))
    }

    fn retrieve_wiki<'a, 'b>(&'a self, name: &'b str)
        -> Result<(Wiki<'a, 'b>, FileLockEntry<'a>)>
    {
        match self.get_wiki(name)? {
            None       => self.create_wiki(name),
            Some(wiki) => {
                let index = wiki.get_index_page()?;
                Ok((wiki, index))
            },
        }
    }

}

fn wiki_path(name: &str) -> Result<StoreId> {
    crate::module_path::new_id(name)
}

