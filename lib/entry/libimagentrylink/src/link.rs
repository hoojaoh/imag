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
use libimagstore::storeid::IntoStoreId;
use libimagstore::store::Store;
use libimagerror::errors::ErrorMsg as EM;

use toml::Value;
use failure::ResultExt;
use failure::Fallible as Result;
use failure::Error;

#[derive(Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum Link {
    Id          { link: StoreId },
    LinkTo   { link: StoreId },
    LinkFrom { link: StoreId },
}

impl Link {

    pub fn exists(&self, store: &Store) -> Result<bool> {
        match *self {
            Link::Id { ref link }             => store.exists(link.clone()),
            Link::LinkTo   { ref link }       => store.exists(link.clone()),
            Link::LinkFrom { ref link }       => store.exists(link.clone()),
        }
        .map_err(From::from)
    }

    pub fn to_str(&self) -> Result<String> {
        match *self {
            Link::Id { ref link }             => link.to_str(),
            Link::LinkTo   { ref link }       => link.to_str(),
            Link::LinkFrom { ref link }       => link.to_str(),
        }
        .map_err(From::from)
    }

    #[cfg(test)]
    pub(crate) fn eq_store_id(&self, id: &StoreId) -> bool {
        match self {
            &Link::Id { link: ref s }             => s.eq(id),
            &Link::LinkTo   { ref link }          => link.eq(id),
            &Link::LinkFrom { ref link }          => link.eq(id),
        }
    }

    /// Get the StoreId inside the Link, which is always present
    pub fn get_store_id(&self) -> &StoreId {
        match self {
            &Link::Id { link: ref s }             => s,
            &Link::LinkTo   { ref link }          => link,
            &Link::LinkFrom { ref link }          => link,
        }
    }

    pub(crate) fn to_value(&self) -> Result<Value> {
        match self {
            Link::Id       { ref link } => link,
            Link::LinkTo   { ref link } => link,
            Link::LinkFrom { ref link } => link,
        }
        .to_str()
        .map(Value::String)
        .context(EM::ConversionError)
        .map_err(Error::from)
    }

}

impl ::std::cmp::PartialEq for Link {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Link::Id { link: ref a }, &Link::Id { link: ref b }) => a.eq(&b),
            (&Link::LinkTo   { link: ref a }, &Link::LinkTo   { link: ref b })=> a.eq(&b),
            (&Link::LinkFrom { link: ref a }, &Link::LinkFrom { link: ref b })=> a.eq(&b),
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
            Link::LinkTo   { link }      => link,
            Link::LinkFrom { link }      => link,
        }
    }
}

impl IntoStoreId for Link {
    fn into_storeid(self) -> Result<StoreId> {
        match self {
            Link::Id { link }            => Ok(link),
            Link::LinkTo   { link }      => Ok(link),
            Link::LinkFrom { link }      => Ok(link),
        }
    }
}

impl AsRef<StoreId> for Link {
    fn as_ref(&self) -> &StoreId {
        match self {
            &Link::Id { ref link }            => &link,
            &Link::LinkTo   { ref link }      => &link,
            &Link::LinkFrom { ref link }      => &link,
        }
    }
}

