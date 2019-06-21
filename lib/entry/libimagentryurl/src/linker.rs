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

use std::ops::DerefMut;

use libimagstore::storeid::StoreId;
use libimagstore::store::Store;
use libimagstore::store::Entry;
use libimagutil::debug_result::DebugResult;
use libimagentrylink::linkable::Linkable;

use failure::Fallible as Result;
use toml::Value;
use toml::map::Map;
use toml_query::read::TomlValueReadExt;
use toml_query::insert::TomlValueInsertExt;
use url::Url;
use sha1::{Sha1, Digest};
use hex;

use crate::iter::UrlIter;

pub trait UrlLinker : Linkable {

    /// Get the external links from the implementor object
    fn get_urls<'a>(&self, store: &'a Store) -> Result<UrlIter<'a>>;

    /// Set the external links for the implementor object
    fn set_urls(&mut self, store: &Store, links: Vec<Url>) -> Result<Vec<StoreId>>;

    /// Add an external link to the implementor object
    fn add_url(&mut self, store: &Store, link: Url) -> Result<Vec<StoreId>>;

    /// Remove an external link from the implementor object
    fn remove_url(&mut self, store: &Store, link: Url) -> Result<Vec<StoreId>>;

}

/// Implement `ExternalLinker` for `Entry`, hiding the fact that there is no such thing as an external
/// link in an entry, but internal links to other entries which serve as external links, as one
/// entry in the store can only have one external link.
impl UrlLinker for Entry {

    /// Get the external links from the implementor object
    fn get_urls<'a>(&self, store: &'a Store) -> Result<UrlIter<'a>> {
        use crate::iter::OnlyExternalLinks;

        // Iterate through all internal links and filter for FileLockEntries which live in
        // /link/external/<SHA> -> load these files and get the external link from their headers,
        // put them into the return vector.
        self.links()
            .map(|iter| {
                debug!("Getting external links");
                iter.only_urls().urls(store)
            })
    }

    /// Set the external links for the implementor object
    ///
    /// # Return Value
    ///
    /// Returns the StoreIds which were newly created for the new external links, if there are more
    /// external links than before.
    /// If there are less external links than before, an empty vec![] is returned.
    ///
    fn set_urls(&mut self, store: &Store, links: Vec<Url>) -> Result<Vec<StoreId>> {
        // Take all the links, generate a SHA sum out of each one, filter out the already existing
        // store entries and store the other URIs in the header of one FileLockEntry each, in
        // the path /link/external/<SHA of the URL>

        debug!("Iterating {} links = {:?}", links.len(), links);
        links.into_iter().map(|link| {
            let hash = hex::encode(Sha1::digest(&link.as_str().as_bytes()));
            let file_id = crate::module_path::new_id(format!("external/{}", hash))
                .map_dbg_err(|_| {
                    format!("Failed to build StoreId for this hash '{:?}'", hash)
                })?;

            debug!("Link    = '{:?}'", link);
            debug!("Hash    = '{:?}'", hash);
            debug!("StoreId = '{:?}'", file_id);

            let link_already_exists = store.get(file_id.clone())?.is_some();

            // retrieve the file from the store, which implicitely creates the entry if it does not
            // exist
            let mut file = store
                .retrieve(file_id.clone())
                .map_dbg_err(|_| {
                    format!("Failed to create or retrieve an file for this link '{:?}'", link)
                })?;

            debug!("Generating header content!");
            {
                let hdr = file.deref_mut().get_header_mut();

                let mut table = match hdr.read("links.external.content")? {
                    Some(&Value::Table(ref table)) => table.clone(),
                    Some(_) => {
                        warn!("There is a value at 'links.external.content' which is not a table.");
                        warn!("Going to override this value");
                        Map::new()
                    },
                    None => Map::new(),
                };

                let v = Value::String(link.into_string());

                debug!("setting URL = '{:?}", v);
                table.insert(String::from("url"), v);

                let _ = hdr.insert("links.external.content", Value::Table(table))?;
                debug!("Setting URL worked");
            }

            // then add an internal link to the new file or return an error if this fails
            let _ = self.add_link(file.deref_mut())?;
            debug!("Added internal link");

            Ok((link_already_exists, file_id))
        })
        .filter_map(|res| match res {
            Ok((exists, entry)) => if exists { Some(Ok(entry)) } else { None },
            Err(e) => Some(Err(e))
        })
        .collect()
    }

    /// Add an external link to the implementor object
    ///
    /// # Return Value
    ///
    /// (See ExternalLinker::set_urls())
    ///
    /// Returns the StoreIds which were newly created for the new external links, if there are more
    /// external links than before.
    /// If there are less external links than before, an empty vec![] is returned.
    ///
    fn add_url(&mut self, store: &Store, link: Url) -> Result<Vec<StoreId>> {
        // get external links, add this one, save them
        debug!("Getting links");
        self.get_urls(store)
            .and_then(|links| {
                let mut links = links.collect::<Result<Vec<_>>>()?;

                debug!("Adding link = '{:?}' to links = {:?}", link, links);
                links.push(link);

                debug!("Setting {} links = {:?}", links.len(), links);
                self.set_urls(store, links)
            })
    }

    /// Remove an external link from the implementor object
    ///
    /// # Return Value
    ///
    /// (See ExternalLinker::set_urls())
    ///
    /// Returns the StoreIds which were newly created for the new external links, if there are more
    /// external links than before.
    /// If there are less external links than before, an empty vec![] is returned.
    ///
    fn remove_url(&mut self, store: &Store, link: Url) -> Result<Vec<StoreId>> {
        // get external links, remove this one, save them
        self.get_urls(store)
            .and_then(|links| {
                debug!("Removing link = '{:?}'", link);
                let links = links
                    .filter_map(Result::ok)
                    .filter(|l| l.as_str() != link.as_str())
                    .collect::<Vec<_>>();
                self.set_urls(store, links)
            })
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
    fn test_simple() {
        setup_logging();
        let store = get_store();
        let mut e = store.retrieve(PathBuf::from("base-test_simple")).unwrap();
        let url   = Url::parse("http://google.de").unwrap();

        assert!(e.add_url(&store, url.clone()).is_ok());

        assert_eq!(1, e.get_urls(&store).unwrap().count());
        assert_eq!(url, e.get_urls(&store).unwrap().next().unwrap().unwrap());
    }

}
