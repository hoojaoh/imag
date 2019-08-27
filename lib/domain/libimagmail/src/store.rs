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
use std::path::PathBuf;
use std::fmt::Debug;

use failure::Fallible as Result;
use toml::Value;
use toml_query::insert::TomlValueInsertExt;

use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;
use libimagstore::storeid::StoreId;
use libimagstore::iter::Entries;
use libimagentryref::hasher::default::DefaultHasher;
use libimagentryref::reference::Config;
use libimagentryref::reference::RefFassade;
use libimagentryref::reference::Ref;
use libimagentryref::reference::MutRef;

use crate::mid::MessageId;
use crate::mail::Mail;
use crate::hasher::MailHasher;
use crate::util::get_message_id_for_mailfile;

pub trait MailStore<'a> {
    fn create_mail_from_path<P, CollName>(&'a self, p: P, collection_name: CollName, config: &Config)
        -> Result<FileLockEntry<'a>>
        where P: AsRef<Path> + Debug,
              CollName: AsRef<str> + Debug;

    fn get_mail_from_path<P>(&'a self, p: P)
        -> Result<Option<FileLockEntry<'a>>>
        where P: AsRef<Path> + Debug;

    fn retrieve_mail_from_path<P, CollName>(&'a self, p: P, collection_name: CollName, config: &Config)
        -> Result<FileLockEntry<'a>>
        where P: AsRef<Path> + Debug,
              CollName: AsRef<str> + Debug;

    fn get_mail(&'a self, mid: MessageId) -> Result<Option<FileLockEntry<'a>>>;
    fn all_mails(&'a self) -> Result<Entries<'a>>;
}

impl<'a> MailStore<'a> for Store {

    fn create_mail_from_path<P, CollName>(&'a self, p: P, collection_name: CollName, config: &Config)
        -> Result<FileLockEntry<'a>>
        where P: AsRef<Path> + Debug,
              CollName: AsRef<str> + Debug
    {
        let message_id = get_message_id_for_mailfile(p.as_ref())?;
        let new_sid    = crate::module_path::new_id(message_id.clone())?;

        let mut entry = self.create(new_sid)?;
        entry
            .as_ref_with_hasher_mut::<MailHasher>()
            .make_ref(p, collection_name, config, false)?;

        let _ = entry
            .get_header_mut()
            .insert("mail.message-id", Value::String(message_id))?;

        Ok(entry)
    }

    /// Same as MailStore::retrieve_mail_from_path() but uses Store::get() instead of
    /// Store::retrieve()
    fn get_mail_from_path<P>(&'a self, p: P)
        -> Result<Option<FileLockEntry<'a>>>
        where P: AsRef<Path> + Debug
    {
        let message_id = get_message_id_for_mailfile(p.as_ref())?;
        let new_sid    = crate::module_path::new_id(message_id.clone())?;

        match self.get(new_sid)? {
            Some(mut entry) => {
                if !entry.is_ref()? {
                    return Err(format_err!("{} is not a ref", entry.get_location()))
                }

                if p.as_ref().ends_with(entry.as_ref_with_hasher::<MailHasher>().get_relative_path()?) {
                    return Err(format_err!("{} is not a ref to {:?}",
                                           entry.get_location(),
                                           p.as_ref().display()))
                }

                let _ = entry.get_header_mut().insert("mail.message-id", Value::String(message_id))?;
                Ok(Some(entry))
            },
            None => Ok(None),
        }
    }

    fn retrieve_mail_from_path<P, CollName>(&'a self, p: P, collection_name: CollName, config: &Config)
        -> Result<FileLockEntry<'a>>
        where P: AsRef<Path> + Debug,
              CollName: AsRef<str> + Debug
    {
        let message_id = get_message_id_for_mailfile(&p)?;
        let new_sid    = crate::module_path::new_id(message_id.clone())?;
        let mut entry  = self.retrieve(new_sid)?;

        let _ = entry
            .get_header_mut()
            .insert("mail.message-id", Value::String(message_id))?;

        entry
            .as_ref_with_hasher_mut::<DefaultHasher>()
            .make_ref(p, collection_name, config, false)?;

        Ok(entry)
    }

    fn get_mail(&'a self, mid: MessageId) -> Result<Option<FileLockEntry<'a>>> {
        let mid_s : String = mid.into();
        self.get(StoreId::new(PathBuf::from(mid_s))?)
            .and_then(|oe| match oe {
                Some(e) => if e.is_mail()? {
                    Ok(Some(e))
                } else {
                    Err(format_err!("{} is not a mail entry", e.get_location()))
                },
                None => Ok(None)
            })
    }

    fn all_mails(&'a self) -> Result<Entries<'a>> {
        self.entries()?.in_collection("mail")
    }
}

