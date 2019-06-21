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


use std::vec::IntoIter;

use failure::Error;
use failure::ResultExt;
use failure::Fallible as Result;
use toml::Value;
use itertools::Itertools;

use libimagstore::store::Store;
use libimagstore::store::FileLockEntry;
use libimagerror::errors::ErrorMsg as EM;

use crate::link::Link;

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
        self.unique()
            .sorted()
            .into_iter() // Cannot sort toml::Value, hence uglyness here
            .map(|link| link.to_value().context(EM::ConversionError).map_err(Error::from))
            .collect()
    }
}

/// An Iterator that `Store::get()`s the Entries from the store while consumed
pub struct GetIter<'a>(IntoIter<Link>, &'a Store);

impl<'a> GetIter<'a> {
    pub fn new(i: IntoIter<Link>, store: &'a Store) -> GetIter<'a> {
        GetIter(i, store)
    }

    pub fn store(&self) -> &Store {
        self.1
    }
}

impl<'a> Iterator for GetIter<'a> {
    type Item = Result<FileLockEntry<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().and_then(|id| match self.1.get(id) {
            Ok(None)    => None,
            Ok(Some(x)) => Some(Ok(x)),
            Err(e)      => Some(Err(e).map_err(From::from)),
        })
    }

}

