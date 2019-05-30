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

use toml::Value;
use uuid::Uuid;

use libimagstore::store::Entry;
use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;
use libimagstore::storeid::StoreIdIterator;
use libimagentrylink::linker::InternalLinker;
use libimagentryutil::isa::Is;
use libimagentryutil::isa::IsKindHeaderPathProvider;

use toml_query::insert::TomlValueInsertExt;

use failure::Fallible as Result;
use failure::ResultExt;
use failure::Error;
use failure::err_msg;

pub trait Annotateable {
    fn annotate<'a>(&mut self, store: &'a Store) -> Result<FileLockEntry<'a>>;
    fn denotate<'a>(&mut self, store: &'a Store, ann_name: &str) -> Result<Option<FileLockEntry<'a>>>;
    fn annotations(&self) -> Result<StoreIdIterator>;
    fn is_annotation(&self) -> Result<bool>;
}

provide_kindflag_path!(IsAnnotation, "annotation.is_annotation");

impl Annotateable for Entry {

    /// Annotate an entry, returns the new entry which is used to annotate
    fn annotate<'a>(&mut self, store: &'a Store) -> Result<FileLockEntry<'a>> {
        let ann_name = Uuid::new_v4().to_hyphenated().to_string();
        debug!("Creating annotation with name = {}", ann_name);

        store.retrieve(crate::module_path::new_id(ann_name.clone())?)
            .and_then(|mut anno| {
                {
                    let _ = anno.set_isflag::<IsAnnotation>()?;
                    let _ = anno
                        .get_header_mut()
                        .insert("annotation.name", Value::String(String::from(ann_name)))?;
                }
                Ok(anno)
            })
            .and_then(|mut anno| {
                anno.add_internal_link(self)
                    .context(err_msg("Linking error"))
                    .map_err(Error::from)
                    .map(|_| anno)
            })
    }

    // Removes the annotation `ann_name` from the current entry.
    // Fails if there's no such annotation entry or if the link to that annotation entry does not
    // exist.
    fn denotate<'a>(&mut self, store: &'a Store, ann_name: &str) -> Result<Option<FileLockEntry<'a>>> {
        if let Some(mut annotation) = store.get(crate::module_path::new_id(ann_name)?)? {
            let _ = self.remove_internal_link(&mut annotation)?;
            Ok(Some(annotation))
        } else {
            // error: annotation does not exist
            Err(format_err!("Annotation '{}' does not exist", ann_name)).map_err(Error::from)
        }
    }

    /// Get all annotations of an entry
    fn annotations(&self) -> Result<StoreIdIterator> {
        self.get_internal_links()
            .map(|it| {
                it.filter(|link| link.get_store_id().is_in_collection(&["annotation"]))
                    .map(|link| Ok(link.get_store_id().clone()))
            })
            .map(Box::new)
            .map(|inner| StoreIdIterator::new(inner))
    }

    fn is_annotation(&self) -> Result<bool> {
        self.is::<IsAnnotation>()
    }

}

