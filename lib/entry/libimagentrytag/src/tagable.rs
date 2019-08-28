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

use itertools::Itertools;

use libimagstore::store::Entry;
use libimagerror::errors::ErrorMsg as EM;

use toml_query::read::TomlValueReadExt;
use toml_query::read::Partial;
use toml_query::insert::TomlValueInsertExt;

use failure::Error;
use failure::Fallible as Result;
use crate::tag::{Tag, TagSlice};
use crate::tag::is_tag_str;

pub trait Tagable {

    fn get_tags(&self) -> Result<Vec<Tag>>;
    fn set_tags(&mut self, ts: &[Tag]) -> Result<()>;

    fn add_tag(&mut self, t: Tag) -> Result<()>;
    fn remove_tag(&mut self, t: Tag) -> Result<()>;

    fn has_tag(&self, t: TagSlice) -> Result<bool>;
    fn has_tags(&self, ts: &[Tag]) -> Result<bool>;

}

#[derive(Serialize, Deserialize, Debug)]
struct TagHeader {
    values: Vec<String>,
}

impl<'a> Partial<'a> for TagHeader {
    const LOCATION: &'static str = "tag";
    type Output                  = Self;
}

impl Tagable for Entry {

    fn get_tags(&self) -> Result<Vec<Tag>> {
        self.get_header()
            .read_partial::<TagHeader>()?
            .map(|header| {
                header.values
                    .iter()
                    .map(|val| is_tag_str(val))
                    .collect::<Result<_>>()?;

                Ok(header.values)
            })
            .unwrap_or_else(|| Ok(vec![]))
    }

    fn set_tags(&mut self, ts: &[Tag]) -> Result<()> {
        let _ = ts
            .iter()
            .map(|val| is_tag_str(val))
            .collect::<Result<Vec<_>>>()?;

        let header = TagHeader {
            values: ts.iter().unique().cloned().collect(),
        };

        debug!("Setting tags = {:?}", header);
        self.get_header_mut()
            .insert_serialized("tag", header)
            .map(|_| ())
            .map_err(|_| Error::from(EM::EntryHeaderWriteError))
    }

    fn add_tag(&mut self, t: Tag) -> Result<()> {
        is_tag_str(&t)?;

        let mut tags = self.get_tags()?;
        debug!("Pushing tag = {:?} to list = {:?}", t, tags);
        tags.push(t);
        self.set_tags(&tags)
    }

    fn remove_tag(&mut self, t: Tag) -> Result<()> {
        is_tag_str(&t)?;

        let mut tags = self.get_tags()?;
        tags.retain(|tag| *tag != t);
        self.set_tags(&tags)
    }

    fn has_tag(&self, t: TagSlice) -> Result<bool> {
        // use any() because Vec::contains() wants &String, but we do not want to allocate.
        self.get_tags().map(|v| v.iter().any(|s| s == t))
    }

    fn has_tags(&self, tags: &[Tag]) -> Result<bool> {
        tags.iter().map(|t| self.has_tag(t)).fold(Ok(true), |a, e| a.and_then(|b| Ok(b && e?)))
    }


}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use toml_query::read::TomlValueReadTypeExt;

    use libimagstore::store::Store;

    use super::*;

    fn setup_logging() {
        let _ = ::env_logger::try_init();
    }

    fn get_store() -> Store {
        Store::new_inmemory(PathBuf::from("/"), &None).unwrap()
    }

    #[test]
    fn test_tag_get_tag() {
        setup_logging();
        let store = get_store();
        let name = "test-tag-get-tags";

        debug!("Creating default entry");
        let id = PathBuf::from(String::from(name));
        let mut entry = store.create(id).unwrap();

        let tags = vec![String::from("a")];
        entry.set_tags(&tags).unwrap();

        let v = entry.get_tags();

        assert!(v.is_ok());
        let v = v.unwrap();

        assert_eq!(v, vec!["a"]);
    }

    #[test]
    fn test_tag_add_adds_tag() {
        setup_logging();
        let store = get_store();
        let name = "test-tag-set-sets-tags";

        debug!("Creating default entry");
        let id = PathBuf::from(String::from(name));
        let mut entry = store.create(id).unwrap();

        entry.add_tag(String::from("test")).unwrap();

        let v = entry.get_header().read_string("tag.values.[0]").unwrap();

        assert!(v.is_some());
        let v = v.unwrap();

        assert_eq!(v, "test");
    }

    #[test]
    fn test_tag_remove_removes_tag() {
        setup_logging();
        let store = get_store();
        let name = "test-tag-set-sets-tags";

        debug!("Creating default entry");
        let id = PathBuf::from(String::from(name));
        let mut entry = store.create(id).unwrap();

        entry.add_tag(String::from("test")).unwrap();

        let v = entry.get_header().read_string("tag.values.[0]").unwrap();
        assert!(v.is_some());

        entry.remove_tag(String::from("test")).unwrap();

        assert!(entry.get_header().read_string("tag.values.[0]").is_err());
        let tags = entry.get_tags();
        assert!(tags.is_ok());
        let tags = tags.unwrap();
        assert!(tags.is_empty());
    }

    #[test]
    fn test_tag_set_sets_tag() {
        setup_logging();
        let store = get_store();
        let name = "test-tag-set-sets-tags";

        debug!("Creating default entry");
        let id = PathBuf::from(String::from(name));
        let mut entry = store.create(id).unwrap();

        let tags = vec![String::from("testtag")];
        entry.set_tags(&tags).unwrap();

        let v = entry.get_header().read_string("tag.values.[0]").unwrap();

        assert!(v.is_some());
        let v = v.unwrap();

        assert_eq!(v, "testtag");
    }

}
