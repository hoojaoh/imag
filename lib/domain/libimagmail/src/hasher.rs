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

use libimagentryref::hasher::Hasher;
use libimagerror::errors::ErrorMsg;
use libimagentryref::hasher::sha1::Sha1Hasher;

pub struct MailHasher;

impl Hasher for MailHasher {
    const NAME: &'static str = "MailHasher";

    /// hash the file at path `path`
    ///
    /// We create a sha1 over the path of the file (which is NOT safe, because mails can move) and
    /// the Message-ID of the mail itself.
    ///
    /// # TODO: Fix
    ///
    /// The file name is not constant with mail files, because flags are encoded in the filename.
    /// The path is not constant with mail files, because they can be moved between boxes.
    fn hash<P: AsRef<Path>>(path: P) -> Result<String> {
        let mut path_str = path
            .as_ref()
            .to_str()
            .map(String::from)
            .ok_or_else(|| ErrorMsg::UTF8Error)?;

        let message_id = crate::util::get_message_id_for_mailfile(path)?;

        path_str.push_str(&message_id);

        Ok(Sha1Hasher::sha1_hash(&path_str))
    }
}
