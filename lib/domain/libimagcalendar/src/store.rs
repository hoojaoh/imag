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

use std::path::Path;

use failure::Fallible as Result;
use toml::Value;
use toml_query::insert::TomlValueInsertExt;
use vobject::ICalendar;

use libimagentryutil::isa::Is;
use libimagentryref::reference::Config;
use libimagentryref::reference::MutRef;
use libimagentryref::hasher::default::DefaultHasher;
use libimagentryref::reference::RefFassade;
use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;

use crate::event::IsEvent;

pub trait EventStore<'a> {
    /// Imports events from a filepath
    ///
    /// # Note
    ///
    /// Because one icalendar file can theoretically hold several events, this function returns a
    /// list of entries.
    ///
    /// # Parameters
    ///
    /// This function is basically a wrapper around `libimagentryref::reference::RefMut::make_ref()`
    /// which also does some parsing and Store::create() ing entry objects.
    ///
    /// Because of the former function, this requires some parameters which are documented in the
    /// documentation of this function:
    ///
    /// libimagentryref::reference::RefMut::make_ref()
    ///
    /// Normally, `force` should be set to `false`.
    ///
    fn import_from_path<P, Coll>(&'a self, p: P, basepath_name: Coll, refconfig: &Config, force: bool)
        -> Result<Vec<Result<FileLockEntry<'a>>>>
        where P: AsRef<Path>,
              Coll: AsRef<str>;

    fn get_event_by_uid<ID>(&'a self, id: ID) -> Result<Option<FileLockEntry<'a>>>
        where ID: AsRef<str>;
}

impl<'a> EventStore<'a> for Store {
    fn import_from_path<P, Coll>(&'a self, p: P, basepath_name: Coll, refconfig: &Config, force: bool)
        -> Result<Vec<Result<FileLockEntry<'a>>>>
        where P: AsRef<Path>,
              Coll: AsRef<str>
    {
        let text = std::fs::read_to_string(p.as_ref())?;
        Ok(ICalendar::build(&text)?
            .events()
            .filter_map(|rresult| match rresult {
                Ok(event)      => Some(event),
                Err(component) => {
                    debug!("Ignoring non-event Component in {}: {}", p.as_ref().display(), component.name);
                    None
                }
            })
            .map(|event| {
                let uid = event.uid().ok_or_else(|| {
                    format_err!("Event in {} has no UID, but icalendar events must have one.", p.as_ref().display())
                })?;

                let sid        = crate::module_path::new_id(uid.raw())?;
                let uid_header = Value::String(uid.into_raw());

                let mut entry = self.create(sid)?;
                let _ = entry
                    .as_ref_with_hasher_mut::<DefaultHasher>()
                    .make_ref(p.as_ref(), basepath_name.as_ref(), refconfig, force)?;
                let _ = entry.get_header_mut().insert("calendar.event.uid", uid_header)?;
                let _ = entry.set_isflag::<IsEvent>()?;
                Ok(entry)
            })
            .collect())
    }

    fn get_event_by_uid<ID>(&'a self, id: ID) -> Result<Option<FileLockEntry<'a>>>
        where ID: AsRef<str>
    {
        self.get(crate::module_path::new_id(id.as_ref())?)
    }
}

