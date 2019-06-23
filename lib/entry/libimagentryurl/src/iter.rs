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

//! Iterator helpers for external linking stuff
//!
//! Contains also helpers to filter iterators for external/internal links
//!
//!
//! # Warning
//!
//! This module uses `internal::Link` as link type, so we operate on _store ids_ here.
//!
//! Not to confuse with `external::Link` which is a real `FileLockEntry` under the hood.
//!

use libimagentrylink::link::Link;
use libimagentrylink::iter::LinkIter;
use libimagstore::store::Store;
use libimagutil::debug_result::DebugResult;

use failure::Fallible as Result;
use url::Url;

/// Helper for building `OnlyUrlIter` and `NoUrlIter`
///
/// The boolean value defines, how to interpret the `is_external_link_storeid()` return value
/// (here as "pred"):
///
/// ```ignore
///     pred | bool | xor | take?
///     ---- | ---- | --- | ----
///        0 |    0 |   0 |   1
///        0 |    1 |   1 |   0
///        1 |    0 |   1 |   0
///        1 |    1 |   0 |   1
/// ```
///
/// If `bool` says "take if return value is false", we take the element if the `pred` returns
/// false... and so on.
///
/// As we can see, the operator between these two operants is `!(a ^ b)`.
pub struct UrlFilterIter(LinkIter, bool);

impl Iterator for UrlFilterIter {
    type Item = Link;

    fn next(&mut self) -> Option<Self::Item> {
        use crate::util::is_external_link_storeid;

        while let Some(elem) = self.0.next() {
            trace!("Check whether is external: {:?}", elem);
            if !(self.1 ^ is_external_link_storeid(&elem)) {
                trace!("Is external id: {:?}", elem);
                return Some(elem);
            }
        }
        None
    }
}

/// Helper trait to be implemented on `LinkIter` to select or deselect all external links
///
/// # See also
///
/// Also see `OnlyUrlIter` and `NoUrlIter` and the helper traits/functions
/// `OnlyInteralLinks`/`only_links()` and `OnlyUrlLinks`/`only_urls()`.
pub trait SelectUrl {
    fn select_urls(self, b: bool) -> UrlFilterIter;
}

impl SelectUrl for LinkIter {
    fn select_urls(self, b: bool) -> UrlFilterIter {
        UrlFilterIter(self, b)
    }
}


pub struct OnlyUrlIter(UrlFilterIter);

impl OnlyUrlIter {
    pub fn new(li: LinkIter) -> OnlyUrlIter {
        OnlyUrlIter(UrlFilterIter(li, true))
    }

    pub fn urls<'a>(self, store: &'a Store) -> UrlIter<'a> {
        UrlIter(self, store)
    }
}

impl Iterator for OnlyUrlIter {
    type Item = Link;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct NoUrlIter(UrlFilterIter);

impl NoUrlIter {
    pub fn new(li: LinkIter) -> NoUrlIter {
        NoUrlIter(UrlFilterIter(li, false))
    }
}

impl Iterator for NoUrlIter {
    type Item = Link;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub trait OnlyUrlLinks : Sized {
    fn only_urls(self) -> OnlyUrlIter ;

    fn no_links(self) -> OnlyUrlIter {
        self.only_urls()
    }
}

impl OnlyUrlLinks for LinkIter {
    fn only_urls(self) -> OnlyUrlIter {
        OnlyUrlIter::new(self)
    }
}

pub trait OnlyInternalLinks : Sized {
    fn only_links(self) -> NoUrlIter;

    fn no_urls(self) -> NoUrlIter {
        self.only_links()
    }
}

impl OnlyInternalLinks for LinkIter {
    fn only_links(self) -> NoUrlIter {
        NoUrlIter::new(self)
    }
}

pub struct UrlIter<'a>(OnlyUrlIter, &'a Store);

impl<'a> Iterator for UrlIter<'a> {
    type Item = Result<Url>;

    fn next(&mut self) -> Option<Self::Item> {
        use crate::link::Link;

        loop {
            let next = self.0
                .next()
                .map(|id| {
                    debug!("Retrieving entry for id: '{:?}'", id);
                    self.1
                        .retrieve(id.clone())
                        .map_dbg_err(|_| format!("Retrieving entry for id: '{:?}' failed", id))
                        .map_err(From::from)
                        .and_then(|f| {
                            debug!("Store::retrieve({:?}) succeeded", id);
                            debug!("getting uri link from file now");
                            f.get_url()
                                .map_dbg_str("Error happened while getting link URI from FLE")
                                .map_dbg_err(|e| format!("URL -> Err = {:?}", e))
                        })
                });

            match next {
                Some(Ok(Some(link))) => return Some(Ok(link)),
                Some(Ok(None))       => continue,
                Some(Err(e))         => return Some(Err(e)),
                None                 => return None
            }
        }
    }

}


