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
use toml::map::Map;
use failure::ResultExt;
use failure::Fallible as Result;
use failure::Error;

#[derive(Eq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum Link {
    Id          { link: StoreId },
    Annotated   { link: StoreId, annotation: String },
}

impl Link {

    pub fn exists(&self, store: &Store) -> Result<bool> {
        match *self {
            Link::Id { ref link }             => store.exists(link.clone()),
            Link::Annotated { ref link, .. }  => store.exists(link.clone()),
        }
        .map_err(From::from)
    }

    pub fn to_str(&self) -> Result<String> {
        match *self {
            Link::Id { ref link }             => link.to_str(),
            Link::Annotated { ref link, .. }  => link.to_str(),
        }
        .map_err(From::from)
    }


    pub(crate) fn eq_store_id(&self, id: &StoreId) -> bool {
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
    pub(crate) fn without_base(self) -> Link {
        match self {
            Link::Id { link: s } => Link::Id { link: s },
            Link::Annotated { link: s, annotation: ann } =>
                Link::Annotated { link: s, annotation: ann },
        }
    }

    pub(crate) fn to_value(&self) -> Result<Value> {
        match self {
            &Link::Id { link: ref s } =>
                s.to_str()
                .map(Value::String)
                .context(EM::ConversionError)
                .map_err(Error::from),
            &Link::Annotated { ref link, annotation: ref anno } => {
                link.to_str()
                    .map(Value::String)
                    .context(EM::ConversionError)
                    .map_err(Error::from)
                    .map(|link| {
                        let mut tab = Map::new();

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
    fn into_storeid(self) -> Result<StoreId> {
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

