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
use toml_query::insert::TomlValueInsertExt;

use failure::Error;
use failure::ResultExt;
use failure::Fallible as Result;
use failure::err_msg;
use crate::tag::{Tag, TagSlice};
use crate::tag::is_tag_str;

use toml::Value;

pub trait Tagable {

    fn get_tags(&self) -> Result<Vec<Tag>>;
    fn set_tags(&mut self, ts: &[Tag]) -> Result<()>;

    fn add_tag(&mut self, t: Tag) -> Result<()>;
    fn remove_tag(&mut self, t: Tag) -> Result<()>;

    fn has_tag(&self, t: TagSlice) -> Result<bool>;
    fn has_tags(&self, ts: &[Tag]) -> Result<bool>;

}

impl Tagable for Value {

    fn get_tags(&self) -> Result<Vec<Tag>> {
        self.read("tag.values")
            .context(format_err!("Failed to read header at 'tag.values'"))
            .map_err(Error::from)
            .context(EM::EntryHeaderReadError)?
            .map(|val| {
                debug!("Got Value of tags...");
                val.as_array()
                    .map(|tags| {
                        debug!("Got Array<T> of tags...");
                        if !tags.iter().all(|t| is_match!(*t, Value::String(_))) {
                            return Err(format_err!("Tag type error: Got Array<T> where T is not a String: {:?}", tags));
                        }
                        debug!("Got Array<String> of tags...");
                        if tags.iter().any(|t| match *t {
                            Value::String(ref s) => !is_tag_str(s).is_ok(),
                            _ => unreachable!()})
                        {
                            return Err(format_err!("At least one tag is not a valid tag string"));
                        }

                        Ok(tags.iter()
                            .cloned()
                            .map(|t| {
                                match t {
                                   Value::String(s) => s,
                                   _ => unreachable!(),
                                }
                            })
                            .collect())
                    })
                    .unwrap_or(Ok(vec![]))
            })
            .unwrap_or(Ok(vec![]))
    }

    fn set_tags(&mut self, ts: &[Tag]) -> Result<()> {
        if ts.iter().any(|tag| !is_tag_str(tag).is_ok()) {
            let not_tag = ts.iter().filter(|t| !is_tag_str(t).is_ok()).next().unwrap();
            return Err(format_err!("Not a tag: '{}'", not_tag));
        }

        let a = ts.iter().unique().map(|t| Value::String(t.clone())).collect();
        debug!("Setting tags = {:?}", a);
        self.insert("tag.values", Value::Array(a))
            .map(|_| ())
            .map_err(|_| Error::from(EM::EntryHeaderWriteError))
    }

    fn add_tag(&mut self, t: Tag) -> Result<()> {
        if !is_tag_str(&t).map(|_| true)
            .map_err(|s| format_err!("{}", s))
            .context(err_msg("Not a tag"))?
        {
            return Err(format_err!("Not a tag: '{}'", t));
        }

        self.get_tags()
            .map(|mut tags| {
                debug!("Pushing tag = {:?} to list = {:?}", t, tags);
                tags.push(t);
                self.set_tags(&tags.into_iter().unique().collect::<Vec<_>>()[..])
            })
            .map(|_| ())
    }

    fn remove_tag(&mut self, t: Tag) -> Result<()> {
        if !is_tag_str(&t).map(|_| true)
            .map_err(|s| format_err!("{}", s))
            .context(err_msg("Not a tag"))?
        {
            debug!("Not a tag: '{}'", t);
            return Err(format_err!("Not a tag: '{}'", t));
        }

        self.get_tags()
            .map(|mut tags| {
                tags.retain(|tag| tag.clone() != t);
                self.set_tags(&tags[..])
            })
            .map(|_| ())
    }

    fn has_tag(&self, t: TagSlice) -> Result<bool> {
        let tags = self.read("tag.values").context(EM::EntryHeaderReadError)?;

        if !tags.iter().all(|t| is_match!(*t, &Value::String(_))) {
            return Err(err_msg("Tag type error"))
        }

        Ok(tags
           .iter()
           .any(|tag| {
               match *tag {
                   &Value::String(ref s) => { s == t },
                   _ => unreachable!()
               }
           }))
    }

    fn has_tags(&self, tags: &[Tag]) -> Result<bool> {
        let mut result = true;
        for tag in tags {
            result = result && self.has_tag(tag)?;
        }

        Ok(result)
    }

}

impl Tagable for Entry {

    fn get_tags(&self) -> Result<Vec<Tag>> {
        self.get_header().get_tags()
    }

    fn set_tags(&mut self, ts: &[Tag]) -> Result<()> {
        self.get_header_mut().set_tags(ts)
    }

    fn add_tag(&mut self, t: Tag) -> Result<()> {
        self.get_header_mut().add_tag(t)
    }

    fn remove_tag(&mut self, t: Tag) -> Result<()> {
        self.get_header_mut().remove_tag(t)
    }

    fn has_tag(&self, t: TagSlice) -> Result<bool> {
        self.get_header().has_tag(t)
    }

    fn has_tags(&self, ts: &[Tag]) -> Result<bool> {
        self.get_header().has_tags(ts)
    }

}

