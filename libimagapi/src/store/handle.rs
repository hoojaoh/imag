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

use std::cmp::Ordering;
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Error as FmtError;
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use std::path::PathBuf;
use std::result::Result as RResult;

use sha1::Sha1;
use toml::Value;

use libimagerror::into::IntoError;
use libimagstore::store::Store;

use cache::Cache;
use store::cache::STORE_CACHE;
use error::ApiErrorKind as AEK;
use error::MapErrInto;
use handle::Handle;
use result::Result;

#[derive(Clone)]
pub struct StoreHandle(Sha1);

impl Deref for StoreHandle {
    type Target = Sha1;

    fn deref(&self) -> &Sha1 {
        &self.0
    }
}

impl PartialEq for StoreHandle {
    fn eq(&self, other: &StoreHandle) -> bool {
        self.0.digest().bytes().eq(&other.digest().bytes())
    }
}

impl PartialOrd for StoreHandle {

    fn partial_cmp(&self, other: &StoreHandle) -> Option<Ordering> {
        self.0.digest().bytes().partial_cmp(&other.digest().bytes())
    }

}

impl Ord for StoreHandle {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.digest().bytes().cmp(&other.digest().bytes())
    }
}

impl Eq for StoreHandle {
    // This is empty
}

impl Debug for StoreHandle {
    fn fmt(&self, f: &mut Formatter) -> RResult<(), FmtError> {
        write!(f, "StoreHandle({:?})", self.0.digest().bytes())
    }
}

impl Handle for StoreHandle {
    fn to_string(&self) -> Result<String> {
        ::std::str::from_utf8(&self.0.digest().bytes())
            .map(String::from)
            .map_err_into(AEK::HandleToStringError)
    }
}

impl StoreHandle {

    fn from_path(loc: &PathBuf) -> Result<StoreHandle> {
        loc.to_str()
            .ok_or_else(|| AEK::HandleInstantiationError.into_error())
            .map(|buf| {
                let mut sha = Sha1::new();
                sha.update(buf.as_bytes());
                StoreHandle(sha)
            })
    }

    // The functions which can be executed on the cached object.

    pub fn new(location: PathBuf, store_config: Option<Value>) -> Result<StoreHandle> {
        let handle = try!(StoreHandle::from_path(&location));

        STORE_CACHE.lock()
            .map_err_into(AEK::CacheLockError)
            .and_then(|cache| {
                if cache.contains_key(&handle) {
                    Err(AEK::ResourceInUse.into_error())
                } else {
                    Store::new(location, store_config)
                        .map(|s| cache.insert(handle.clone(), s))
                        .map(|_| handle)
                }
            })

    }

}

