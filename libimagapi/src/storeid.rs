//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015, 2016 Matthias Beyer <mail@beyermatthias.de> and contributors
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

use uuid::Uuid;

use libimagstore::storeid::StoreId;

use cache::Cache;

#[derive(PartialOrd, Ord, PartialEq, Eq, Clone, Debug)]
pub struct StoreIdHandle(Uuid);

impl ToString for StoreIdHandle {
    fn to_string(&self) -> String {
        self.0.simple().to_string()
    }
}

impl StoreIdHandle {

    // The functions which can be executed on the cached object.

}

pub struct StoreIdCache(Cache<StoreIdHandle, StoreId>);

impl StoreIdCache {

    /// This is intensionally private.
    fn new() -> StoreIdCache {
        StoreIdCache(Cache::new())
    }

}

lazy_static! {
    pub static ref STORE_ID_CACHE: StoreIdCache = StoreIdCache::new();
}


