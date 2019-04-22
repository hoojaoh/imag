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

use std::path::Path;
use std::path::PathBuf;
use std::collections::BTreeMap;
use std::ops::Deref;

use libimagentryutil::isa::Is;
use libimagentryutil::isa::IsKindHeaderPathProvider;
use libimagerror::errors::ErrorMsg as EM;

use toml::Value;
use toml_query::read::TomlValueReadExt;
use toml_query::read::Partial;
use toml_query::delete::TomlValueDeleteExt;
use toml_query::insert::TomlValueInsertExt;
use failure::Fallible as Result;
use failure::Error;
use failure::err_msg;
use failure::ResultExt;

use hasher::Hasher;

/// A configuration of "basepath name" -> "basepath path" mappings
///
/// Should be deserializeable from the configuration file right away, because we expect a
/// configuration like this in the config file:
///
/// ```toml
/// [ref.basepathes]
/// music = "/home/alice/music"
/// documents = "/home/alice/doc"
/// ```
///
/// for example.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config(BTreeMap<String, PathBuf>);

impl Config {
    pub fn new(map: BTreeMap<String, PathBuf>) -> Self {
        Config(map)
    }
}

impl Deref for Config {
    type Target = BTreeMap<String, PathBuf>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Partial<'a> for Config {
    const LOCATION: &'static str = "ref.basepathes";
    type Output = Self;
}

provide_kindflag_path!(pub IsRef, "ref.is_ref");

/// Fassade module
///
/// This module is necessary to build a generic fassade around the "entry with a (default) hasher to
/// represent the entry as a ref".
///
/// The module is for code-structuring only, all types in the module are exported publicly in the
/// supermodule.
pub mod fassade {
    use std::marker::PhantomData;

    use libimagstore::store::Entry;
    use libimagentryutil::isa::Is;

    use failure::Fallible as Result;
    use failure::Error;

    use hasher::sha1::Sha1Hasher;
    use hasher::Hasher;
    use super::IsRef;

    pub trait RefFassade {
        fn is_ref(&self)                                -> Result<bool>;
        fn as_ref_with_hasher<H: Hasher>(&self)         -> RefWithHasher<H>;
        fn as_ref_with_hasher_mut<H: Hasher>(&mut self) -> MutRefWithHasher<H>;
    }

    impl RefFassade for Entry {
        /// Check whether the underlying object is actually a ref
        fn is_ref(&self) -> Result<bool> {
            self.is::<IsRef>().map_err(Error::from)
        }

        fn as_ref_with_hasher<H: Hasher>(&self)     -> RefWithHasher<H> {
            RefWithHasher::new(self)
        }

        fn as_ref_with_hasher_mut<H: Hasher>(&mut self) -> MutRefWithHasher<H> {
            MutRefWithHasher::new(self)
        }

    }

    pub struct RefWithHasher<'a, H: Hasher = Sha1Hasher>(pub(crate) &'a Entry, PhantomData<H>);

    impl<'a, H> RefWithHasher<'a, H>
        where H: Hasher
    {
        fn new(entry: &'a Entry) -> Self {
            RefWithHasher(entry, PhantomData)
        }
    }

    pub struct MutRefWithHasher<'a, H: Hasher = Sha1Hasher>(pub(crate) &'a mut Entry, PhantomData<H>);

    impl<'a, H> MutRefWithHasher<'a, H>
        where H: Hasher
    {
        fn new(entry: &'a mut Entry) -> Self {
            MutRefWithHasher(entry, PhantomData)
        }
    }
}
pub use self::fassade::*;


pub trait Ref {

    /// Check whether the underlying object is actually a ref
    fn is_ref(&self) -> Result<bool>;

    fn get_path(&self, config: &Config) -> Result<PathBuf>;

    fn get_relative_path(&self) -> Result<PathBuf>;

    /// Get the stored hash.
    fn get_hash(&self) -> Result<&str>;

    /// Check whether the referenced file still matches its hash
    fn hash_valid(&self, config: &Config) -> Result<bool>;
}

impl<'a, H: Hasher> Ref for RefWithHasher<'a, H> {

    /// Check whether the underlying object is actually a ref
    fn is_ref(&self) -> Result<bool> {
        self.0.is::<IsRef>().map_err(Error::from)
    }

    fn get_hash(&self) -> Result<&str> {
        let header_path = format!("ref.hash.{}", H::NAME);
        self.0
            .get_header()
            .read(&header_path)
            .map_err(Error::from)?
            .ok_or_else(|| {
                Error::from(EM::EntryHeaderFieldMissing("ref.hash.<hash>"))
            })
            .and_then(|v| {
                v.as_str().ok_or_else(|| {
                    Error::from(EM::EntryHeaderTypeError2("ref.hash.<hash>", "string"))
                })
            })
    }

    /// Get the path of the actual file
    fn get_path(&self, config: &Config) -> Result<PathBuf> {
        use toml_query::read::TomlValueReadTypeExt;

        let relpath = self.0
            .get_header()
            .read_string("ref.relpath")?
            .map(PathBuf::from)
            .ok_or_else(|| Error::from(EM::EntryHeaderFieldMissing("ref.relpath")))?;

        let basepath_name = self.0
            .get_header()
            .read_string("ref.basepath")?
            .ok_or_else(|| Error::from(EM::EntryHeaderFieldMissing("ref.basepath")))?;

        get_file_path(config, &basepath_name, relpath)
    }

    /// Get the relative path, relative to the configured basepath
    fn get_relative_path(&self) -> Result<PathBuf> {
        self.0
            .get_header()
            .read("ref.relpath")
            .map_err(Error::from)?
            .ok_or_else(|| Error::from(EM::EntryHeaderFieldMissing("ref.relpath")))
            .and_then(|v| {
                v.as_str()
                    .ok_or_else(|| EM::EntryHeaderTypeError2("ref.relpath", "string"))
                    .map_err(Error::from)
            })
            .map(PathBuf::from)
    }

    fn hash_valid(&self, config: &Config) -> Result<bool> {
        let ref_header = self.0
            .get_header()
            .read("ref")?
            .ok_or_else(|| err_msg("Header missing at 'ref'"))?;

        let basepath_name = ref_header
            .read("basepath")
            .map_err(Error::from)?
            .ok_or_else(|| err_msg("Header missing at 'ref.basepath'"))?
            .as_str()
            .ok_or_else(|| Error::from(EM::EntryHeaderTypeError2("ref.hash.<hash>", "string")))?;

        let path = ref_header
            .read("relpath")
            .map_err(Error::from)?
            .ok_or_else(|| err_msg("Header missing at 'ref.relpath'"))?
            .as_str()
            .map(PathBuf::from)
            .ok_or_else(|| Error::from(EM::EntryHeaderTypeError2("ref.hash.<hash>", "string")))?;


        let file_path = get_file_path(config, basepath_name.as_ref(), &path)?;

        ref_header
            .read(H::NAME)
            .map_err(Error::from)?
            .ok_or_else(|| format_err!("Header missing at 'ref.{}'", H::NAME))
            .and_then(|v| {
                v.as_str().ok_or_else(|| {
                    Error::from(EM::EntryHeaderTypeError2("ref.hash.<hash>", "string"))
                })
            })
            .and_then(|hash| H::hash(file_path).map(|h| h == hash))
    }

}

pub trait MutRef {
    fn remove_ref(&mut self) -> Result<()>;

    /// Make a ref out of a normal (non-ref) entry.
    ///
    /// If the entry is already a ref, this fails if `force` is false
    fn make_ref<P, Coll>(&mut self, path: P, basepath_name: Coll, config: &Config, force: bool)
        -> Result<()>
        where P: AsRef<Path>,
              Coll: AsRef<str>;
}


impl<'a, H> MutRef for MutRefWithHasher<'a, H>
    where H: Hasher
{

    fn remove_ref(&mut self) -> Result<()> {
        debug!("Removing 'ref' header section");
        {
            let header = self.0.get_header_mut();
            trace!("header = {:?}", header);

            let _ = header.delete("ref.relpath").context("Removing ref.relpath")?;

            if let Some(hash_tbl) = header.read_mut("ref.hash")? {
                match hash_tbl {
                    Value::Table(ref mut tbl) => *tbl = BTreeMap::new(),
                    _ => {
                        // should not happen
                    }
                }
            }

            let _ = header.delete("ref.hash").context("Removing ref.hash")?;
            let _ = header.delete("ref.basepath").context("Removing ref.basepath")?;
        }

        debug!("Removing 'ref' header marker");
        self.0.remove_isflag::<IsRef>().context("Removing ref")?;

        let _ = self.0
            .get_header_mut()
            .delete("ref")
            .context("Removing ref")?;

        trace!("header = {:?}", self.0.get_header());
        Ok(())
    }

    /// Make a ref out of a normal (non-ref) entry.
    ///
    /// `path` is the path to refer to,
    ///
    /// # Warning
    ///
    /// If the entry is already a ref, this fails if `force` is false
    ///
    fn make_ref<P, Coll>(&mut self, path: P, basepath_name: Coll, config: &Config, force: bool)
        -> Result<()>
        where P: AsRef<Path>,
              Coll: AsRef<str>
    {
        trace!("Making ref out of {:?}", self.0);
        trace!("Making ref with basepath name {:?}", basepath_name.as_ref());
        trace!("Making ref with config {:?}", config);
        trace!("Making ref forced = {}", force);

        if self.0.get_header().read("ref.is_ref")?.is_some() && !force {
            debug!("Entry is already a Ref!");
            let _ = Err(err_msg("Entry is already a reference")).context("Making ref out of entry")?;
        }

        let file_path = get_file_path(config, basepath_name.as_ref(), &path)?;

        if !file_path.exists() {
            let msg = format_err!("File '{:?}' does not exist", file_path);
            let _   = Err(msg).context("Making ref out of entry")?;
        }

        debug!("Entry hashing = {}", file_path.display());
        let _ = H::hash(&file_path)
            .and_then(|hash| {
                trace!("hash = {}", hash);

                // stripping the prefix of "path"
                let prefix = get_basepath(basepath_name.as_ref(), config)?;

                trace!("Stripping = {}", prefix.display());
                let relpath = path.as_ref().strip_prefix(prefix)?;

                trace!("Using relpath = {} to make header section", relpath.display());
                make_header_section(hash, H::NAME, relpath, basepath_name)
            })
            .and_then(|h| self.0.get_header_mut().insert("ref", h).map_err(Error::from))
            .and_then(|_| self.0.set_isflag::<IsRef>())
            .context("Making ref out of entry")?;

        debug!("Setting is-ref flag");
        self.0
            .set_isflag::<IsRef>()
            .context("Setting ref-flag")
            .map_err(Error::from)
            .map(|_| ())
    }

}

/// Create a new header section for a "ref".
///
/// # Warning
///
/// The `relpath` _must_ be relative to the configured path for that basepath.
pub(crate) fn make_header_section<P, C, H>(hash: String, hashname: H, relpath: P, basepath: C)
    -> Result<Value>
    where P: AsRef<Path>,
          C: AsRef<str>,
          H: AsRef<str>,
{
    let mut header_section = Value::Table(BTreeMap::new());
    {
        let relpath = relpath
            .as_ref()
            .to_str()
            .map(String::from)
            .ok_or_else(|| {
                let msg = format_err!("UTF Error in '{:?}'", relpath.as_ref());
                Error::from(msg)
            })?;

        let _ = header_section.insert("relpath", Value::String(relpath))?;
    }

    {
        let mut hash_table = Value::Table(BTreeMap::new());
        let _ = hash_table.insert(hashname.as_ref(), Value::String(hash))?;
        let _ = header_section.insert("hash", hash_table)?;
    }

    let _ = header_section.insert("basepath", Value::String(String::from(basepath.as_ref())));

    Ok(header_section)
}

fn get_basepath<'a, Coll: AsRef<str>>(basepath_name: Coll, config: &'a Config) -> Result<&'a PathBuf> {
    config.get(basepath_name.as_ref())
        .ok_or_else(|| format_err!("basepath {} seems not to exist in config",
                                   basepath_name.as_ref()))
        .map_err(Error::from)
}

fn get_file_path<P>(config: &Config, basepath_name: &str, path: P) -> Result<PathBuf>
        where P: AsRef<Path>
{
    config
        .get(basepath_name)
        .map(PathBuf::clone)
        .ok_or_else(|| {
            format_err!("Configuration missing for basepath: '{}'", basepath_name)
        })
        .context("Making ref out of entry")
        .map_err(Error::from)
        .map(|p| {
            let filepath = p.join(&path);
            trace!("Found filepath: {:?}", filepath.display());
            filepath
        })
}


#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use libimagstore::store::Store;
    use libimagstore::store::Entry;

    use super::*;
    use hasher::Hasher;

    fn setup_logging() {
        let _ = ::env_logger::try_init();
    }

    pub fn get_store() -> Store {
        Store::new_inmemory(PathBuf::from("/"), &None).unwrap()
    }

    struct TestHasher;
    impl Hasher for TestHasher {
        const NAME: &'static str = "Testhasher";

        fn hash<P: AsRef<Path>>(path: P) -> Result<String> {
            path.as_ref()
                .to_str()
                .map(String::from)
                .ok_or_else(|| Error::from(err_msg("Failed to create test hash")))
        }
    }


    #[test]
    fn test_isref() {
        setup_logging();
        let store = get_store();
        let entry = store.retrieve(PathBuf::from("test_isref")).unwrap();

        assert!(!entry.is_ref().unwrap());
    }

    #[test]
    fn test_makeref() {
        setup_logging();
        let store           = get_store();
        let mut entry       = store.retrieve(PathBuf::from("test_makeref")).unwrap();
        let file            = PathBuf::from("/tmp"); // has to exist
        let basepath_name = "some_basepath";
        let config          = Config({
            let mut c = BTreeMap::new();
            c.insert(String::from("some_basepath"), PathBuf::from("/"));
            c
        });

        let r = entry.as_ref_with_hasher_mut::<TestHasher>().make_ref(file, basepath_name, &config, false);
        assert!(r.is_ok());
    }

    #[test]
    fn test_makeref_isref() {
        setup_logging();
        let store           = get_store();
        let mut entry       = store.retrieve(PathBuf::from("test_makeref_isref")).unwrap();
        let file            = PathBuf::from("/tmp"); // has to exists
        let basepath_name = "some_basepath";
        let config          = Config({
            let mut c = BTreeMap::new();
            c.insert(String::from("some_basepath"), PathBuf::from("/"));
            c
        });

        let res = entry.as_ref_with_hasher_mut::<TestHasher>().make_ref(file, basepath_name, &config, false);
        assert!(res.is_ok(), "Expected to be ok: {:?}", res);

        assert!(entry.as_ref_with_hasher::<TestHasher>().is_ref().unwrap());
    }

    #[test]
    fn test_makeref_is_ref_with_testhash() {
        setup_logging();
        let store           = get_store();
        let mut entry       = store.retrieve(PathBuf::from("test_makeref_is_ref_with_testhash")).unwrap();
        let file            = PathBuf::from("/tmp"); // has to exist
        let basepath_name = "some_basepath";
        let config          = Config({
            let mut c = BTreeMap::new();
            c.insert(String::from("some_basepath"), PathBuf::from("/"));
            c
        });

        assert!(entry.as_ref_with_hasher_mut::<TestHasher>().make_ref(file, basepath_name, &config, false).is_ok());

        let check_isstr = |entry: &Entry, location, shouldbe| {
            let var = entry.get_header().read(location);

            assert!(var.is_ok(), "{} is not Ok(_): {:?}", location, var);
            let var = var.unwrap();

            assert!(var.is_some(), "{} is not Some(_): {:?}", location, var);
            let var = var.unwrap().as_str();

            assert!(var.is_some(), "{} is not String: {:?}", location, var);
            assert_eq!(var.unwrap(), shouldbe, "{} is not == {}", location, shouldbe);
        };

        check_isstr(&entry, "ref.relpath", "tmp");
        check_isstr(&entry, "ref.hash.Testhasher", "/tmp"); // TestHasher hashes by returning the path itself
        check_isstr(&entry, "ref.basepath", "some_basepath");
    }

    #[test]
    fn test_makeref_remref() {
        setup_logging();
        let store           = get_store();
        let mut entry       = store.retrieve(PathBuf::from("test_makeref_remref")).unwrap();
        let file            = PathBuf::from("/"); // has to exist
        let basepath_name = "some_basepath";
        let config          = Config({
            let mut c = BTreeMap::new();
            c.insert(String::from("some_basepath"), PathBuf::from("/"));
            c
        });

        assert!(entry.as_ref_with_hasher_mut::<TestHasher>().make_ref(file, basepath_name, &config, false).is_ok());
        assert!(entry.as_ref_with_hasher::<TestHasher>().is_ref().unwrap());
        let res = entry.as_ref_with_hasher_mut::<TestHasher>().remove_ref();
        assert!(res.is_ok(), "Expected to be ok: {:?}", res);
        assert!(!entry.as_ref_with_hasher::<TestHasher>().is_ref().unwrap());
    }

}

