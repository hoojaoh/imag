//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015, 2016 Matthias Beyer <mail@beyermatthias.de> and contributors
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

use std::collections::BTreeMap;
#[cfg(test)]
use std::path::PathBuf;

use libimagstore::storeid::StoreId;
use libimagstore::storeid::IntoStoreId;
use libimagstore::store::Entry;
use libimagstore::store::Result as StoreResult;
use libimagstore::toml_ext::TomlValueExt;
use libimagerror::into::IntoError;

use error::LinkErrorKind as LEK;
use error::MapErrInto;
use result::Result;
use self::iter::LinkIter;
use self::iter::IntoValues;

use toml::Value;

#[derive(Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum Link {
    Id          { link: StoreId },
    Annotated   { link: StoreId, annotation: String },
}

impl Link {

    pub fn exists(&self) -> Result<bool> {
        match *self {
            Link::Id { ref link }             => link.exists(),
            Link::Annotated { ref link, .. }  => link.exists(),
        }
        .map_err_into(LEK::StoreIdError)
    }

    pub fn to_str(&self) -> Result<String> {
        match *self {
            Link::Id { ref link }             => link.to_str(),
            Link::Annotated { ref link, .. }  => link.to_str(),
        }
        .map_err_into(LEK::StoreReadError)
    }


    fn eq_store_id(&self, id: &StoreId) -> bool {
        match self {
            &Link::Id { link: ref s }             => s.eq(id),
            &Link::Annotated { link: ref s, .. }  => s.eq(id),
        }
    }

    /// Get the StoreId inside the Link, which is always present
    pub fn get_store_id(&self) -> &StoreId {
        match self {
            &Link::Id { link: ref s }             => s,
            &Link::Annotated { link: ref s, .. }  => s,
        }
    }

    /// Helper wrapper around Link for StoreId
    fn without_base(self) -> Link {
        match self {
            Link::Id { link: s } => Link::Id { link: s.without_base() },
            Link::Annotated { link: s, annotation: ann } =>
                Link::Annotated { link: s.without_base(), annotation: ann },
        }
    }

    /// Helper wrapper around Link for StoreId
    #[cfg(test)]
    fn with_base(self, pb: PathBuf) -> Link {
        match self {
            Link::Id { link: s } => Link::Id { link: s.with_base(pb) },
            Link::Annotated { link: s, annotation: ann } =>
                Link::Annotated { link: s.with_base(pb), annotation: ann },
        }
    }

    fn to_value(&self) -> Result<Value> {
        match self {
            &Link::Id { link: ref s } =>
                s.to_str().map(Value::String).map_err_into(LEK::InternalConversionError),
            &Link::Annotated { ref link, annotation: ref anno } => {
                link.to_str()
                    .map(Value::String)
                    .map_err_into(LEK::InternalConversionError)
                    .map(|link| {
                        let mut tab = BTreeMap::new();

                        tab.insert("link".to_owned(),       link);
                        tab.insert("annotation".to_owned(), Value::String(anno.clone()));
                        Value::Table(tab)
                    })
            }
        }
    }

}

impl ::std::cmp::PartialEq for Link {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Link::Id { link: ref a }, &Link::Id { link: ref b }) => a.eq(&b),
            (&Link::Annotated { link: ref a, annotation: ref ann1 },
             &Link::Annotated { link: ref b, annotation: ref ann2 }) =>
                (a, ann1).eq(&(b, ann2)),
            _ => false,
        }
    }
}

impl From<StoreId> for Link {

    fn from(s: StoreId) -> Link {
        Link::Id { link: s }
    }
}

impl Into<StoreId> for Link {
    fn into(self) -> StoreId {
        match self {
            Link::Id { link }            => link,
            Link::Annotated { link, .. } => link,
        }
    }
}

impl IntoStoreId for Link {
    fn into_storeid(self) -> StoreResult<StoreId> {
        match self {
            Link::Id { link }            => Ok(link),
            Link::Annotated { link, .. } => Ok(link),
        }
    }
}

impl AsRef<StoreId> for Link {
    fn as_ref(&self) -> &StoreId {
        match self {
            &Link::Id { ref link }            => &link,
            &Link::Annotated { ref link, .. } => &link,
        }
    }
}

pub trait InternalLinker {

    /// Get the internal links from the implementor object
    fn get_internal_links(&self) -> Result<LinkIter>;

    /// Set the internal links for the implementor object
    fn set_internal_links(&mut self, links: Vec<&mut Entry>) -> Result<LinkIter>;

    /// Add an internal link to the implementor object
    fn add_internal_link(&mut self, link: &mut Entry) -> Result<()>;

    /// Remove an internal link from the implementor object
    fn remove_internal_link(&mut self, link: &mut Entry) -> Result<()>;

    /// Add internal annotated link
    fn add_internal_annotated_link(&mut self, link: &mut Entry, annotation: String) -> Result<()>;
}

pub mod iter {
    use std::vec::IntoIter;
    use super::Link;

    use error::LinkErrorKind as LEK;
    use error::MapErrInto;
    use result::Result;

    use toml::Value;
    use itertools::Itertools;

    use libimagstore::store::Store;
    use libimagstore::store::FileLockEntry;

    pub struct LinkIter(IntoIter<Link>);

    impl LinkIter {

        pub fn new(v: Vec<Link>) -> LinkIter {
            LinkIter(v.into_iter())
        }

        pub fn into_getter(self, store: &Store) -> GetIter {
            GetIter(self.0, store)
        }

    }

    impl Iterator for LinkIter {
        type Item = Link;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.next()
        }
    }

    pub trait IntoValues {
        fn into_values(self) -> Vec<Result<Value>>;
    }

    impl<I: Iterator<Item = Link>> IntoValues for I {
        fn into_values(self) -> Vec<Result<Value>> {
            self.map(|s| s.without_base())
                .unique()
                .sorted()
                .into_iter() // Cannot sort toml::Value, hence uglyness here
                .map(|link| link.to_value().map_err_into(LEK::InternalConversionError))
                .collect()
        }
    }

    /// An Iterator that `Store::get()`s the Entries from the store while consumed
    pub struct GetIter<'a>(IntoIter<Link>, &'a Store);

    impl<'a> GetIter<'a> {
        pub fn new(i: IntoIter<Link>, store: &'a Store) -> GetIter<'a> {
            GetIter(i, store)
        }

        /// Turn this iterator into a LinkGcIter, which `Store::delete()`s entries that are not
        /// linked to any other entry.
        pub fn delete_unlinked(self) -> DeleteUnlinkedIter<'a> {
            DeleteUnlinkedIter(self)
        }

        /// Turn this iterator into a FilterLinksIter that removes all entries that are not linked
        /// to any other entry, by filtering them out the iterator.
        ///
        /// This does _not_ remove the entries from the store.
        pub fn without_unlinked(self) -> FilterLinksIter<'a> {
            FilterLinksIter::new(self, Box::new(|links: &[Link]| links.len() > 0))
        }

        /// Turn this iterator into a FilterLinksIter that removes all entries that have less than
        /// `n` links to any other entries.
        ///
        /// This does _not_ remove the entries from the store.
        pub fn with_less_than_n_links(self, n: usize) -> FilterLinksIter<'a> {
            FilterLinksIter::new(self, Box::new(move |links: &[Link]| links.len() < n))
        }

        /// Turn this iterator into a FilterLinksIter that removes all entries that have more than
        /// `n` links to any other entries.
        ///
        /// This does _not_ remove the entries from the store.
        pub fn with_more_than_n_links(self, n: usize) -> FilterLinksIter<'a> {
            FilterLinksIter::new(self, Box::new(move |links: &[Link]| links.len() > n))
        }

        /// Turn this iterator into a FilterLinksIter that removes all entries where the predicate
        /// `F` returns false
        ///
        /// This does _not_ remove the entries from the store.
        pub fn filtered_for_links(self, f: Box<Fn(&[Link]) -> bool>) -> FilterLinksIter<'a> {
            FilterLinksIter::new(self, f)
        }

        pub fn store(&self) -> &Store {
            self.1
        }
    }

    impl<'a> Iterator for GetIter<'a> {
        type Item = Result<FileLockEntry<'a>>;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.next().and_then(|id| match self.1.get(id).map_err_into(LEK::StoreReadError) {
                Ok(None)    => None,
                Ok(Some(x)) => Some(Ok(x)),
                Err(e)      => Some(Err(e)),
            })
        }

    }

    /// An iterator helper that has a function F.
    ///
    /// If the function F returns `false` for the number of links, the entry is ignored, else it is
    /// taken.
    pub struct FilterLinksIter<'a>(GetIter<'a>, Box<Fn(&[Link]) -> bool>);

    impl<'a> FilterLinksIter<'a> {
        pub fn new(gi: GetIter<'a>, f: Box<Fn(&[Link]) -> bool>) -> FilterLinksIter<'a> {
            FilterLinksIter(gi, f)
        }
    }

    impl<'a> Iterator for FilterLinksIter<'a> {
        type Item = Result<FileLockEntry<'a>>;

        fn next(&mut self) -> Option<Self::Item> {
            use internal::InternalLinker;

            loop {
                match self.0.next() {
                    Some(Ok(fle)) => {
                        let links = match fle.get_internal_links().map_err_into(LEK::StoreReadError)
                        {
                            Err(e) => return Some(Err(e)),
                            Ok(links) => links.collect::<Vec<_>>(),
                        };
                        if !(self.1)(&links) {
                            continue;
                        } else {
                            return Some(Ok(fle));
                        }
                    },
                    Some(Err(e)) => return Some(Err(e)),
                    None => break,
                }
            }
            None
        }

    }

    /// An iterator that removes all Items from the iterator that are not linked anymore by calling
    /// `Store::delete()` on them.
    ///
    /// It yields only items which are somehow linked to another entry
    ///
    /// # Warning
    ///
    /// Deletes entries from the store.
    ///
    pub struct DeleteUnlinkedIter<'a>(GetIter<'a>);

    impl<'a> Iterator for DeleteUnlinkedIter<'a> {
        type Item = Result<FileLockEntry<'a>>;

        fn next(&mut self) -> Option<Self::Item> {
            use internal::InternalLinker;

            loop {
                match self.0.next() {
                    Some(Ok(fle)) => {
                        let links = match fle.get_internal_links().map_err_into(LEK::StoreReadError)
                        {
                            Err(e) => return Some(Err(e)),
                            Ok(links) => links,
                        };
                        if links.count() == 0 {
                            match self.0
                                 .store()
                                 .delete(fle.get_location().clone())
                                 .map_err_into(LEK::StoreWriteError)
                            {
                                Ok(x)  => x,
                                Err(e) => return Some(Err(e)),
                            }
                        } else {
                            return Some(Ok(fle));
                        }
                    },
                    Some(Err(e)) => return Some(Err(e)),
                    None => break,
                }
            }
            None
        }

    }

}

pub mod pred {
    pub mod entry {
        //! Predicate types to be used on iterators over `libimagstore::store::Entry` to filter
        //! for links

        use std::error::Error;
        use std::cell::RefCell;

        use filters::filter::Filter;

        use libimagstore::store::Entry;

        use super::super::Link;
        use super::super::InternalLinker;

        pub enum LinkCountOp {
            LT,
            EQ,
            GT,
        }

        pub struct FilterLinkCount {
            op: LinkCountOp,
            n: usize,
            errfn: Box<Fn(&Error) -> bool>,
        }

        impl FilterLinkCount {

            /// Construct a new FilterLinkCount object using the `LinkCountOp` as comperator and `n`
            /// as righthandside of the comparison.
            ///
            /// If the retrieval of the internal links for the `Entry` failed, use the `errfn`
            /// function to decide whether the entry should be filtered out.
            pub fn new(op: LinkCountOp, n: usize, errfn: Box<Fn(&Error) -> bool>) -> FilterLinkCount {
                FilterLinkCount {
                    op: op,
                    n: n,
                    errfn: errfn
                }
            }
        }

        impl Filter<Entry> for FilterLinkCount {
            fn filter(&self, entry: &Entry) -> bool {
                match entry.get_internal_links() {
                    Err(e)    => (self.errfn)(&e),
                    Ok(links) => match self.op {
                        LinkCountOp::LT => links.count() < self.n,
                        LinkCountOp::EQ => links.count() == self.n,
                        LinkCountOp::GT => links.count() > self.n,
                    },
                }
            }
        }
    }

}

impl InternalLinker for Entry {

    fn get_internal_links(&self) -> Result<LinkIter> {
        process_rw_result(self.get_header().read("imag.links"))
    }

    /// Set the links in a header and return the old links, if any.
    fn set_internal_links(&mut self, links: Vec<&mut Entry>) -> Result<LinkIter> {
        use internal::iter::IntoValues;

        let self_location = self.get_location().clone();
        let mut new_links = vec![];

        for link in links {
            if let Err(e) = add_foreign_link(link, self_location.clone()) {
                return Err(e);
            }
            new_links.push(link.get_location().clone().into());
        }

        let new_links = try!(LinkIter::new(new_links)
                             .into_values()
                             .into_iter()
                             .fold(Ok(vec![]), |acc, elem| {
                                acc.and_then(move |mut v| {
                                    elem.map_err_into(LEK::InternalConversionError)
                                        .map(|e| {
                                            v.push(e);
                                            v
                                        })
                                })
                            }));
        process_rw_result(self.get_header_mut().set("imag.links", Value::Array(new_links)))
    }

    fn add_internal_link(&mut self, link: &mut Entry) -> Result<()> {
        let location = link.get_location().clone().into();
        add_internal_link_with_instance(self, link, location)
    }

    fn remove_internal_link(&mut self, link: &mut Entry) -> Result<()> {
        let own_loc   = self.get_location().clone().without_base();
        let other_loc = link.get_location().clone().without_base();

        debug!("Removing internal link from {:?} to {:?}", own_loc, other_loc);

        link.get_internal_links()
            .and_then(|links| {
                debug!("Rewriting own links for {:?}, without {:?}", other_loc, own_loc);
                let links = links.filter(|l| !l.eq_store_id(&own_loc));
                rewrite_links(link.get_header_mut(), links)
            })
            .and_then(|_| {
                self.get_internal_links()
                    .and_then(|links| {
                        debug!("Rewriting own links for {:?}, without {:?}", own_loc, other_loc);
                        let links = links.filter(|l| !l.eq_store_id(&other_loc));
                        rewrite_links(self.get_header_mut(), links)
                    })
            })
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
    let links = try!(links.into_values()
                     .into_iter()
                     .fold(Ok(vec![]), |acc, elem| {
                        acc.and_then(move |mut v| {
                            elem.map_err_into(LEK::InternalConversionError)
                                .map(|e| {
                                    v.push(e);
                                    v
                                })
                        })
                     }));

    debug!("Setting new link array: {:?}", links);
    let process = header.set("imag.links", Value::Array(links));
    process_rw_result(process).map(|_| ())
}

/// When Linking A -> B, the specification wants us to link back B -> A.
/// This is a helper function which does this.
fn add_foreign_link(target: &mut Entry, from: StoreId) -> Result<()> {
    debug!("Linking back from {:?} to {:?}", target.get_location(), from);
    target.get_internal_links()
        .and_then(|links| {
            let links = try!(links
                             .chain(LinkIter::new(vec![from.into()]))
                             .into_values()
                             .into_iter()
                             .fold(Ok(vec![]), |acc, elem| {
                                acc.and_then(move |mut v| {
                                    elem.map_err_into(LEK::InternalConversionError)
                                        .map(|e| {
                                            v.push(e);
                                            v
                                        })
                                })
                             }));
            debug!("Setting links in {:?}: {:?}", target.get_location(), links);
            process_rw_result(target.get_header_mut().set("imag.links", Value::Array(links)))
                .map(|_| ())
        })
}

fn process_rw_result(links: StoreResult<Option<Value>>) -> Result<LinkIter> {
    use std::path::PathBuf;

    let links = match links {
        Err(e) => {
            debug!("RW action on store failed. Generating LinkError");
            return Err(LEK::EntryHeaderReadError.into_error_with_cause(Box::new(e)))
        },
        Ok(None) => {
            debug!("We got no value from the header!");
            return Ok(LinkIter::new(vec![]))
        },
        Ok(Some(Value::Array(l))) => l,
        Ok(Some(_)) => {
            debug!("We expected an Array for the links, but there was a non-Array!");
            return Err(LEK::ExistingLinkTypeWrong.into());
        }
    };

    if !links.iter().all(|l| is_match!(*l, Value::String(_)) || is_match!(*l, Value::Table(_))) {
        debug!("At least one of the Values which were expected in the Array of links is not a String or a Table!");
        debug!("Generating LinkError");
        return Err(LEK::ExistingLinkTypeWrong.into());
    }

    let links : Vec<Link> = try!(links.into_iter()
        .map(|link| {
            debug!("Matching the link: {:?}", link);
            match link {
                Value::String(s) => StoreId::new_baseless(PathBuf::from(s))
                    .map_err_into(LEK::StoreIdError)
                    .map(|s| Link::Id { link: s })
                    ,
                Value::Table(mut tab) => {
                    debug!("Destructuring table");
                    if !tab.contains_key("link")
                    || !tab.contains_key("annotation") {
                        debug!("Things missing... returning Error instance");
                        Err(LEK::LinkParserError.into_error())
                    } else {
                        let link = try!(tab.remove("link")
                            .ok_or(LEK::LinkParserFieldMissingError.into_error()));

                        let anno = try!(tab.remove("annotation")
                            .ok_or(LEK::LinkParserFieldMissingError.into_error()));

                        debug!("Ok, here we go with building a Link::Annotated");
                        match (link, anno) {
                            (Value::String(link), Value::String(anno)) => {
                                StoreId::new_baseless(PathBuf::from(link))
                                    .map_err_into(LEK::StoreIdError)
                                    .map(|link| {
                                        Link::Annotated {
                                            link: link,
                                            annotation: anno,
                                        }
                                    })
                            },
                            _ => Err(LEK::LinkParserFieldTypeError.into_error()),
                        }
                    }
                }
                _ => unreachable!(),
            }
        })
        .collect());

    debug!("Ok, the RW action was successful, returning link vector now!");
    Ok(LinkIter::new(links))
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use libimagstore::store::Store;

    use super::InternalLinker;

    fn setup_logging() {
        use env_logger;
        let _ = env_logger::init().unwrap_or(());
    }

    pub fn get_store() -> Store {
        Store::new(PathBuf::from("/"), None).unwrap()
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

            assert!(e1_links.first().map(|l| l.clone().with_base(store.path().clone()).eq_store_id(e2.get_location())).unwrap_or(false));
            assert!(e2_links.first().map(|l| l.clone().with_base(store.path().clone()).eq_store_id(e1.get_location())).unwrap_or(false));
        }

        {
            assert!(e1.remove_internal_link(&mut e2).is_ok());

            println!("{:?}", e2.to_str());
            let e2_links = e2.get_internal_links().unwrap().collect::<Vec<_>>();
            assert_eq!(e2_links.len(), 0, "Expected [], got: {:?}", e2_links);

            println!("{:?}", e1.to_str());
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

}

