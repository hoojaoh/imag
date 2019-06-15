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

use libimagstore::store::Entry;
use libimagerror::errors::ErrorMsg as EM;

use toml_query::read::TomlValueReadTypeExt;

pub trait Link {

    fn get_link_uri_from_filelockentry(&self) -> Result<Option<Url>>;

    fn get_url(&self) -> Result<Option<Url>>;

}

impl Link for Entry {

    fn get_link_uri_from_filelockentry(&self) -> Result<Option<Url>> {
        self.get_header()
            .read_string("links.external.content.url")
            .context(format_err!("Error reading header 'links.external.content.url' from '{}'", self.get_location()))
            .context(EM::EntryHeaderReadError)
            .map_err(Error::from)
            .and_then(|opt| match opt {
                None        => Ok(None),
                Some(ref s) => {
                    debug!("Found url, parsing: {:?}", s);
                    Url::parse(&s[..])
                        .map_err(Error::from)
                        .context(format_err!("Failed to parse URL: '{}'", s))
                        .context(err_msg("Invalid URI"))
                        .map_err(Error::from)
                        .map(Some)
                },
            })
            .context("Failed to get link URI from entry")
            .map_err(Error::from)
    }

    fn get_url(&self) -> Result<Option<Url>> {
        match self.get_header().read_string("links.external.url")? {
            None        => Ok(None),
            Some(ref s) => Url::parse(&s[..])
                .context(format_err!("Failed to parse URL: '{}'", s))
                .map(Some)
                .map_err(Error::from)
                .context(EM::EntryHeaderReadError)
                .map_err(Error::from),
        }
    }

}

