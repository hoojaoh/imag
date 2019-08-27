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

//! BookmarkCollection module
//!
//! A BookmarkCollection is nothing more than a simple store entry. One can simply call functions
//! from the libimagentryurl::linker::UrlLinker trait on this to generate external links.
//!
//! The BookmarkCollection type offers helper functions to get all links or such things.

use regex::Regex;

use failure::Fallible as Result;
use failure::ResultExt;
use failure::Error;

use libimagstore::store::Store;
use libimagstore::store::Entry;
use libimagstore::store::FileLockEntry;
use libimagstore::storeid::StoreId;
use libimagentryurl::linker::UrlLinker;
use libimagentryurl::iter::UrlIter;
use libimagentrylink::linkable::Linkable;
use libimagentrylink::link::Link as StoreLink;

use crate::link::Link;

use self::iter::LinksMatchingRegexIter;

pub trait BookmarkCollectionStore<'a> {
    fn new(&'a self, name: &str)                     -> Result<FileLockEntry<'a>>;
    fn get(&'a self, name: &str)                     -> Result<Option<FileLockEntry<'a>>>;
    fn delete(&'a self, name: &str)                     -> Result<()>;
}

impl<'a> BookmarkCollectionStore<'a> for Store {

    #[allow(clippy::new_ret_no_self)]
    fn new(&'a self, name: &str) -> Result<FileLockEntry<'a>> {
        crate::module_path::new_id(name)
            .and_then(|id| self.create(id)
                      .context("Failed to create FileLockEntry")
                      .map_err(Error::from))
            .context("Failed to create Id for new Bookmark Collection")
            .map_err(Error::from)
    }

    fn get(&'a self, name: &str) -> Result<Option<FileLockEntry<'a>>> {
        crate::module_path::new_id(name)
            .and_then(|id| self.get(id)
                      .context("Failed to get FileLockEntry")
                      .map_err(Error::from))
            .context("Failed to get Bookmark Collection")
            .map_err(Error::from)
    }

    fn delete(&'a self, name: &str) -> Result<()> {
        crate::module_path::new_id(name)
            .and_then(|id| self.delete(id)
                      .context("Failed to delete FileLockEntry")
                      .map_err(Error::from))
            .context("Failed to delete Bookmark Collection")
            .map_err(Error::from)
    }

}

pub trait BookmarkCollection : Sized + Linkable + UrlLinker {
    fn get_links<'a>(&self, store: &'a Store)                    -> Result<UrlIter<'a>>;
    fn link_entries(&self)                                       -> Result<Vec<StoreLink>>;
    fn add_link(&mut self, store: &Store, l: Link)               -> Result<Vec<StoreId>>;
    fn get_links_matching<'a>(&self, store: &'a Store, r: Regex) -> Result<LinksMatchingRegexIter<'a>>;
    fn remove_link(&mut self, store: &Store, l: Link)            -> Result<Vec<StoreId>>;
}

impl BookmarkCollection for Entry {

    fn get_links<'a>(&self, store: &'a Store) -> Result<UrlIter<'a>> {
        self.get_urls(store)
    }

    #[allow(clippy::redundant_closure)]
    fn link_entries(&self) -> Result<Vec<StoreLink>> {
        use libimagentryurl::util::is_external_link_storeid;
        self.links().map(|v| v.filter(|id| is_external_link_storeid(id)).collect())
    }

    fn add_link(&mut self, store: &Store, l: Link) -> Result<Vec<StoreId>> {
        use crate::link::IntoUrl;
        l.into_url().and_then(|url| self.add_url(store, url))
    }

    fn get_links_matching<'a>(&self, store: &'a Store, r: Regex) -> Result<LinksMatchingRegexIter<'a>> {
        use self::iter::IntoLinksMatchingRegexIter;
        self.get_urls(store).map(|iter| iter.matching_regex(r))
    }

    fn remove_link(&mut self, store: &Store, l: Link) -> Result<Vec<StoreId>> {
        use crate::link::IntoUrl;
        l.into_url().and_then(|url| self.remove_url(store, url))
    }

}

pub mod iter {
    use crate::link::Link;
    use failure::Fallible as Result;
    use regex::Regex;

    use libimagentryurl::iter::UrlIter;

    pub struct LinkIter<I>(I)
        where I: Iterator<Item = Link>;

    impl<I: Iterator<Item = Link>> LinkIter<I> {
        pub fn new(i: I) -> LinkIter<I> {
            LinkIter(i)
        }
    }

    impl<I: Iterator<Item = Link>> Iterator for LinkIter<I> {
        type Item = Link;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.next()
        }
    }

    impl<I> From<I> for LinkIter<I> where I: Iterator<Item = Link> {
        fn from(i: I) -> LinkIter<I> {
            LinkIter(i)
        }
    }

    pub struct LinksMatchingRegexIter<'a>(UrlIter<'a>, Regex);

    impl<'a> LinksMatchingRegexIter<'a> {
        pub fn new(i: UrlIter<'a>, r: Regex) -> LinksMatchingRegexIter<'a> {
            LinksMatchingRegexIter(i, r)
        }
    }

    impl<'a> Iterator for LinksMatchingRegexIter<'a> {
        type Item = Result<Link>;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                let n = match self.0.next() {
                    Some(Ok(n))  => n,
                    Some(Err(e)) => return Some(Err(e)),
                    None         => return None,
                };

                let s = n.into_string();
                if self.1.is_match(&s[..]) {
                    return Some(Ok(Link::from(s)))
                } else {
                    continue;
                }
            }
        }
    }

    pub trait IntoLinksMatchingRegexIter<'a> {
        fn matching_regex(self, _: Regex) -> LinksMatchingRegexIter<'a>;
    }

    impl<'a> IntoLinksMatchingRegexIter<'a> for UrlIter<'a> {
        fn matching_regex(self, r: Regex) -> LinksMatchingRegexIter<'a> {
            LinksMatchingRegexIter(self, r)
        }
    }

}

