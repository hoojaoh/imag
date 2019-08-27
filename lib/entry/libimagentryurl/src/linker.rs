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

use libimagstore::storeid::StoreId;
use libimagstore::store::Store;
use libimagstore::store::Entry;
use libimagutil::debug_result::DebugResult;
use libimagentrylink::linkable::Linkable;

use failure::Fallible as Result;
use url::Url;
use sha1::{Sha1, Digest};
use hex;

use crate::link::Link;
use crate::iter::UrlIter;

pub trait UrlLinker : Linkable {

    fn get_urls<'a>(&self, store: &'a Store) -> Result<UrlIter<'a>>;

    fn set_urls(&mut self, store: &Store, links: Vec<Url>) -> Result<Vec<StoreId>>;

    fn add_url(&mut self, store: &Store, link: Url) -> Result<Vec<StoreId>>;

    fn remove_url(&mut self, store: &Store, link: Url) -> Result<Vec<StoreId>>;

}

impl UrlLinker for Entry {

    /// Get URLs from the Entry
    ///
    ///
    /// # Notice
    ///
    /// (Also see documentation of `UrlLinker::set_urls()`)
    ///
    /// This fetches all Links (as in `libimagentrylink` for the Entry and filters them by entries
    /// which contain an URL.
    ///
    ///
    /// # Return Value
    ///
    /// Iterator over URLs
    ///
    fn get_urls<'a>(&self, store: &'a Store) -> Result<UrlIter<'a>> {
        use crate::iter::OnlyUrlLinks;

        // Iterate through all internal links and filter for FileLockEntries which live in
        // /url/<SHA> -> load these files and get the url from their headers,
        // put them into the return vector.
        self.links()
            .map(|iter| {
                debug!("Getting urls");
                iter.only_urls().urls(store)
            })
    }

    /// Set URLs for the Entry
    ///
    /// # Notice
    ///
    /// This does not actually add each URL in this entry, but retrieves (as in
    /// `Store::retrieve()`) one entry for each URL and links (as in `libimagentrylink`) this entry
    /// to the retrieved ones.
    ///
    ///
    /// # Return Value
    ///
    /// Returns the StoreIds which were newly created for the new urls, if there are more
    /// urls than before.
    /// If there are less urls than before, an empty vec![] is returned.
    ///
    fn set_urls(&mut self, store: &Store, links: Vec<Url>) -> Result<Vec<StoreId>> {
        debug!("Iterating {} links = {:?}", links.len(), links);
        links.into_iter().map(|link| {
            let hash = hex::encode(Sha1::digest(&link.as_str().as_bytes()));
            let file_id = crate::module_path::new_id(hash.clone())
                .map_dbg_err(|_| format!("Failed to build StoreId for this hash '{:?}'", hash))?;

            debug!("Link    = '{:?}'", link);
            debug!("Hash    = '{:?}'", hash);
            debug!("StoreId = '{:?}'", file_id);

            let link_already_exists = store.exists(file_id.clone())?;

            // retrieve the file from the store, which implicitely creates the entry if it does not
            // exist
            let mut file = store
                .retrieve(file_id.clone())
                .map_dbg_err(|_| {
                    format!("Failed to create or retrieve an file for this link '{:?}'", link)
                })?;

            debug!("Generating header content!");
            file.set_url(link)?;

            // then add an internal link to the new file or return an error if this fails
            self.add_link(&mut file)?;
            debug!("Added linking: {:?} <-> {:?}", self.get_location(), file.get_location());

            Ok((link_already_exists, file_id))
        })
        .filter_map(|res| match res {
            Ok((exists, entry)) => if exists { Some(Ok(entry)) } else { None },
            Err(e) => Some(Err(e))
        })
        .collect()
    }

    /// Add an URL to the entry
    ///
    ///
    /// # Notice
    ///
    /// (Also see documentation of `UrlLinker::set_urls()`)
    ///
    ///
    /// # Return Value
    ///
    /// (See UrlLinker::set_urls())
    ///
    fn add_url(&mut self, store: &Store, link: Url) -> Result<Vec<StoreId>> {
        let mut links = self.get_urls(store)?.collect::<Result<Vec<_>>>()?;
        links.push(link);
        self.set_urls(store, links)
    }

    /// Remove an URL from the entry
    ///
    ///
    /// # Notice
    ///
    /// (Also see documentation of `UrlLinker::set_urls()`)
    ///
    ///
    /// # Return Value
    ///
    /// (See UrlLinker::set_urls())
    ///
    fn remove_url(&mut self, store: &Store, link: Url) -> Result<Vec<StoreId>> {
        let mut links = self.get_urls(store)?.collect::<Result<Vec<_>>>()?;
        links.retain(|l| *l != link);
        self.set_urls(store, links)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use libimagstore::store::Store;

    fn setup_logging() {
        let _ = env_logger::try_init();
    }

    pub fn get_store() -> Store {
        Store::new_inmemory(PathBuf::from("/"), &None).unwrap()
    }

    #[test]
    fn test_adding_url() {
        use toml_query::read::TomlValueReadTypeExt;

        setup_logging();
        let store = get_store();
        let mut e = store.retrieve(PathBuf::from("base-test_simple")).unwrap();
        let url   = Url::parse("http://google.de").unwrap();

        assert!(e.add_url(&store, url.clone()).is_ok());

        debug!("{:?}", e);
        debug!("Header: {:?}", e.get_header());

        let link = e.links().unwrap().next();
        assert!(link.is_some());
        let link = link.unwrap();

        debug!("link[0] = {:?}", link);
        let id = link.get_store_id();

        let link_entry = store.get(id.clone()).unwrap().unwrap();

        debug!("Entry = {:?}", link_entry);
        debug!("Header = {:?}", link_entry.get_header());

        let link = match link_entry.get_header().read_string("url.uri") {
            Ok(Some(s)) => s,
            Ok(None) => {
                assert!(false);
                unreachable!()
            },
            Err(e) => {
                error!("{:?}", e);
                assert!(false);
                unreachable!()
            },
        };

        assert_eq!(link, "http://google.de/");
    }

    #[test]
    fn test_simple() {
        setup_logging();
        let store = get_store();
        let mut e = store.retrieve(PathBuf::from("base-test_simple")).unwrap();
        let url   = Url::parse("http://google.de").unwrap();

        assert!(e.add_url(&store, url.clone()).is_ok());

        debug!("{:?}", e);
        debug!("Header: {:?}", e.get_header());

        let urls = e.get_urls(&store);
        let urls = match urls {
            Err(e) => {
                debug!("Error: {:?}", e);
                assert!(false);
                unreachable!()
            },
            Ok(urls) => urls.collect::<Vec<_>>(),
        };

        debug!("urls = {:?}", urls);

        assert_eq!(1, urls.len());
        assert_eq!(url, e.get_urls(&store).unwrap().next().unwrap().unwrap());
    }

}
