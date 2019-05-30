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

/// Helper for building `OnlyExternalIter` and `NoExternalIter`
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
pub struct ExternalFilterIter(LinkIter, bool);

impl Iterator for ExternalFilterIter {
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
/// Also see `OnlyExternalIter` and `NoExternalIter` and the helper traits/functions
/// `OnlyInteralLinks`/`only_internal_links()` and `OnlyExternalLinks`/`only_urls()`.
pub trait SelectExternal {
    fn select_urls(self, b: bool) -> ExternalFilterIter;
}

impl SelectExternal for LinkIter {
    fn select_urls(self, b: bool) -> ExternalFilterIter {
        ExternalFilterIter(self, b)
    }
}


pub struct OnlyExternalIter(ExternalFilterIter);

impl OnlyExternalIter {
    pub fn new(li: LinkIter) -> OnlyExternalIter {
        OnlyExternalIter(ExternalFilterIter(li, true))
    }

    pub fn urls<'a>(self, store: &'a Store) -> UrlIter<'a> {
        UrlIter(self, store)
    }
}

impl Iterator for OnlyExternalIter {
    type Item = Link;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub struct NoExternalIter(ExternalFilterIter);

impl NoExternalIter {
    pub fn new(li: LinkIter) -> NoExternalIter {
        NoExternalIter(ExternalFilterIter(li, false))
    }
}

impl Iterator for NoExternalIter {
    type Item = Link;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub trait OnlyExternalLinks : Sized {
    fn only_urls(self) -> OnlyExternalIter ;

    fn no_internal_links(self) -> OnlyExternalIter {
        self.only_urls()
    }
}

impl OnlyExternalLinks for LinkIter {
    fn only_urls(self) -> OnlyExternalIter {
        OnlyExternalIter::new(self)
    }
}

pub trait OnlyInternalLinks : Sized {
    fn only_internal_links(self) -> NoExternalIter;

    fn no_urls(self) -> NoExternalIter {
        self.only_internal_links()
    }
}

impl OnlyInternalLinks for LinkIter {
    fn only_internal_links(self) -> NoExternalIter {
        NoExternalIter::new(self)
    }
}

pub struct UrlIter<'a>(OnlyExternalIter, &'a Store);

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
                            debug!("getting external link from file now");
                            f.get_link_uri_from_filelockentry()
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


