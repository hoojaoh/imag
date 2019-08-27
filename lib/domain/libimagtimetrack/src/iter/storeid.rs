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

use chrono::naive::NaiveDateTime as NDT;
use failure::Fallible as Result;
use failure::Error;

use crate::constants::*;
use crate::iter::tag::TagIter;
use crate::iter::create::CreateTimeTrackIter;

use libimagstore::store::Store;
use libimagstore::storeid::StoreId;

pub struct TagStoreIdIter {
    inner: TagIter,
    datetime: NDT,
}

impl TagStoreIdIter {

    pub fn new(inner: TagIter, datetime: NDT) -> TagStoreIdIter {
        TagStoreIdIter { inner, datetime }
    }

    pub fn create_entries(self, store: &Store) -> CreateTimeTrackIter<'_> {
        CreateTimeTrackIter::new(self, store)
    }

}

impl Iterator for TagStoreIdIter {
    type Item = Result<(StoreId, NDT)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|res| res.and_then(|tag| {
                let dt     = self.datetime.format(DATE_TIME_FORMAT).to_string();
                let id_str = format!("{}-{}", dt, tag.as_str());
                crate::module_path::new_id(id_str)
                    .map_err(Error::from)
                    .map(|id| (id, self.datetime))
            }))
    }
}

