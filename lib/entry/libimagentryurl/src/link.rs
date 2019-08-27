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

use failure::Error;
use failure::ResultExt;
use failure::Fallible as Result;
use failure::err_msg;
use url::Url;
use toml_query::read::Partial;
use toml_query::insert::TomlValueInsertExt;
use toml::Value;

use libimagstore::store::Entry;
use libimagerror::errors::ErrorMsg as EM;

use toml_query::read::TomlValueReadExt;

pub trait Link {
    fn get_url(&self) -> Result<Option<Url>>;
    fn set_url(&mut self, url: Url) -> Result<()>;
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct UrlHeader {
    pub uri: Option<String>,
}

impl Default for UrlHeader {
    fn default() -> Self {
        UrlHeader {
            uri: None
        }
    }
}

impl<'a> Partial<'a> for UrlHeader {
    const LOCATION: &'static str = "url";
    type Output                  = Self;
}


impl Link for Entry {

    /// Get the URL from entry Entry
    ///
    /// # Notice
    ///
    /// This actually returns the header field of the entry, parsed as URL
    ///
    ///
    fn get_url(&self) -> Result<Option<Url>> {
        let partial = self.get_header()
            .read_partial::<UrlHeader>()
            .context(format_err!("Error reading header 'url.uri' from '{}'", self.get_location()))
            .context(EM::EntryHeaderReadError)
            .map_err(Error::from)?
            .unwrap_or_else(Default::default);

        debug!("Partial deserialized: {:?}", partial);

        let url = match partial.uri {
            Some(uri) => uri,
            None      => return Ok(None),
        };

        debug!("Found url, parsing: {:?}", url);
        Url::parse(&url)
            .map_err(Error::from)
            .context(format_err!("Failed to parse URL: '{}'", url))
            .context(err_msg("Invalid URI"))
            .map_err(Error::from)
            .map(Some)
            .context("Failed to get link URI from entry")
            .map_err(Error::from)
    }

    fn set_url(&mut self, url: Url) -> Result<()> {
        let val = Value::String(url.to_string());
        self.get_header_mut().insert_serialized("url.uri", val)?;

        debug!("Setting URL worked");
        Ok(())
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
    fn test_header_set_correctly() {
        setup_logging();
        let store = get_store();
        let mut e = store.retrieve(PathBuf::from("urlentry")).unwrap();
        let url   = Url::parse("http://google.de").unwrap();

        assert!(e.set_url(url).is_ok());
        debug!("Fetch header: {:?}", e.get_header());

        let url = e.get_header().read("url.uri");

        debug!("Fetched header: {:?}", url);

        assert!(url.is_ok());
        let url = url.unwrap();

        assert!(url.is_some());
        let url = url.unwrap();

        match url {
            Value::String(ref s) => assert_eq!("http://google.de/", s),
            _ => assert!(false),
        }
    }

}
