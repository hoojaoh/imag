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

use std::path::PathBuf;
use std::path::Path;

use toml::Value;
use toml::to_string as toml_to_string;
use toml::from_str as toml_from_str;
use toml_query::insert::TomlValueInsertExt;
use vobject::vcard::Vcard;
use failure::Error;
use failure::Fallible as Result;
use failure::ResultExt;

use libimagstore::storeid::StoreId;
use libimagstore::iter::Entries;
use libimagstore::store::Store;
use libimagstore::store::FileLockEntry;
use libimagentryutil::isa::Is;
use libimagentryref::reference::{MutRef, Config as RefConfig};

use crate::contact::IsContact;
use crate::deser::DeserVcard;
use crate::util;

pub trait ContactStore<'a> {

    fn create_from_path<CN>(&'a self, p: &PathBuf, rc: &RefConfig, collection_name: CN)
        -> Result<FileLockEntry<'a>>
        where CN: AsRef<str>;

    fn retrieve_from_path<CN>(&'a self,
                              p: &PathBuf,
                              rc: &RefConfig,
                              collection_name: CN,
                              force: bool)
        -> Result<FileLockEntry<'a>>
        where CN: AsRef<str>;

    fn create_from_buf<CN, P>(&'a self, buf: &str, path: P, rc: &RefConfig, collection_name: CN)
        -> Result<FileLockEntry<'a>>
        where CN: AsRef<str>,
              P: AsRef<Path>;

    fn retrieve_from_buf<CN, P>(&'a self,
                                buf: &str,
                                path: P,
                                rc: &RefConfig,
                                collection_name: CN,
                                force: bool)
        -> Result<FileLockEntry<'a>>
        where CN: AsRef<str>,
              P: AsRef<Path>;

    // getting

    fn all_contacts(&'a self) -> Result<Entries<'a>>;
}

/// The extension for the Store to work with contacts
impl<'a> ContactStore<'a> for Store {

    /// Create a contact from a filepath
    ///
    /// Uses the collection with `collection_name` from RefConfig to store the reference to the
    /// file.
    fn create_from_path<CN>(&'a self, p: &PathBuf, rc: &RefConfig, collection_name: CN)
        -> Result<FileLockEntry<'a>>
        where CN: AsRef<str>
    {
        util::read_to_string(p).and_then(|buf| self.create_from_buf(&buf, p, rc, collection_name))
    }

    /// Retrieve a contact from a filepath
    ///
    /// Uses the collection with `collection_name` from RefConfig to store the reference to the
    /// file.
    fn retrieve_from_path<CN>(&'a self, p: &PathBuf, rc: &RefConfig, collection_name: CN, force: bool)
        -> Result<FileLockEntry<'a>>
        where CN: AsRef<str>
    {
        util::read_to_string(p)
            .and_then(|buf| self.retrieve_from_buf(&buf, p, rc, collection_name, force))
    }

    /// Create a contact from a buffer
    ///
    /// Uses the collection with `collection_name` from RefConfig to store the reference to the
    /// file.
    ///
    /// Needs the `path` passed where the buffer was read from, because we want to create a
    /// reference to it.
    ///
    /// # Note
    ///
    /// This does _never_ force-override existing reference data, thus the `force` parameter of
    /// `postprocess_fetched_entry()` is hardcoded to `false`.
    ///
    fn create_from_buf<CN, P>(&'a self, buf: &str, path: P, rc: &RefConfig, collection_name: CN)
        -> Result<FileLockEntry<'a>>
        where CN: AsRef<str>,
              P: AsRef<Path>
    {
        let (sid, value) = prepare_fetching_from_store(buf)?;
        postprocess_fetched_entry(self.create(sid)?, value, path, rc, collection_name, false)
    }

    /// Retrieve a contact from a buffer
    ///
    /// Uses the collection with `collection_name` from RefConfig to store the reference to the
    /// file.
    ///
    /// Needs the `path` passed where the buffer was read from, because we want to create a
    /// reference to it.
    fn retrieve_from_buf<CN, P>(&'a self,
                                buf: &str,
                                path: P,
                                rc: &RefConfig,
                                collection_name: CN,
                                force: bool)
        -> Result<FileLockEntry<'a>>
        where CN: AsRef<str>,
              P: AsRef<Path>
    {
        let (sid, value) = prepare_fetching_from_store(buf)?;
        postprocess_fetched_entry(self.retrieve(sid)?, value, path, rc, collection_name, force)
    }

    fn all_contacts(&'a self) -> Result<Entries<'a>> {
        self.entries()?.in_collection("contact")
    }

}

/// Prepare the fetching from the store.
///
/// That means calculating the StoreId and the Value from the vcard data
fn prepare_fetching_from_store(buf: &str) -> Result<(StoreId, Value)> {
    let vcard = Vcard::build(&buf).context("Cannot parse Vcard").map_err(Error::from)?;
    debug!("Parsed: {:?}", vcard);

    let uid = vcard.uid()
        .ok_or_else(|| Error::from(format_err!("UID Missing: {}", buf.to_string())))?;

    let value = { // dirty ugly hack
        let serialized = DeserVcard::from(vcard);
        let serialized = toml_to_string(&serialized)?;
        toml_from_str::<Value>(&serialized)?
    };

    let sid = crate::module_path::new_id(uid.raw())?;

    Ok((sid, value))
}

/// Postprocess the entry just fetched (created or retrieved) from the store
///
/// We need the path, the refconfig and the collection name passed here because we create a
/// reference here.
///
/// This is marked as inline because what it does is trivial, but repetitve in this module.
#[inline]
fn postprocess_fetched_entry<'a, CN, P>(mut entry: FileLockEntry<'a>,
                                        value: Value,
                                        path: P,
                                        rc: &RefConfig,
                                        collection_name: CN,
                                        force: bool)
    -> Result<FileLockEntry<'a>>
    where CN: AsRef<str>,
           P: AsRef<Path>
{
    use libimagentryref::reference::RefFassade;
    use libimagentryref::hasher::sha1::Sha1Hasher;

    entry.set_isflag::<IsContact>()?;
    entry.get_header_mut().insert("contact.data", value)?;
    entry.as_ref_with_hasher_mut::<Sha1Hasher>().make_ref(path, collection_name, rc, force)?;

    Ok(entry)
}

