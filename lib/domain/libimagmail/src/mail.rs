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

use failure::Fallible as Result;
use failure::ResultExt;
use failure::Error;

use libimagstore::store::Entry;
use libimagentryutil::isa::Is;
use libimagentryutil::isa::IsKindHeaderPathProvider;
use libimagentryref::reference::Config as RefConfig;
use libimagentryref::reference::{Ref, RefFassade};

provide_kindflag_path!(pub IsMail, "mail.is_mail");

pub trait Mail : RefFassade {
    fn is_mail(&self)                                       -> Result<bool>;
    fn get_field(&self, refconfig: &RefConfig, field: &str) -> Result<Option<String>>;
    fn get_from(&self, refconfig: &RefConfig)               -> Result<Option<String>>;
    fn get_to(&self, refconfig: &RefConfig)                 -> Result<Option<String>>;
    fn get_subject(&self, refconfig: &RefConfig)            -> Result<Option<String>>;
    fn get_message_id(&self, refconfig: &RefConfig)         -> Result<Option<String>>;
    fn get_in_reply_to(&self, refconfig: &RefConfig)        -> Result<Option<String>>;
}

impl Mail for Entry {

    fn is_mail(&self) -> Result<bool> {
        self.is::<IsMail>()
    }

    /// Get a value of a single field of the mail file
    fn get_field(&self, refconfig: &RefConfig, field: &str) -> Result<Option<String>> {
        use std::fs::read_to_string;
        use crate::hasher::MailHasher;

        debug!("Getting field in mail: {:?}", field);
        let mail_file_location = self.as_ref_with_hasher::<MailHasher>().get_path(refconfig)?;

        match ::mailparse::parse_mail(read_to_string(mail_file_location.as_path())?.as_bytes())
            .context(format_err!("Cannot parse Email {}", mail_file_location.display()))?
            .headers
            .into_iter()
            .filter_map(|hdr| match hdr.get_key() {
                Err(e) => Some(Err(e).map_err(Error::from)),
                Ok(k) => if k == field {
                    Some(Ok(hdr))
                } else {
                    None
                }
            })
            .next()
        {
            None          => Ok(None),
            Some(Err(e))  => Err(e),
            Some(Ok(hdr)) => Ok(Some(hdr.get_value()?))
        }
    }

    /// Get a value of the `From` field of the mail file
    ///
    /// # Note
    ///
    /// Use `Mail::mail_header()` if you need to read more than one field.
    fn get_from(&self, refconfig: &RefConfig) -> Result<Option<String>> {
        self.get_field(refconfig, "From")
    }

    /// Get a value of the `To` field of the mail file
    ///
    /// # Note
    ///
    /// Use `Mail::mail_header()` if you need to read more than one field.
    fn get_to(&self, refconfig: &RefConfig) -> Result<Option<String>> {
        self.get_field(refconfig, "To")
    }

    /// Get a value of the `Subject` field of the mail file
    ///
    /// # Note
    ///
    /// Use `Mail::mail_header()` if you need to read more than one field.
    fn get_subject(&self, refconfig: &RefConfig) -> Result<Option<String>> {
        self.get_field(refconfig, "Subject")
    }

    /// Get a value of the `Message-ID` field of the mail file
    ///
    /// # Note
    ///
    /// Use `Mail::mail_header()` if you need to read more than one field.
    fn get_message_id(&self, refconfig: &RefConfig) -> Result<Option<String>> {
        self.get_field(refconfig, "Message-ID")
            .map(|o| o.map(crate::util::strip_message_delimiters))
    }

    /// Get a value of the `In-Reply-To` field of the mail file
    ///
    /// # Note
    ///
    /// Use `Mail::mail_header()` if you need to read more than one field.
    fn get_in_reply_to(&self, refconfig: &RefConfig) -> Result<Option<String>> {
        self.get_field(refconfig, "In-Reply-To")
    }

}

#[derive(Debug)]
pub struct MailHeader<'a>(Vec<::mailparse::MailHeader<'a>>);

impl<'a> From<Vec<::mailparse::MailHeader<'a>>> for MailHeader<'a> {
    fn from(mh: Vec<::mailparse::MailHeader<'a>>) -> Self {
        MailHeader(mh)
    }
}

impl<'a> MailHeader<'a> {
    /// Get a value of a single field of the mail file
    pub fn get_field(&self, field: &str) -> Result<Option<String>> {
        match self.0
            .iter()
            .filter_map(|hdr| match hdr.get_key() {
                Err(e) => Some(Err(e).map_err(Error::from)),
                Ok(key) => if key == field {
                    Some(Ok(hdr))
                } else {
                    None
                }
            })
            .next()
        {
            None          => Ok(None),
            Some(Err(e))  => Err(e),
            Some(Ok(hdr)) => Ok(Some(hdr.get_value()?))
        }
    }

    /// Get a value of the `From` field of the mail file
    pub fn get_from(&self) -> Result<Option<String>> {
        self.get_field("From")
    }

    /// Get a value of the `To` field of the mail file
    pub fn get_to(&self) -> Result<Option<String>> {
        self.get_field("To")
    }

    /// Get a value of the `Subject` field of the mail file
    pub fn get_subject(&self) -> Result<Option<String>> {
        self.get_field("Subject")
    }

    /// Get a value of the `Message-ID` field of the mail file
    pub fn get_message_id(&self) -> Result<Option<String>> {
        self.get_field("Message-ID")
    }

    /// Get a value of the `In-Reply-To` field of the mail file
    pub fn get_in_reply_to(&self) -> Result<Option<String>> {
        self.get_field("In-Reply-To")
    }

    // TODO: Offer functionality to load and parse mail _once_ from disk, and then use helper object
    // to offer access to header fields and content.
    //
    // With the existing functionality, one has to open-parse-close the file all the time, which is
    // _NOT_ optimal.
}
