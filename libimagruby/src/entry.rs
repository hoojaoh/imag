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

#[allow(unused_variables)]

use std::error::Error;

use ruru::{Class, Object, AnyObject, Boolean, RString, VM, Hash, NilClass, VerifiedObject};
use toml::Value;


#[cfg(feature = "tagging")]
use ruru::Array;

#[cfg(feature = "tagging")]
use ruru::result::Result as RRResult;

#[cfg(feature = "tagging")]
use libimagentrytag::tagable::Tagable;

use libimagstore::store::EntryHeader;
use libimagstore::store::EntryContent;
use libimagstore::store::Entry;
use libimagstore::storeid::StoreId;
use libimagstore::toml_ext::TomlValueExt;

use ruby_utils::IntoToml;
use toml_utils::IntoRuby;
use util::Wrap;
use util::Unwrap;
use cache::StoreHandle;

pub struct FileLockEntryHandle(StoreHandle, StoreId);

impl FileLockEntryHandle {
    pub fn new(sh: StoreHandle, id: StoreId) -> FileLockEntryHandle {
        FileLockEntryHandle(sh, id)
    }

    pub fn store_handle(&self) -> &StoreHandle {
        &self.0
    }

    pub fn fle_handle(&self) -> &StoreId {
        &self.1
    }
}

wrappable_struct!(FileLockEntryHandle, FileLockEntryWrapper, FLE_WRAPPER);
class!(RFileLockEntryHandle);
impl_wrap!(FileLockEntryHandle => FLE_WRAPPER);
impl_unwrap!(RFileLockEntryHandle => FileLockEntryHandle => FLE_WRAPPER);
impl_verified_object!(RFileLockEntryHandle);

/// Helper macro for operating on RUBY_STORE_CACHE object
///
/// This helps us calling operations on FileLockEntry objects.
///
/// What I do here: Fetch the Store object from the cache, fetch the appropriate FileLockEntry and
/// call the operation on it.
///
/// This could be improved with another cache, so not the store is cached but the FileLockEntry
/// only, but then we run into lifetime problems with the Store and its FileLockEntry objects.
/// Feel free to fix this, but for now, this is a workable solution.
///
#[macro_export]
macro_rules! call_on_fle_from_store {
    ($itself:ident ($wrapper:ident) -> $name:ident -> $operation:block) => {{
        let handle = $itself.get_data(&*$wrapper);
        let store_handle = handle.store_handle();
        call_on_store_by_handle! {
            store_handle named store inside {
                match store.get(handle.fle_handle().clone()) {
                    Ok(Some(mut $name)) => {
                        $operation
                    },
                    Ok(None) => {
                        VM::raise(Class::from_existing("RImagStoreReadError"), "Obj does not exist");
                        NilClass::new().to_any_object()
                    },
                    Err(e) => {
                        VM::raise(Class::from_existing("RImagStoreReadError"), e.description());
                        NilClass::new().to_any_object()
                    },
                }
            }
        }
    }};
    ($itself:ident ($wrapper:ident) -> $name:ident -> $operation: block on fail return $ex:expr) => {{
        let handle = $itself.get_data(&*$wrapper);
        let store_handle = handle.store_handle();
        call_on_store_by_handle! {
            store_handle named store inside {
                match store.get(handle.fle_handle().clone()) {
                    Ok(Some(mut $name)) => {
                        $operation
                    },
                    Ok(None) => {
                        VM::raise(Class::from_existing("RImagStoreReadError"), "Obj does not exist");
                        $ex
                    },
                    Err(e) => {
                        VM::raise(Class::from_existing("RImagStoreReadError"), e.description());
                        $ex
                    },
                }
            } on fail return $ex
        }
    }};
}


methods!(
    RFileLockEntryHandle,
    itself,

    fn r_get_location() -> AnyObject {
        call_on_fle_from_store!(itself (FLE_WRAPPER) -> fle -> { fle.get_location().clone().wrap() })
    }

    fn r_get_header() -> AnyObject {
        call_on_fle_from_store!(itself (FLE_WRAPPER) -> fle -> { fle.get_header().clone().wrap() })
    }

    fn r_set_header(hdr: Hash) -> NilClass {
        use ruby_utils::IntoToml;
        use toml::Value;

        let entryheader = match typecheck!(hdr or return NilClass::new()).into_toml() {
            Value::Table(t) => Value::Table(t),
            _ => {
                let ec = Class::from_existing("RImagEntryHeaderWriteError");
                VM::raise(ec, "Something weird happened. Hash seems to be not a Hash");
                return NilClass::new();
            },
        };

        call_on_fle_from_store!(itself (FLE_WRAPPER) -> fle -> {
            *fle.get_header_mut() = entryheader;
            NilClass::new().to_any_object()
        });

        NilClass::new()
    }

    fn r_get_content() -> AnyObject {
        call_on_fle_from_store!(itself (FLE_WRAPPER) -> fle -> {
            fle.get_content().clone().wrap()
        } on fail return NilClass::new().to_any_object())
    }

    fn r_set_content(ctt: RString) -> NilClass {
        use ruby_utils::IntoToml;
        use toml::Value;

        let content = match typecheck!(ctt).into_toml() {
            Value::String(s) => s,
            _ => {
                let ec = Class::from_existing("RImagEntryError");
                VM::raise(ec, "Something weird happened. String seems to be not a String");
                return NilClass::new();
            },
        };

        call_on_fle_from_store!(itself (FLE_WRAPPER) -> fle -> {
            *fle.get_content_mut() = content;
            NilClass::new().to_any_object()
        });

        NilClass::new()
    }

    //
    //
    // Tagging functionality from libimagentrytag
    //
    //

    // Get the tags of the entry.
    //
    // # Returns
    //
    // On success: Array[Tag]
    // On failure: Nil
    //
    fn r_get_tags() -> AnyObject {
        #[cfg(feature = "tagging")]
        #[inline]
        fn run(itself: &RFileLockEntryHandle) -> AnyObject {
            call_on_fle_from_store!(itself (FLE_WRAPPER) -> fle -> {
                match fle.get_tags() {
                    Err(e) => {
                        VM::raise(Class::from_existing("RuntimeError"), e.description());
                        NilClass::new().to_any_object()
                    }
                    Ok(ts) => ts
                        .into_iter()
                        .map(|s| RString::new(&s))
                        .map(|r| r.to_any_object())
                        .collect::<Array>()
                        .to_any_object(),
                }
            })
        }

        #[cfg(not(feature = "tagging"))]
        #[inline]
        fn run(itself: &RFileLockEntryHandle) -> AnyObject { NilClass::new().to_any_object() }

        run(&itself)
    }

    // Set the tags of the entry.
    //
    // # Returns
    //
    // On success: Nil
    // On failure: Nil, raising Exception
    //
    fn r_set_tags(ary: Array) -> NilClass {
        #[cfg(feature = "tagging")]
        #[inline]
        fn run(itself: &RFileLockEntryHandle, ary: RRResult<Array>) -> NilClass {
            let ary = typecheck!(ary or return NilClass::new());
            let tags = {
                let mut tags = Vec::with_capacity(ary.length());

                // no iterator chaining because we need to typecheck!() here
                for tag in ary.into_iter() {
                    let tag = match tag.try_convert_to::<RString>() {
                        Ok(t) => t,
                        Err(e) => {
                            VM::raise(Class::from_existing("RuntimeError"), e.description());
                            return NilClass::new();
                        },
                    };
                    tags.push(tag.to_string());
                }

                tags
            };

            call_on_fle_from_store!(itself (FLE_WRAPPER) -> fle -> {
                if let Err(e) = fle.set_tags(&tags) {
                    VM::raise(Class::from_existing("RuntimeError"), e.description());
                }
                NilClass::new()
            } on fail return NilClass::new())
        }

        #[cfg(not(feature = "tagging"))]
        #[inline]
        fn run(itself: &RFileLockEntryHandle, _: RRResult<Array>) -> NilClass {
            NilClass::new()
        }

        run(&itself, ary)
    }

    // Add a tag
    //
    // # Returns
    //
    // On success: Nil
    // On failure: Nil, raising Exception
    //
    fn r_add_tag(tag: RString) -> NilClass {
        #[cfg(feature = "tagging")]
        #[inline]
        fn run(itself: &RFileLockEntryHandle, tag: RRResult<RString>) -> NilClass {
            let tag = typecheck!(tag or return NilClass::new());

            call_on_fle_from_store!(itself (FLE_WRAPPER) -> fle -> {
                if let Err(e) = fle.add_tag(tag.to_string()) {
                    VM::raise(Class::from_existing("RuntimeError"), e.description());
                }
                NilClass::new()
            } on fail return NilClass::new())
        }

        #[cfg(not(feature = "tagging"))]
        #[inline]
        fn run(itself: &RFileLockEntryHandle, tag: RRResult<RString>) -> NilClass {
            NilClass::new().to_any_object()
        }

        run(&itself, tag)
    }

    // Remove a tag
    //
    // # Returns
    //
    // On success: Nil
    // On failure: Nil, raising Exception
    //
    fn r_remove_tag(tag: RString) -> NilClass {
        #[cfg(feature = "tagging")]
        #[inline]
        fn run(itself: &RFileLockEntryHandle, tag: RRResult<RString>) -> NilClass {
            let tag = typecheck!(tag or return NilClass::new());

            call_on_fle_from_store!(itself (FLE_WRAPPER) -> fle -> {
                if let Err(e) = fle.remove_tag(tag.to_string()) {
                    VM::raise(Class::from_existing("RuntimeError"), e.description());
                }
                NilClass::new()
            } on fail return NilClass::new())
        }

        #[cfg(not(feature = "tagging"))]
        #[inline]
        fn run(itself: &RFileLockEntryHandle, _: RRResult<RString>) -> NilClass { NilClass::new() }

        run(&itself, tag)
    }

    // Check whether a tag is set for the entry
    //
    // # Returns
    //
    // On success: Boolean
    // On failure: Nil, raising Exception
    //
    fn r_has_tag(s: RString) -> AnyObject {
        #[cfg(feature = "tagging")]
        #[inline]
        fn run(itself: &RFileLockEntryHandle, s: RRResult<RString>) -> AnyObject {
            let tag = typecheck!(s or return any NilClass::new());

            call_on_fle_from_store!(itself (FLE_WRAPPER) -> fle -> {
                match fle.has_tag(&tag.to_string()) {
                    Err(e) => {
                        VM::raise(Class::from_existing("RuntimeError"), e.description());
                        NilClass::new().to_any_object()
                    },
                    Ok(b) => Boolean::new(b).to_any_object()
                }
            } on fail return NilClass::new().to_any_object())
        }

        #[cfg(not(feature = "tagging"))]
        #[inline]
        fn run(itself: &RFileLockEntryHandle, s: RRResult<RString>) -> AnyObject {
            NilClass::new().to_any_object()
        }

        run(&itself, s)
    }

    // Check whether a list of tags are set for the entry
    //
    // # Returns
    //
    // On success: Boolean
    // On failure: Nil, raising Exception
    //
    fn r_has_tags(ary: Array) -> AnyObject {
        unimplemented!()
    }

);

wrappable_struct!(Value, EntryHeaderWrapper, ENTRY_HEADER_WRAPPER);
class!(REntryHeader);
impl_wrap!(Value => ENTRY_HEADER_WRAPPER);
impl_unwrap!(REntryHeader => Value => ENTRY_HEADER_WRAPPER);
impl_verified_object!(REntryHeader);

methods!(
    REntryHeader,
    itself,

    fn r_entry_header_new() -> AnyObject {
        Entry::default_header().wrap()
    }

    fn r_entry_header_insert(spec: RString, obj: AnyObject) -> Boolean {
        let spec = typecheck!(spec or return Boolean::new(false)).to_string();
        let obj = obj.unwrap(); // possibly not safe... TODO

        match itself.get_data(&*ENTRY_HEADER_WRAPPER).insert(&spec, obj.into_toml()) {
            Ok(b) => Boolean::new(b),
            Err(e) => {
                VM::raise(Class::from_existing("RImagEntryHeaderWriteError"), e.description());
                Boolean::new(false)
            }
        }
    }

    fn r_entry_header_set(spec: RString, obj: AnyObject) -> AnyObject {
        use ruru::NilClass;

        let spec = typecheck!(spec or return any Boolean::new(false)).to_string();
        let obj = obj.unwrap(); // possibly not safe... TODO

        match itself.get_data(&*ENTRY_HEADER_WRAPPER).set(&spec, obj.into_toml()) {
            Ok(Some(v)) => v.into_ruby(),
            Ok(None)    => NilClass::new().to_any_object(),
            Err(e) => {
                VM::raise(Class::from_existing("RImagEntryHeaderWriteError"), e.description());
                return Boolean::new(false).to_any_object();
            }
        }
    }

    fn r_entry_header_get(spec: RString) -> AnyObject {
        use ruru::NilClass;

        let spec = typecheck!(spec or return any Boolean::new(false)).to_string();

        match itself.get_data(&*ENTRY_HEADER_WRAPPER).read(&spec) {
            Ok(Some(v)) => v.into_ruby(),
            Ok(None)    => NilClass::new().to_any_object(),
            Err(e) => {
                VM::raise(Class::from_existing("RImagEntryHeaderReadError"), e.description());
                return Boolean::new(false).to_any_object();
            }
        }
    }

);

wrappable_struct!(EntryContent, EntryContentWrapper, ENTRY_CONTENT_WRAPPER);
class!(REntryContent);
impl_wrap!(EntryContent => ENTRY_CONTENT_WRAPPER);
impl_unwrap!(REntryContent => EntryContent => ENTRY_CONTENT_WRAPPER);

wrappable_struct!(Entry, EntryWrapper, ENTRY_WRAPPER);
class!(REntry);
impl_unwrap!(REntry => Entry => ENTRY_WRAPPER);

pub fn setup_filelockentry() -> Class {
    let mut class = Class::new("RFileLockEntryHandle", None);
    class.define(|itself| {
        itself.def("location", r_get_location);
        itself.def("header"  , r_get_header);
        itself.def("header=" , r_set_header);
        itself.def("content" , r_get_content);
        itself.def("content=", r_set_content);

        // Tagging functionality with libimagentrytag
        itself.def("get_tags"   , r_get_tags);
        itself.def("set_tags"   , r_set_tags);
        itself.def("add_tag"    , r_add_tag);
        itself.def("remove_tag" , r_remove_tag);
        itself.def("has_tag"    , r_has_tag);
        itself.def("has_tags"   , r_has_tags);

    });
    class
}

pub fn setup_entryheader() -> Class {
    let mut class = Class::new("REntryHeader", None);
    class.define(|itself| {
        itself.def("insert", r_entry_header_insert);
        itself.def("set"   , r_entry_header_set);
        itself.def("[]="   , r_entry_header_set);
        itself.def("read"  , r_entry_header_get);
        itself.def("[]"    , r_entry_header_get);
    });
    class
}

pub fn setup_entrycontent() -> Class {
    let string = Class::from_existing("String");
    Class::new("REntryContent", Some(&string))
}
