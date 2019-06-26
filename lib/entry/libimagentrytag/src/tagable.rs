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
    const LOCATION: &'static str = "tags";
    type Output                  = Self;
}

impl Tagable for Entry {

    fn get_tags(&self) -> Result<Vec<Tag>> {
        self.get_header()
            .read_partial::<TagHeader>()?
            .map(|header| {
                let _ = header.values
                    .iter()
                    .map(is_tag_str)
                    .collect::<Result<_>>()?;

                Ok(header.values)
            })
            .unwrap_or_else(|| Ok(vec![]))
    }

    fn set_tags(&mut self, ts: &[Tag]) -> Result<()> {
        let _ = ts
            .iter()
            .map(is_tag_str)
            .collect::<Result<Vec<_>>>()?;

        let header = TagHeader {
            values: ts.iter().unique().cloned().collect(),
        };

        debug!("Setting tags = {:?}", header);
        self.get_header_mut()
            .insert_serialized("tags", header)
            .map(|_| ())
            .map_err(|_| Error::from(EM::EntryHeaderWriteError))
    }

    fn add_tag(&mut self, t: Tag) -> Result<()> {
        let _ = is_tag_str(&t)?;

        let mut tags = self.get_tags()?;
        debug!("Pushing tag = {:?} to list = {:?}", t, tags);
        tags.push(t);
        self.set_tags(&tags)
    }

    fn remove_tag(&mut self, t: Tag) -> Result<()> {
        let _ = is_tag_str(&t)?;

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

