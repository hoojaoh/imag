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

use libimagstore::storeid::StoreId;
use libimagstore::store::Entry;
use libimagstore::store::Store;

use toml_query::read::Partial;
use toml_query::read::TomlValueReadExt;
use toml_query::insert::TomlValueInsertExt;
use failure::ResultExt;
use failure::Fallible as Result;
use failure::err_msg;

use crate::iter::LinkIter;
use crate::link::Link;

pub trait Linkable {

    /// Get all links
    fn links(&self) -> Result<LinkIter>;

    /// Get all links which are unidirectional links
    fn unidirectional_links(&self) -> Result<LinkIter>;

    /// Get all links which are directional links, outgoing
    fn directional_links_to(&self) -> Result<LinkIter>;

    /// Get all links which are directional links, incoming
    fn directional_links_from(&self) -> Result<LinkIter>;

    /// Add an internal link to the implementor object
    fn add_link(&mut self, link: &mut Entry) -> Result<()>;

    /// Remove an internal link from the implementor object
    fn remove_link(&mut self, link: &mut Entry) -> Result<()>;

    /// Remove _all_ internal links
    fn unlink(&mut self, store: &Store) -> Result<()>;

    /// Add a directional link: self -> otehr
    fn add_link_to(&mut self, other: &mut Entry) -> Result<()>;

    /// Remove a directional link: self -> otehr
    fn remove_link_to(&mut self, other: &mut Entry) -> Result<()>;

    /// Check whether an entry is linked to another entry
    fn is_linked_to(&self, other: &Entry) -> Result<bool>;
}

#[derive(Serialize, Deserialize, Debug)]
struct LinkPartial {
    internal: Option<Vec<String>>,
    from: Option<Vec<String>>,
    to: Option<Vec<String>>,
}

impl Default for LinkPartial {
    fn default() -> Self {
        LinkPartial {
            internal: None,
            from: None,
            to: None,
        }
    }
}

impl<'a> Partial<'a> for LinkPartial {
    const LOCATION: &'static str = "links";
    type Output = Self;
}

impl Linkable for Entry {

    fn links(&self) -> Result<LinkIter> {
        debug!("Getting internal links");
        trace!("Getting internal links from header of '{}' = {:?}", self.get_location(), self.get_header());

        let partial : LinkPartial = self
            .get_header()
            .read_partial::<LinkPartial>()?
            .unwrap_or_else(LinkPartial::default);

        partial
            .internal
            .unwrap_or_else(|| vec![])
            .into_iter()
            .chain(partial.from.unwrap_or_else(|| vec![]).into_iter())
            .chain(partial.to.unwrap_or_else(|| vec![]).into_iter())
            .map(PathBuf::from)
            .map(StoreId::new)
            .map(|r| r.map(Link::from))
            .collect::<Result<Vec<Link>>>()
            .map(LinkIter::new)
    }

    /// Get all links which are unidirectional links
    fn unidirectional_links(&self) -> Result<LinkIter> {
        debug!("Getting unidirectional links");
        trace!("Getting unidirectional links from header of '{}' = {:?}", self.get_location(), self.get_header());

        let iter = self.get_header()
            .read_partial::<LinkPartial>()?
            .unwrap_or_else(Default::default)
            .internal
            .unwrap_or_else(|| vec![])
            .into_iter();

        link_string_iter_to_link_iter(iter)
    }

    /// Get all links which are directional links, outgoing
    fn directional_links_to(&self) -> Result<LinkIter> {
        debug!("Getting directional links (to)");
        trace!("Getting unidirectional (to) links from header of '{}' = {:?}", self.get_location(), self.get_header());

        let iter = self.get_header()
            .read_partial::<LinkPartial>()?
            .unwrap_or_else(Default::default)
            .to
            .unwrap_or_else(|| vec![])
            .into_iter();

        link_string_iter_to_link_iter(iter)
    }

    /// Get all links which are directional links, incoming
    fn directional_links_from(&self) -> Result<LinkIter> {
        debug!("Getting directional links (from)");
        trace!("Getting unidirectional (from) links from header of '{}' = {:?}", self.get_location(), self.get_header());

        let iter = self.get_header()
            .read_partial::<LinkPartial>()?
            .unwrap_or_else(Default::default)
            .from
            .unwrap_or_else(|| vec![])
            .into_iter();

        link_string_iter_to_link_iter(iter)
    }

    fn add_link(&mut self, other: &mut Entry) -> Result<()> {
        debug!("Adding internal link: {:?}", other);
        let left_location  = self.get_location().to_str()?;
        let right_location = other.get_location().to_str()?;

        alter_linking(self, other, |mut left, mut right| {
            let mut left_internal = left.internal.unwrap_or_else(|| vec![]);
            trace!("left: {:?} <- {:?}", left_internal, right_location);
            left_internal.push(right_location);
            trace!("left: {:?}", left_internal);

            left_internal.sort_unstable();
            left_internal.dedup();

            let mut right_internal = right.internal.unwrap_or_else(|| vec![]);
            trace!("right: {:?} <- {:?}", right_internal, left_location);
            right_internal.push(left_location);
            trace!("right: {:?}", right_internal);

            right_internal.sort_unstable();
            right_internal.dedup();

            left.internal = Some(left_internal);
            right.internal = Some(right_internal);

            trace!("Finished: ({:?}, {:?})", left, right);
            Ok((left, right))
        })
    }

    fn remove_link(&mut self, other: &mut Entry) -> Result<()> {
        debug!("Remove internal link: {:?}", other);
        let left_location  = self.get_location().to_str()?;
        let right_location = other.get_location().to_str()?;

        alter_linking(self, other, |mut left, mut right| {
            let mut left_internal = left.internal.unwrap_or_else(|| vec![]);
            trace!("left: {:?} retaining {:?}", left_internal, right_location);
            left_internal.retain(|l| *l != right_location);
            trace!("left: {:?}", left_internal);

            left_internal.sort_unstable();
            left_internal.dedup();

            let mut right_internal = right.internal.unwrap_or_else(|| vec![]);
            trace!("right: {:?} retaining {:?}", right_internal, left_location);
            right_internal.retain(|l| *l != left_location);
            trace!("right: {:?}", right_internal);

            right_internal.sort_unstable();
            right_internal.dedup();

            left.internal = Some(left_internal);
            right.internal = Some(right_internal);

            trace!("Finished: ({:?}, {:?})", left, right);
            Ok((left, right))
        })
    }

    fn unlink(&mut self, store: &Store) -> Result<()> {
        debug!("Unlinking {:?}", self);
        for id in self.links()?.map(|l| l.get_store_id().clone()) {
            match store.get(id).context("Failed to get entry")? {
                Some(mut entry) => self.remove_link(&mut entry)?,
                None            => return Err(err_msg("Link target does not exist")),
            }
        }

        Ok(())
    }

    fn add_link_to(&mut self, other: &mut Entry) -> Result<()> {
        let left_location  = self.get_location().to_str()?;
        let right_location = other.get_location().to_str()?;

        alter_linking(self, other, |mut left, mut right| {
            let mut left_to = left.to.unwrap_or_else(|| vec![]);
            trace!("left_to: {:?} <- {:?}", left_to, right_location);
            left_to.push(right_location);
            trace!("left_to: {:?}", left_to);

            let mut right_from = right.from.unwrap_or_else(|| vec![]);
            trace!("right_from: {:?} <- {:?}", right_from, left_location);
            right_from.push(left_location);
            trace!("right_from: {:?}", right_from);

            left.to = Some(left_to);
            right.from = Some(right_from);

            trace!("Finished: ({:?}, {:?})", left, right);
            Ok((left, right))
        })
    }

    /// Remove a directional link: self -> otehr
    fn remove_link_to(&mut self, other: &mut Entry) -> Result<()> {
        let left_location  = self.get_location().to_str()?;
        let right_location = other.get_location().to_str()?;

        alter_linking(self, other, |mut left, mut right| {
            let mut left_to = left.to.unwrap_or_else(|| vec![]);
            trace!("left_to: {:?} retaining {:?}", left_to, right_location);
            left_to.retain(|l| *l != right_location);
            trace!("left_to: {:?}", left_to);

            let mut right_from = right.from.unwrap_or_else(|| vec![]);
            trace!("right_from: {:?} retaining {:?}", right_from, left_location);
            right_from.retain(|l| *l != left_location);
            trace!("right_from: {:?}", right_from);

            left.to = Some(left_to);
            right.from = Some(right_from);

            trace!("Finished: ({:?}, {:?})", left, right);
            Ok((left, right))
        })
    }

    /// Check whether an entry is linked to another entry
    fn is_linked_to(&self, other: &Entry) -> Result<bool> {
        let left_partial  = get_link_partial(self)?
            .ok_or_else(|| format_err!("Cannot read links from {}", self.get_location()))?;
        let right_partial = get_link_partial(&other)?
            .ok_or_else(|| format_err!("Cannot read links from {}", other.get_location()))?;

        let left_id       = self.get_location();
        let right_id      = other.get_location();

        let strary_contains = |sary: &Vec<String>, id: &StoreId| -> Result<bool> {
            sary.iter().map(|e| {
                StoreId::new(PathBuf::from(e)).map(|e| e == *id)
            }).fold(Ok(false), |a, e| a.and_then(|_| e))
        };

        let is_linked_from = |partial: &LinkPartial, id| {
            partial.from.as_ref().map(|f| strary_contains(f, id)).unwrap_or(Ok(false))
        };
        let is_linked_to   = |partial: &LinkPartial, id| {
            partial.to.as_ref().map(|t| strary_contains(t, id)).unwrap_or(Ok(false))
        };

        Ok({
            is_linked_from(&left_partial, &right_id)? && is_linked_from(&right_partial, &left_id)?
            ||
            is_linked_to(&left_partial, &right_id)? && is_linked_to(&right_partial, &left_id)?
        })
    }
}

fn link_string_iter_to_link_iter<I>(iter: I) -> Result<LinkIter>
    where I: Iterator<Item = String>
{
    iter.map(PathBuf::from)
        .map(StoreId::new)
        .map(|r| r.map(Link::from))
        .collect::<Result<Vec<Link>>>()
        .map(LinkIter::new)
}

fn alter_linking<F>(left: &mut Entry, right: &mut Entry, f: F) -> Result<()>
    where F: FnOnce(LinkPartial, LinkPartial) -> Result<(LinkPartial, LinkPartial)>
{
    debug!("Altering linkage of {:?} and {:?}", left, right);
    let get_partial = |e| -> Result<_> {
        Ok(get_link_partial(e)?.unwrap_or_else(LinkPartial::default))
    };

    let left_partial : LinkPartial = get_partial(&left)?;
    let right_partial : LinkPartial = get_partial(&right)?;

    trace!("Partial left before: {:?}", left_partial);
    trace!("Partial right before: {:?}", right_partial);

    let (left_partial, right_partial) = f(left_partial, right_partial)?;

    trace!("Partial left after: {:?}", left_partial);
    trace!("Partial right after: {:?}", right_partial);

    left.get_header_mut().insert_serialized("links", left_partial)?;
    right.get_header_mut().insert_serialized("links", right_partial)?;

    debug!("Finished altering linkage!");
    Ok(())
}

fn get_link_partial(entry: &Entry) -> Result<Option<LinkPartial>> {
    use failure::Error;
    entry.get_header().read_partial::<LinkPartial>().map_err(Error::from)
}


#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use libimagstore::store::Store;

    use super::Linkable;

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
        let links = entry.links();
        assert!(links.is_ok());
        let links = links.unwrap();
        assert_eq!(links.count(), 0);
    }

    #[test]
    fn test_link_two_entries() {
        setup_logging();
        let store = get_store();
        let mut e1 = store.create(PathBuf::from("test_link_two_entries1")).unwrap();
        assert!(e1.links().is_ok());

        let mut e2 = store.create(PathBuf::from("test_link_two_entries2")).unwrap();
        assert!(e2.links().is_ok());

        {
            let res = e1.add_link(&mut e2);
            debug!("Result = {:?}", res);
            assert!(res.is_ok());

            let e1_links = e1.links().unwrap().collect::<Vec<_>>();
            let e2_links = e2.links().unwrap().collect::<Vec<_>>();

            debug!("1 has links: {:?}", e1_links);
            debug!("2 has links: {:?}", e2_links);

            assert_eq!(e1_links.len(), 1);
            assert_eq!(e2_links.len(), 1);

            assert!(e1_links.first().map(|l| l.clone().eq_store_id(e2.get_location())).unwrap_or(false));
            assert!(e2_links.first().map(|l| l.clone().eq_store_id(e1.get_location())).unwrap_or(false));
        }

        {
            assert!(e1.remove_link(&mut e2).is_ok());

            debug!("{:?}", e2.to_str());
            let e2_links = e2.links().unwrap().collect::<Vec<_>>();
            assert_eq!(e2_links.len(), 0, "Expected [], got: {:?}", e2_links);

            debug!("{:?}", e1.to_str());
            let e1_links = e1.links().unwrap().collect::<Vec<_>>();
            assert_eq!(e1_links.len(), 0, "Expected [], got: {:?}", e1_links);

        }
    }

    #[test]
    #[clippy::cognitive_complexity = "49"]
    fn test_multiple_links() {
        setup_logging();
        let store = get_store();

        let mut e1 = store.retrieve(PathBuf::from("1")).unwrap();
        let mut e2 = store.retrieve(PathBuf::from("2")).unwrap();
        let mut e3 = store.retrieve(PathBuf::from("3")).unwrap();
        let mut e4 = store.retrieve(PathBuf::from("4")).unwrap();
        let mut e5 = store.retrieve(PathBuf::from("5")).unwrap();

        assert!(e1.add_link(&mut e2).is_ok());

        assert_eq!(e1.links().unwrap().count(), 1);
        assert_eq!(e2.links().unwrap().count(), 1);
        assert_eq!(e3.links().unwrap().count(), 0);
        assert_eq!(e4.links().unwrap().count(), 0);
        assert_eq!(e5.links().unwrap().count(), 0);

        assert!(e1.add_link(&mut e3).is_ok());

        assert_eq!(e1.links().unwrap().count(), 2);
        assert_eq!(e2.links().unwrap().count(), 1);
        assert_eq!(e3.links().unwrap().count(), 1);
        assert_eq!(e4.links().unwrap().count(), 0);
        assert_eq!(e5.links().unwrap().count(), 0);

        assert!(e1.add_link(&mut e4).is_ok());

        assert_eq!(e1.links().unwrap().count(), 3);
        assert_eq!(e2.links().unwrap().count(), 1);
        assert_eq!(e3.links().unwrap().count(), 1);
        assert_eq!(e4.links().unwrap().count(), 1);
        assert_eq!(e5.links().unwrap().count(), 0);

        assert!(e1.add_link(&mut e5).is_ok());

        assert_eq!(e1.links().unwrap().count(), 4);
        assert_eq!(e2.links().unwrap().count(), 1);
        assert_eq!(e3.links().unwrap().count(), 1);
        assert_eq!(e4.links().unwrap().count(), 1);
        assert_eq!(e5.links().unwrap().count(), 1);

        assert!(e5.remove_link(&mut e1).is_ok());

        assert_eq!(e1.links().unwrap().count(), 3);
        assert_eq!(e2.links().unwrap().count(), 1);
        assert_eq!(e3.links().unwrap().count(), 1);
        assert_eq!(e4.links().unwrap().count(), 1);
        assert_eq!(e5.links().unwrap().count(), 0);

        assert!(e4.remove_link(&mut e1).is_ok());

        assert_eq!(e1.links().unwrap().count(), 2);
        assert_eq!(e2.links().unwrap().count(), 1);
        assert_eq!(e3.links().unwrap().count(), 1);
        assert_eq!(e4.links().unwrap().count(), 0);
        assert_eq!(e5.links().unwrap().count(), 0);

        assert!(e3.remove_link(&mut e1).is_ok());

        assert_eq!(e1.links().unwrap().count(), 1);
        assert_eq!(e2.links().unwrap().count(), 1);
        assert_eq!(e3.links().unwrap().count(), 0);
        assert_eq!(e4.links().unwrap().count(), 0);
        assert_eq!(e5.links().unwrap().count(), 0);

        assert!(e2.remove_link(&mut e1).is_ok());

        assert_eq!(e1.links().unwrap().count(), 0);
        assert_eq!(e2.links().unwrap().count(), 0);
        assert_eq!(e3.links().unwrap().count(), 0);
        assert_eq!(e4.links().unwrap().count(), 0);
        assert_eq!(e5.links().unwrap().count(), 0);

    }

    #[test]
    fn test_link_deleting() {
        setup_logging();
        let store = get_store();

        let mut e1 = store.retrieve(PathBuf::from("1")).unwrap();
        let mut e2 = store.retrieve(PathBuf::from("2")).unwrap();

        assert_eq!(e1.links().unwrap().count(), 0);
        assert_eq!(e2.links().unwrap().count(), 0);

        assert!(e1.add_link(&mut e2).is_ok());

        assert_eq!(e1.links().unwrap().count(), 1);
        assert_eq!(e2.links().unwrap().count(), 1);

        assert!(e1.remove_link(&mut e2).is_ok());

        assert_eq!(e1.links().unwrap().count(), 0);
        assert_eq!(e2.links().unwrap().count(), 0);
    }

    #[test]
    fn test_link_deleting_multiple_links() {
        setup_logging();
        let store = get_store();

        let mut e1 = store.retrieve(PathBuf::from("1")).unwrap();
        let mut e2 = store.retrieve(PathBuf::from("2")).unwrap();
        let mut e3 = store.retrieve(PathBuf::from("3")).unwrap();

        assert_eq!(e1.links().unwrap().count(), 0);
        assert_eq!(e2.links().unwrap().count(), 0);
        assert_eq!(e3.links().unwrap().count(), 0);

        assert!(e1.add_link(&mut e2).is_ok()); // 1-2
        assert!(e1.add_link(&mut e3).is_ok()); // 1-2, 1-3

        assert_eq!(e1.links().unwrap().count(), 2);
        assert_eq!(e2.links().unwrap().count(), 1);
        assert_eq!(e3.links().unwrap().count(), 1);

        assert!(e2.add_link(&mut e3).is_ok()); // 1-2, 1-3, 2-3

        assert_eq!(e1.links().unwrap().count(), 2);
        assert_eq!(e2.links().unwrap().count(), 2);
        assert_eq!(e3.links().unwrap().count(), 2);

        assert!(e1.remove_link(&mut e2).is_ok()); // 1-3, 2-3

        assert_eq!(e1.links().unwrap().count(), 1);
        assert_eq!(e2.links().unwrap().count(), 1);
        assert_eq!(e3.links().unwrap().count(), 2);

        assert!(e1.remove_link(&mut e3).is_ok()); // 2-3

        assert_eq!(e1.links().unwrap().count(), 0);
        assert_eq!(e2.links().unwrap().count(), 1);
        assert_eq!(e3.links().unwrap().count(), 1);

        assert!(e2.remove_link(&mut e3).is_ok());

        assert_eq!(e1.links().unwrap().count(), 0);
        assert_eq!(e2.links().unwrap().count(), 0);
        assert_eq!(e3.links().unwrap().count(), 0);
    }

    #[test]
    fn test_directional_link() {
        use libimagstore::store::Entry;

        setup_logging();
        let store      = get_store();
        let mut entry1 = store.create(PathBuf::from("test_directional_link-1")).unwrap();
        let mut entry2 = store.create(PathBuf::from("test_directional_link-2")).unwrap();

        assert!(entry1.unidirectional_links().unwrap().next().is_none());
        assert!(entry2.unidirectional_links().unwrap().next().is_none());

        assert!(entry1.directional_links_to().unwrap().next().is_none());
        assert!(entry2.directional_links_to().unwrap().next().is_none());

        assert!(entry1.directional_links_from().unwrap().next().is_none());
        assert!(entry2.directional_links_from().unwrap().next().is_none());

        assert!(entry1.add_link_to(&mut entry2).is_ok());

        assert_eq!(entry1.unidirectional_links().unwrap().collect::<Vec<_>>(), vec![]);
        assert_eq!(entry2.unidirectional_links().unwrap().collect::<Vec<_>>(), vec![]);

        let get_directional_links_to = |e: &Entry| -> Result<Vec<String>, _> {
            e.directional_links_to()
                .unwrap()
                .map(|l| l.to_str())
                .collect::<Result<Vec<_>, _>>()
        };

        let get_directional_links_from = |e: &Entry| {
            e.directional_links_from()
                .unwrap()
                .map(|l| l.to_str())
                .collect::<Result<Vec<_>, _>>()
        };

        {
            let entry1_dir_links = get_directional_links_to(&entry1).unwrap();
            assert_eq!(entry1_dir_links, vec!["test_directional_link-2"]);
        }
        {
            let entry2_dir_links = get_directional_links_to(&entry2).unwrap();
            assert!(entry2_dir_links.is_empty());
        }

        {
            let entry1_dir_links = get_directional_links_from(&entry1).unwrap();
            assert!(entry1_dir_links.is_empty());
        }
        {
            let entry2_dir_links = get_directional_links_from(&entry2).unwrap();
            assert_eq!(entry2_dir_links, vec!["test_directional_link-1"]);
        }

    }

}
