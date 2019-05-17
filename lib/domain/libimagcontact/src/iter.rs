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

use libimagstore::storeid::StoreIdIterator;
use libimagstore::store::Store;
use libimagstore::store::FileLockEntry;
use libimagerror::errors::ErrorMsg as EM;

use crate::contact::Contact;
use failure::Fallible as Result;
use failure::Error;
use failure::ResultExt;

pub struct ContactIter<'a>(StoreIdIterator, &'a Store);

/// Iterator over contacts
impl<'a> ContactIter<'a> {

    pub fn new(sii: StoreIdIterator, store: &'a Store) -> ContactIter<'a> {
        ContactIter(sii, store)
    }

}

impl<'a> Iterator for ContactIter<'a> {
    type Item = Result<FileLockEntry<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.next() {
                None          => return None,
                Some(Err(e))  => {
                    let e = Err(e)
                        .context("Found error while iterating over contacts")
                        .map_err(Error::from);

                    return Some(e)
                },
                Some(Ok(sid)) => match self.1.get(sid.clone()).map_err(From::from) {
                    Err(e)          => return Some(Err(e)),
                    Ok(None)        => return
                        Some(Err(Error::from(EM::EntryNotFound(sid.local_display_string())))),
                    Ok(Some(entry)) => match entry.is_contact()
                        .context("Cannot check whether entry is a contact")
                        .map_err(Error::from)
                    {
                        Ok(true)    => return Some(Ok(entry)),
                        Ok(false)   => continue,
                        Err(e)      => return Some(Err(e)),
                    },

                },
            }
        }
    }

}

