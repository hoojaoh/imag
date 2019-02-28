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

use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;

use std::fmt::{Display, Debug, Formatter};
use std::fmt::Error as FmtError;
use std::result::Result as RResult;
use std::path::Components;

use failure::ResultExt;
use failure::Fallible as Result;
use failure::err_msg;
use failure::Error;

use crate::store::Store;

use crate::iter::create::StoreCreateIterator;
use crate::iter::delete::StoreDeleteIterator;
use crate::iter::get::StoreGetIterator;
use crate::iter::retrieve::StoreRetrieveIterator;

/// The Index into the Store
///
/// A StoreId object is a unique identifier for one entry in the store which might be present or
/// not.
///
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StoreId(PathBuf);

impl StoreId {

    pub fn new(id: PathBuf) -> Result<StoreId> {
        debug!("Trying to get a new baseless id from: {:?}", id);
        if id.is_absolute() {
            debug!("Error: Id is absolute!");
            Err(format_err!("Store Id local part is absolute: {}", id.display()))
        } else {
            debug!("Building Storeid object baseless");
            Ok(StoreId(id))
        }
    }

    pub(crate) fn with_base<'a>(self, base: &'a PathBuf) -> StoreIdWithBase<'a> {
        StoreIdWithBase(base, self.0)
    }

    pub fn to_str(&self) -> Result<String> {
        Ok(self.0.display().to_string())
    }

    /// Helper function for creating a displayable String from StoreId
    ///
    /// This is safe because the
    ///
    /// ```ignore
    ///     impl<T: fmt::Display + ?Sized> ToString for T
    /// ```
    ///
    /// does only fail if Display::display() failed. The implementation of ::std::path::Display and
    /// the implementation ::std::fmt::Display for ::std::path::Display do not return errors though.
    pub fn local_display_string(&self) -> String {
        self.local().display().to_string()
    }

    /// Returns the components of the `id` part of the StoreId object.
    ///
    /// Can be used to check whether a StoreId points to an entry in a specific collection of
    /// StoreIds.
    pub fn components(&self) -> Components {
        self.0.components()
    }

    /// Get the _local_ part of a StoreId object, as in "the part from the store root to the entry".
    pub fn local(&self) -> &PathBuf {
        &self.0
    }

    /// Check whether a StoreId points to an entry in a specific collection.
    ///
    /// A "collection" here is simply a directory. So `foo/bar/baz` is an entry which is in
    /// collection ["foo", "bar", "baz"], but also in ["foo", "bar"] and ["foo"].
    ///
    /// # Warning
    ///
    /// The collection specification _has_ to start with the module name. Otherwise this function
    /// may return false negatives.
    ///
    pub fn is_in_collection<S: AsRef<str>, V: AsRef<[S]>>(&self, colls: &V) -> bool {
        use std::path::Component;

        self.0
            .components()
            .zip(colls.as_ref().iter())
            .all(|(component, pred_coll)| match component {
                Component::Normal(ref s) => s
                    .to_str()
                    .map(|ref s| s == &pred_coll.as_ref())
                    .unwrap_or(false),
                _ => false
            })
    }

    pub fn local_push<P: AsRef<Path>>(&mut self, path: P) {
        self.0.push(path)
    }

}

impl Display for StoreId {

    fn fmt(&self, fmt: &mut Formatter) -> RResult<(), FmtError> {
        write!(fmt, "{}", self.0.display())
    }

}

/// This Trait allows you to convert various representations to a single one
/// suitable for usage in the Store
pub trait IntoStoreId {
    fn into_storeid(self) -> Result<StoreId>;
}

impl IntoStoreId for StoreId {
    fn into_storeid(self) -> Result<StoreId> {
        Ok(self)
    }
}

impl IntoStoreId for PathBuf {
    fn into_storeid(self) -> Result<StoreId> {
        StoreId::new(self)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StoreIdWithBase<'a>(&'a PathBuf, PathBuf);

impl<'a> StoreIdWithBase<'a> {
    #[cfg(test)]
    pub(crate) fn new(base: &'a PathBuf, path: PathBuf) -> Self {
        StoreIdWithBase(base, path)
    }

    pub(crate) fn without_base(self) -> StoreId {
        StoreId(self.1)
    }

    /// Transform the StoreId object into a PathBuf, error if the base of the StoreId is not
    /// specified.
    pub(crate) fn into_pathbuf(self) -> Result<PathBuf> {
        let mut base = self.0.clone();
        base.push(self.1);
        Ok(base)
    }

    /// Try to create a StoreId object from a filesystem-absolute path.
    ///
    /// Automatically creates a StoreId object which has a `base` set to `store_part` if stripping
    /// the `store_part` from the `full_path` succeeded.
    pub(crate) fn from_full_path<D>(store_part: &'a PathBuf, full_path: D) -> Result<StoreIdWithBase<'a>>
        where D: Deref<Target = Path>
    {
        trace!("Creating StoreIdWithBase object from full path = {} with store_part = {}",
               full_path.display(),
               store_part.display());
        let p = full_path
            .strip_prefix(store_part)
            .map_err(Error::from)
            .context(err_msg("Error building Store Id from full path"))?;
        Ok(StoreIdWithBase(store_part, PathBuf::from(p)))
    }
}

impl<'a> IntoStoreId for StoreIdWithBase<'a> {
    fn into_storeid(self) -> Result<StoreId> {
        Ok(StoreId(self.1))
    }
}

impl<'a> Into<StoreId> for StoreIdWithBase<'a> {
    fn into(self) -> StoreId {
        StoreId(self.1)
    }
}

impl<'a> Display for StoreIdWithBase<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> RResult<(), FmtError> {
        write!(fmt, "{}/{}", self.0.display(), self.1.display())
    }
}


#[macro_export]
macro_rules! module_entry_path_mod {
    ($name:expr) => (
        #[allow(missing_docs,
                missing_copy_implementations,
                trivial_casts, trivial_numeric_casts,
                unstable_features,
                unused_import_braces, unused_qualifications,
                unused_imports)]
        /// A helper module to create valid module entry paths
        pub mod module_path {
            use std::convert::AsRef;
            use std::path::Path;
            use std::path::PathBuf;
            use $crate::storeid::StoreId;
            use failure::Fallible as Result;

            pub fn new_id<P: AsRef<Path>>(p: P) -> Result<StoreId> {

                let path_str = p
                    .as_ref()
                    .to_str()
                    .ok_or_else(|| {
                        format_err!("File path is not valid UTF-8: {}", p.as_ref().display())
                    })?;

                let id = format!("{}/{}", $name, path_str);

                StoreId::new(PathBuf::from(id))
            }

        }
    )
}

pub struct StoreIdIterator {
    iter: Box<Iterator<Item = Result<StoreId>>>,
}

impl Debug for StoreIdIterator {

    fn fmt(&self, fmt: &mut Formatter) -> RResult<(), FmtError> {
        write!(fmt, "StoreIdIterator")
    }

}

impl StoreIdIterator {

    pub fn new(iter: Box<Iterator<Item = Result<StoreId>>>) -> StoreIdIterator {
        StoreIdIterator { iter }
    }

    pub fn with_store<'a>(self, store: &'a Store) -> StoreIdIteratorWithStore<'a> {
        StoreIdIteratorWithStore(self, store)
    }

}

impl Iterator for StoreIdIterator {
    type Item = Result<StoreId>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

}

pub struct StoreIdIteratorWithStore<'a>(StoreIdIterator, &'a Store);

impl<'a> Deref for StoreIdIteratorWithStore<'a> {
    type Target = StoreIdIterator;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Iterator for StoreIdIteratorWithStore<'a> {
    type Item = Result<StoreId>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> StoreIdIteratorWithStore<'a> {

    pub fn new(iter: Box<Iterator<Item = Result<StoreId>>>, store: &'a Store) -> Self {
        StoreIdIteratorWithStore(StoreIdIterator::new(iter), store)
    }

    pub fn into_storeid_iter(self) -> StoreIdIterator {
        self.0
    }

    /// Transform the iterator into a StoreCreateIterator
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_create_iter(self) -> StoreCreateIterator<'a> {
        StoreCreateIterator::new(Box::new(self.0), self.1)
    }

    /// Transform the iterator into a StoreDeleteIterator
    ///
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_delete_iter(self) -> StoreDeleteIterator<'a> {
        StoreDeleteIterator::new(Box::new(self.0), self.1)
    }

    /// Transform the iterator into a StoreGetIterator
    ///
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_get_iter(self) -> StoreGetIterator<'a> {
        StoreGetIterator::new(Box::new(self.0), self.1)
    }

    /// Transform the iterator into a StoreRetrieveIterator
    ///
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_retrieve_iter(self) -> StoreRetrieveIterator<'a> {
        StoreRetrieveIterator::new(Box::new(self.0), self.1)
    }

}

#[cfg(test)]
mod test {
    module_entry_path_mod!("test");

    #[test]
    fn test_correct_path() {
        let p = crate::storeid::test::module_path::new_id("test");

        assert_eq!(p.unwrap().to_str().unwrap(), "test/test");
    }

    #[test]
    fn storeid_in_collection() {
        let p = crate::storeid::test::module_path::new_id("1/2/3/4/5/6/7/8/9/0").unwrap();

        assert!(p.is_in_collection(&["test", "1"]));
        assert!(p.is_in_collection(&["test", "1", "2"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4", "5"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4", "5", "6"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4", "5", "6", "7"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4", "5", "6", "7", "8"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4", "5", "6", "7", "8", "9"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0"]));

        assert!(!p.is_in_collection(&["test", "0", "2", "3", "4", "5", "6", "7", "8", "9", "0"]));
        assert!(!p.is_in_collection(&["test", "1", "2", "3", "4", "5", "6", "8"]));
        assert!(!p.is_in_collection(&["test", "1", "2", "3", "leet", "5", "6", "7"]));
    }

}
