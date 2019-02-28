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

macro_rules! mk_iterator_mod {
    {
        modname   = $modname:ident,
        itername  = $itername:ident,
        iteryield = $yield:ty,
        extname   = $extname:ident,
        extfnname = $extfnname:ident,
        fun       = $fun:expr
    } => {
        pub mod $modname {
            use crate::storeid::StoreId;
            #[allow(unused_imports)]
            use crate::store::FileLockEntry;
            use crate::store::Store;
            use failure::Fallible as Result;

            pub struct $itername<'a>(Box<Iterator<Item = Result<StoreId>> + 'a>, &'a Store);

            impl<'a> $itername<'a>
            {
                pub fn new(inner: Box<Iterator<Item = Result<StoreId>> + 'a>, store: &'a Store) -> Self {
                    $itername(inner, store)
                }
            }

            impl<'a> Iterator for $itername<'a>
            {
                type Item = Result<$yield>;

                fn next(&mut self) -> Option<Self::Item> {
                    self.0.next().map(|id| $fun(id?, self.1))
                }
            }

            pub trait $extname<'a> {
                fn $extfnname(self, store: &'a Store) -> $itername<'a>;
            }

            impl<'a, I> $extname<'a> for I
                where I: Iterator<Item = Result<StoreId>> + 'a
            {
                fn $extfnname(self, store: &'a Store) -> $itername<'a> {
                    $itername(Box::new(self), store)
                }
            }
        }
    }
}

mk_iterator_mod! {
    modname   = create,
    itername  = StoreCreateIterator,
    iteryield = FileLockEntry<'a>,
    extname   = StoreIdCreateIteratorExtension,
    extfnname = into_create_iter,
    fun       = |id: StoreId, store: &'a Store| store.create(id)
}

mk_iterator_mod! {
    modname   = delete,
    itername  = StoreDeleteIterator,
    iteryield = (),
    extname   = StoreIdDeleteIteratorExtension,
    extfnname = into_delete_iter,
    fun       = |id: StoreId, store: &'a Store| store.delete(id)
}

mk_iterator_mod! {
    modname   = get,
    itername  = StoreGetIterator,
    iteryield = Option<FileLockEntry<'a>>,
    extname   = StoreIdGetIteratorExtension,
    extfnname = into_get_iter,
    fun       = |id: StoreId, store: &'a Store| store.get(id)
}

mk_iterator_mod! {
    modname   = retrieve,
    itername  = StoreRetrieveIterator,
    iteryield = FileLockEntry<'a>,
    extname   = StoreIdRetrieveIteratorExtension,
    extfnname = into_retrieve_iter,
    fun       = |id: StoreId, store: &'a Store| store.retrieve(id)
}

#[cfg(test)]
#[allow(dead_code)]
mod compile_test {

    // This module contains code to check whether this actually compiles the way we would like it to
    // compile

    use crate::store::Store;
    use crate::storeid::StoreId;

    fn store() -> Store {
        unimplemented!("Not implemented because in compile-test")
    }

    fn test_compile_get() {
        let store = store();
        let _ = store
            .entries()
            .unwrap()
            .into_get_iter();
    }

    fn test_compile_get_result() {
        fn to_result(e: StoreId) -> Result<StoreId, ()> {
            Ok(e)
        }

        let store = store();
        let _ = store
            .entries()
            .unwrap()
            .into_get_iter();
    }
}

use crate::storeid::StoreId;
use crate::storeid::StoreIdIterator;
use self::delete::StoreDeleteIterator;
use self::get::StoreGetIterator;
use self::retrieve::StoreRetrieveIterator;
use crate::file_abstraction::iter::PathIterator;
use crate::store::Store;
use failure::Fallible as Result;

/// Iterator for iterating over all (or a subset of all) entries
///
/// The iterator now has functionality to optimize the iteration, if only a subdirectory of the
/// store is required, for example `$STORE/foo`.
///
/// This is done via functionality where the underlying iterator gets
/// altered.
///
/// As the (for the filesystem backend underlying) `walkdir::WalkDir` type is not as nice as it
/// could be, iterating over two subdirectories with one iterator is not possible. Thus, iterators
/// for two collections in the store should be build like this (untested):
///
/// ```ignore
///     store
///         .entries()?
///         .in_collection("foo")
///         .chain(store.entries()?.in_collection("bar"))
/// ```
///
/// Functionality to exclude subdirectories is not possible with the current implementation and has
/// to be done during iteration, with filtering (as usual).
pub struct Entries<'a>(PathIterator<'a>, &'a Store);

impl<'a> Entries<'a> {

    pub(crate) fn new(pi: PathIterator<'a>, store: &'a Store) -> Self {
        Entries(pi, store)
    }

    pub fn in_collection(self, c: &str) -> Self {
        Entries(self.0.in_collection(c), self.1)
    }

    /// Turn `Entries` iterator into generic `StoreIdIterator`
    ///
    /// # TODO
    ///
    /// Revisit whether this can be done in a cleaner way. See commit message for why this is
    /// needed.
    pub fn into_storeid_iter(self) -> StoreIdIterator {
        use crate::storeid::StoreIdWithBase;
        use crate::storeid::IntoStoreId;

        let storepath = self.1.path().to_path_buf();

        let iter = self.0
            .into_inner()
            .map(move |r| {
                r.and_then(|path| {
                    StoreIdWithBase::from_full_path(&storepath, path)?.into_storeid()
                })
            });
        StoreIdIterator::new(Box::new(iter))
    }

    /// Transform the iterator into a StoreDeleteIterator
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_delete_iter(self) -> StoreDeleteIterator<'a> {
        StoreDeleteIterator::new(Box::new(self.0.map(|r| r.map(|id| id.without_base()))), self.1)
    }

    /// Transform the iterator into a StoreGetIterator
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_get_iter(self) -> StoreGetIterator<'a> {
        StoreGetIterator::new(Box::new(self.0.map(|r| r.map(|id| id.without_base()))), self.1)
    }

    /// Transform the iterator into a StoreRetrieveIterator
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_retrieve_iter(self) -> StoreRetrieveIterator<'a> {
        StoreRetrieveIterator::new(Box::new(self.0.map(|r| r.map(|id| id.without_base()))), self.1)
    }

}

impl<'a> Iterator for Entries<'a> {
    type Item = Result<StoreId>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|r| r.map(|id| id.without_base()))
    }
}

#[cfg(test)]
mod tests {
    extern crate env_logger;

    use std::path::PathBuf;
    use std::sync::Arc;

    fn setup_logging() {
        let _ = env_logger::try_init();
    }

    use crate::store::Store;
    use crate::storeid::StoreId;
    use crate::file_abstraction::inmemory::InMemoryFileAbstraction;
    use libimagutil::variants::generate_variants;

    pub fn get_store() -> Store {
        let backend = Arc::new(InMemoryFileAbstraction::default());
        Store::new_with_backend(PathBuf::from("/"), &None, backend).unwrap()
    }

    #[test]
    fn test_entries_iterator_in_collection() {
        setup_logging();
        let store = get_store();

        let ids = {
            let base = String::from("entry");
            let variants = vec!["coll_1", "coll_2", "coll_3"];
            let modifier = |base: &String, v: &&str| {
                StoreId::new(PathBuf::from(format!("{}/{}", *v, base))).unwrap()
            };

            generate_variants(&base, variants.iter(), &modifier)
        };

        for id in ids {
            let _ = store.retrieve(id).unwrap();
        }

        let succeeded = store.entries()
            .unwrap()
            .in_collection("coll_3")
            .map(|id| { debug!("Processing id = {:?}", id); id })
            .all(|id| id.unwrap().is_in_collection(&["coll_3"]));

        assert!(succeeded, "not all entries in iterator are from coll_3 collection");
    }
}

