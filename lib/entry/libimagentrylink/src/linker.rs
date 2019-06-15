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
use libimagstore::store::Entry;
use libimagstore::store::Store;
use libimagerror::errors::ErrorMsg as EM;

use toml_query::read::TomlValueReadExt;
use toml_query::insert::TomlValueInsertExt;
use failure::ResultExt;
use failure::Fallible as Result;
use failure::Error;
use failure::err_msg;

use crate::iter::LinkIter;
use crate::iter::IntoValues;
use crate::link::Link;

use toml::Value;

pub trait InternalLinker {

    /// Get the internal links from the implementor object
    fn get_internal_links(&self) -> Result<LinkIter>;

    /// Add an internal link to the implementor object
    fn add_internal_link(&mut self, link: &mut Entry) -> Result<()>;

    /// Remove an internal link from the implementor object
    fn remove_internal_link(&mut self, link: &mut Entry) -> Result<()>;

    /// Remove _all_ internal links
    fn unlink(&mut self, store: &Store) -> Result<()>;

    /// Add internal annotated link
    fn add_internal_annotated_link(&mut self, link: &mut Entry, annotation: String) -> Result<()>;
}

impl InternalLinker for Entry {

    fn get_internal_links(&self) -> Result<LinkIter> {
        debug!("Getting internal links");
        trace!("Getting internal links from header of '{}' = {:?}", self.get_location(), self.get_header());
        let res = self
            .get_header()
            .read("links.internal")
            .context(format_err!("Failed to read header 'links.internal' of '{}'", self.get_location()))
            .context(EM::EntryHeaderReadError)
            .context(EM::EntryHeaderError)
            .map_err(Error::from)
            .map(|r| r.cloned());
        process_rw_result(res)
    }

    fn add_internal_link(&mut self, link: &mut Entry) -> Result<()> {
        debug!("Adding internal link: {:?}", link);
        let location = link.get_location().clone().into();
        add_internal_link_with_instance(self, link, location)
    }

    fn remove_internal_link(&mut self, link: &mut Entry) -> Result<()> {
        debug!("Removing internal link: {:?}", link);

        // Cloning because of borrowing
        let own_loc   = self.get_location().clone();
        let other_loc = link.get_location().clone();

        debug!("Removing internal link from {:?} to {:?}", own_loc, other_loc);

        let links = link.get_internal_links()?;
        debug!("Rewriting own links for {:?}, without {:?}", other_loc, own_loc);

        let links = links.filter(|l| !l.eq_store_id(&own_loc));
        let _     = rewrite_links(link.get_header_mut(), links)?;

        self.get_internal_links()
            .and_then(|links| {
                debug!("Rewriting own links for {:?}, without {:?}", own_loc, other_loc);
                let links = links.filter(|l| !l.eq_store_id(&other_loc));
                rewrite_links(self.get_header_mut(), links)
            })
    }

    fn unlink(&mut self, store: &Store) -> Result<()> {
        for id in self.get_internal_links()?.map(|l| l.get_store_id().clone()) {
            match store.get(id).context("Failed to get entry")? {
                Some(mut entry) => self.remove_internal_link(&mut entry)?,
                None            => return Err(err_msg("Link target does not exist")),
            }
        }

        Ok(())
    }

    fn add_internal_annotated_link(&mut self, link: &mut Entry, annotation: String) -> Result<()> {
        let new_link = Link::Annotated {
            link: link.get_location().clone(),
            annotation: annotation,
        };

        add_internal_link_with_instance(self, link, new_link)
    }

}

fn add_internal_link_with_instance(this: &mut Entry, link: &mut Entry, instance: Link) -> Result<()> {
    debug!("Adding internal link from {:?} to {:?}", this.get_location(), instance);

    add_foreign_link(link, this.get_location().clone())
        .and_then(|_| {
            this.get_internal_links()
                .and_then(|links| {
                    let links = links.chain(LinkIter::new(vec![instance]));
                    rewrite_links(this.get_header_mut(), links)
                })
        })
}

fn rewrite_links<I: Iterator<Item = Link>>(header: &mut Value, links: I) -> Result<()> {
    let links = links.into_values()
                     .into_iter()
                     .fold(Ok(vec![]), |acc: Result<Vec<_>>, elem| {
                        acc.and_then(move |mut v| {
                            v.push(elem.context(EM::ConversionError)?);
                            Ok(v)
                        })
                     })?;

    debug!("Setting new link array: {:?}", links);
    let process = header
        .insert("links.internal", Value::Array(links))
        .context(format_err!("Failed to insert header 'links.internal'"))
        .context(EM::EntryHeaderReadError)
        .map_err(Error::from);
    process_rw_result(process).map(|_| ())
}

/// When Linking A -> B, the specification wants us to link back B -> A.
/// This is a helper function which does this.
fn add_foreign_link(target: &mut Entry, from: StoreId) -> Result<()> {
    debug!("Linking back from {:?} to {:?}", target.get_location(), from);
    target.get_internal_links()
        .and_then(|links| {
            let links = links
                             .chain(LinkIter::new(vec![from.into()]))
                             .into_values()
                             .into_iter()
                             .fold(Ok(vec![]), |acc: Result<Vec<_>>, elem| {
                                acc.and_then(move |mut v| {
                                    v.push(elem.context(EM::ConversionError)?);
                                    Ok(v)
                                })
                             })?;
            debug!("Setting links in {:?}: {:?}", target.get_location(), links);

            let res = target
                .get_header_mut()
                .insert("links.internal", Value::Array(links))
                .context(format_err!("Failed to insert header 'links.internal'"))
                .context(EM::EntryHeaderReadError)
                .map_err(Error::from);

            process_rw_result(res).map(|_| ())
        })
}

fn process_rw_result(links: Result<Option<Value>>) -> Result<LinkIter> {
    use std::path::PathBuf;

    let links = match links {
        Err(e) => {
            debug!("RW action on store failed. Generating LinkError");
            return Err(e).context(EM::EntryHeaderReadError).map_err(Error::from)
        },
        Ok(None) => {
            debug!("We got no value from the header!");
            return Ok(LinkIter::new(vec![]))
        },
        Ok(Some(Value::Array(l))) => l,
        Ok(Some(_)) => {
            debug!("We expected an Array for the links, but there was a non-Array!");
            return Err(err_msg("Link type error"));
        }
    };

    if !links.iter().all(|l| is_match!(*l, Value::String(_)) || is_match!(*l, Value::Table(_))) {
        debug!("At least one of the Values which were expected in the Array of links is not a String or a Table!");
        debug!("Generating LinkError");
        return Err(err_msg("Existing Link type error"));
    }

    let links : Vec<Link> = links.into_iter()
        .map(|link| {
            debug!("Matching the link: {:?}", link);
            match link {
                Value::String(s) => StoreId::new(PathBuf::from(s))
                    .map(|s| Link::Id { link: s })
                    .map_err(From::from)
                    ,
                Value::Table(mut tab) => {
                    debug!("Destructuring table");
                    if !tab.contains_key("link")
                    || !tab.contains_key("annotation") {
                        debug!("Things missing... returning Error instance");
                        Err(err_msg("Link parser error"))
                    } else {
                        let link = tab.remove("link")
                            .ok_or(err_msg("Link parser: field missing"))?;

                        let anno = tab.remove("annotation")
                            .ok_or(err_msg("Link parser: Field missing"))?;

                        debug!("Ok, here we go with building a Link::Annotated");
                        match (link, anno) {
                            (Value::String(link), Value::String(anno)) => {
                                StoreId::new(PathBuf::from(link))
                                    .map_err(From::from)
                                    .map(|link| {
                                        Link::Annotated {
                                            link: link,
                                            annotation: anno,
                                        }
                                    })
                            },
                            _ => Err(err_msg("Link parser: Field type error")),
                        }
                    }
                }
                _ => unreachable!(),
            }
        })
        .collect::<Result<Vec<Link>>>()?;

    debug!("Ok, the RW action was successful, returning link vector now!");
    Ok(LinkIter::new(links))
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use libimagstore::store::Store;

    use super::InternalLinker;
    use super::Link;

    fn setup_logging() {
        let _ = ::env_logger::try_init();
    }

    pub fn get_store() -> Store {
        Store::new_inmemory(PathBuf::from("/"), &None).unwrap()
    }

    #[test]
    fn test_new_entry_no_links() {
        setup_logging();
        let store = get_store();
        let entry = store.create(PathBuf::from("test_new_entry_no_links")).unwrap();
        let links = entry.get_internal_links();
        assert!(links.is_ok());
        let links = links.unwrap();
        assert_eq!(links.collect::<Vec<_>>().len(), 0);
    }

    #[test]
    fn test_link_two_entries() {
        setup_logging();
        let store = get_store();
        let mut e1 = store.create(PathBuf::from("test_link_two_entries1")).unwrap();
        assert!(e1.get_internal_links().is_ok());

        let mut e2 = store.create(PathBuf::from("test_link_two_entries2")).unwrap();
        assert!(e2.get_internal_links().is_ok());

        {
            assert!(e1.add_internal_link(&mut e2).is_ok());

            let e1_links = e1.get_internal_links().unwrap().collect::<Vec<_>>();
            let e2_links = e2.get_internal_links().unwrap().collect::<Vec<_>>();

            debug!("1 has links: {:?}", e1_links);
            debug!("2 has links: {:?}", e2_links);

            assert_eq!(e1_links.len(), 1);
            assert_eq!(e2_links.len(), 1);

            assert!(e1_links.first().map(|l| l.clone().eq_store_id(e2.get_location())).unwrap_or(false));
            assert!(e2_links.first().map(|l| l.clone().eq_store_id(e1.get_location())).unwrap_or(false));
        }

        {
            assert!(e1.remove_internal_link(&mut e2).is_ok());

            debug!("{:?}", e2.to_str());
            let e2_links = e2.get_internal_links().unwrap().collect::<Vec<_>>();
            assert_eq!(e2_links.len(), 0, "Expected [], got: {:?}", e2_links);

            debug!("{:?}", e1.to_str());
            let e1_links = e1.get_internal_links().unwrap().collect::<Vec<_>>();
            assert_eq!(e1_links.len(), 0, "Expected [], got: {:?}", e1_links);

        }
    }

    #[test]
    fn test_multiple_links() {
        setup_logging();
        let store = get_store();

        let mut e1 = store.retrieve(PathBuf::from("1")).unwrap();
        let mut e2 = store.retrieve(PathBuf::from("2")).unwrap();
        let mut e3 = store.retrieve(PathBuf::from("3")).unwrap();
        let mut e4 = store.retrieve(PathBuf::from("4")).unwrap();
        let mut e5 = store.retrieve(PathBuf::from("5")).unwrap();

        assert!(e1.add_internal_link(&mut e2).is_ok());

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e4.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e5.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e1.add_internal_link(&mut e3).is_ok());

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 2);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e4.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e5.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e1.add_internal_link(&mut e4).is_ok());

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 3);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e4.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e5.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e1.add_internal_link(&mut e5).is_ok());

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 4);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e4.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e5.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);

        assert!(e5.remove_internal_link(&mut e1).is_ok());

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 3);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e4.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e5.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e4.remove_internal_link(&mut e1).is_ok());

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 2);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e4.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e5.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e3.remove_internal_link(&mut e1).is_ok());

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e4.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e5.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e2.remove_internal_link(&mut e1).is_ok());

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e4.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e5.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);

    }

    #[test]
    fn test_link_deleting() {
        setup_logging();
        let store = get_store();

        let mut e1 = store.retrieve(PathBuf::from("1")).unwrap();
        let mut e2 = store.retrieve(PathBuf::from("2")).unwrap();

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e1.add_internal_link(&mut e2).is_ok());

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);

        assert!(e1.remove_internal_link(&mut e2).is_ok());

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
    }

    #[test]
    fn test_link_deleting_multiple_links() {
        setup_logging();
        let store = get_store();

        let mut e1 = store.retrieve(PathBuf::from("1")).unwrap();
        let mut e2 = store.retrieve(PathBuf::from("2")).unwrap();
        let mut e3 = store.retrieve(PathBuf::from("3")).unwrap();

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e1.add_internal_link(&mut e2).is_ok()); // 1-2
        assert!(e1.add_internal_link(&mut e3).is_ok()); // 1-2, 1-3

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 2);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);

        assert!(e2.add_internal_link(&mut e3).is_ok()); // 1-2, 1-3, 2-3

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 2);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 2);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 2);

        assert!(e1.remove_internal_link(&mut e2).is_ok()); // 1-3, 2-3

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 2);

        assert!(e1.remove_internal_link(&mut e3).is_ok()); // 2-3

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 1);

        assert!(e2.remove_internal_link(&mut e3).is_ok());

        assert_eq!(e1.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e2.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e3.get_internal_links().unwrap().collect::<Vec<_>>().len(), 0);
    }

    #[test]
    fn test_link_annotating() {
        setup_logging();
        let store      = get_store();
        let mut entry1 = store.create(PathBuf::from("test_link_annotating-1")).unwrap();
        let mut entry2 = store.create(PathBuf::from("test_link_annotating-2")).unwrap();

        let res = entry1.add_internal_annotated_link(&mut entry2, String::from("annotation"));
        assert!(res.is_ok());

        {
            for link in entry1.get_internal_links().unwrap() {
                match link  {
                    Link::Annotated {annotation, ..} => assert_eq!(annotation, "annotation"),
                    _ => assert!(false, "Non-annotated link found"),
                }
            }
        }

        {
            for link in entry2.get_internal_links().unwrap() {
                match link  {
                    Link::Id {..}        => {},
                    Link::Annotated {..} => assert!(false, "Annotated link found"),
                }
            }
        }
    }

}
