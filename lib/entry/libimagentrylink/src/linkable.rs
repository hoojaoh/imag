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

    /// Add an internal link to the implementor object
    fn add_link(&mut self, link: &mut Entry) -> Result<()>;

    /// Remove an internal link from the implementor object
    fn remove_link(&mut self, link: &mut Entry) -> Result<()>;

    /// Remove _all_ internal links
    fn unlink(&mut self, store: &Store) -> Result<()>;

}

#[derive(Serialize, Deserialize, Debug)]
struct LinkPartial {
    internal: Option<Vec<String>>,
}

impl Default for LinkPartial {
    fn default() -> Self {
        LinkPartial {
            internal: None,
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
            .unwrap_or_else(|| LinkPartial::default());

        partial
            .internal
            .unwrap_or_else(|| vec![])
            .into_iter()
            .map(PathBuf::from)
            .map(StoreId::new)
            .map(|r| r.map(Link::from))
            .collect::<Result<Vec<Link>>>()
            .map(LinkIter::new)
    }

    fn add_link(&mut self, other: &mut Entry) -> Result<()> {
        debug!("Adding internal link: {:?}", other);
        let left_location  = self.get_location().to_str()?;
        let right_location = other.get_location().to_str()?;

        alter_linking(self, other, |mut left, mut right| {
            let mut left_internal = left.internal.unwrap_or_else(|| vec![]);
            left_internal.push(right_location);

            left_internal.sort_unstable();
            left_internal.dedup();

            let mut right_internal = right.internal.unwrap_or_else(|| vec![]);
            right_internal.push(left_location);

            right_internal.sort_unstable();
            right_internal.dedup();

            left.internal = Some(left_internal);
            right.internal = Some(right_internal);

            Ok((left, right))
        })
    }

    fn remove_link(&mut self, other: &mut Entry) -> Result<()> {
        debug!("Remove internal link: {:?}", other);
        let left_location  = self.get_location().to_str()?;
        let right_location = other.get_location().to_str()?;

        alter_linking(self, other, |mut left, mut right| {
            let mut left_internal = left.internal.unwrap_or_else(|| vec![]);
            left_internal.retain(|l| *l != right_location);

            left_internal.sort_unstable();
            left_internal.dedup();

            let mut right_internal = right.internal.unwrap_or_else(|| vec![]);
            right_internal.retain(|l| *l != left_location);

            right_internal.sort_unstable();
            right_internal.dedup();

            left.internal = Some(left_internal);
            right.internal = Some(right_internal);

            Ok((left, right))
        })
    }

    fn unlink(&mut self, store: &Store) -> Result<()> {
        for id in self.links()?.map(|l| l.get_store_id().clone()) {
            match store.get(id).context("Failed to get entry")? {
                Some(mut entry) => self.remove_link(&mut entry)?,
                None            => return Err(err_msg("Link target does not exist")),
            }
        }

        Ok(())
    }

}

fn alter_linking<F>(left: &mut Entry, right: &mut Entry, f: F) -> Result<()>
    where F: FnOnce(LinkPartial, LinkPartial) -> Result<(LinkPartial, LinkPartial)>
{
    debug!("Altering linkage of {:?} and {:?}", left, right);

    let get_partial = |entry: &mut Entry| -> Result<LinkPartial> {
        Ok(entry.get_header().read_partial::<LinkPartial>()?.unwrap_or_else(|| LinkPartial::default()))
    };

    let left_partial : LinkPartial = get_partial(left)?;
    let right_partial : LinkPartial = get_partial(right)?;

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
        assert_eq!(links.collect::<Vec<_>>().len(), 0);
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
    fn test_multiple_links() {
        setup_logging();
        let store = get_store();

        let mut e1 = store.retrieve(PathBuf::from("1")).unwrap();
        let mut e2 = store.retrieve(PathBuf::from("2")).unwrap();
        let mut e3 = store.retrieve(PathBuf::from("3")).unwrap();
        let mut e4 = store.retrieve(PathBuf::from("4")).unwrap();
        let mut e5 = store.retrieve(PathBuf::from("5")).unwrap();

        assert!(e1.add_link(&mut e2).is_ok());

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e4.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e5.links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e1.add_link(&mut e3).is_ok());

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 2);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e4.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e5.links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e1.add_link(&mut e4).is_ok());

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 3);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e4.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e5.links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e1.add_link(&mut e5).is_ok());

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 4);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e4.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e5.links().unwrap().collect::<Vec<_>>().len(), 1);

        assert!(e5.remove_link(&mut e1).is_ok());

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 3);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e4.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e5.links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e4.remove_link(&mut e1).is_ok());

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 2);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e4.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e5.links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e3.remove_link(&mut e1).is_ok());

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e4.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e5.links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e2.remove_link(&mut e1).is_ok());

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e4.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e5.links().unwrap().collect::<Vec<_>>().len(), 0);

    }

    #[test]
    fn test_link_deleting() {
        setup_logging();
        let store = get_store();

        let mut e1 = store.retrieve(PathBuf::from("1")).unwrap();
        let mut e2 = store.retrieve(PathBuf::from("2")).unwrap();

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e1.add_link(&mut e2).is_ok());

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 1);

        assert!(e1.remove_link(&mut e2).is_ok());

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 0);
    }

    #[test]
    fn test_link_deleting_multiple_links() {
        setup_logging();
        let store = get_store();

        let mut e1 = store.retrieve(PathBuf::from("1")).unwrap();
        let mut e2 = store.retrieve(PathBuf::from("2")).unwrap();
        let mut e3 = store.retrieve(PathBuf::from("3")).unwrap();

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 0);

        assert!(e1.add_link(&mut e2).is_ok()); // 1-2
        assert!(e1.add_link(&mut e3).is_ok()); // 1-2, 1-3

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 2);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 1);

        assert!(e2.add_link(&mut e3).is_ok()); // 1-2, 1-3, 2-3

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 2);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 2);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 2);

        assert!(e1.remove_link(&mut e2).is_ok()); // 1-3, 2-3

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 2);

        assert!(e1.remove_link(&mut e3).is_ok()); // 2-3

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 1);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 1);

        assert!(e2.remove_link(&mut e3).is_ok());

        assert_eq!(e1.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e2.links().unwrap().collect::<Vec<_>>().len(), 0);
        assert_eq!(e3.links().unwrap().collect::<Vec<_>>().len(), 0);
    }

}
