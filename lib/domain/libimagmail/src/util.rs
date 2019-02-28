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

use failure::Error;
use failure::Fallible as Result;
use failure::ResultExt;

/// Get the message header at a specific key
///
/// # WARNING
///
/// Key must be all-lowercase
///
/// # WARNING
///
/// Expensive, as mailfile gets read from disk internally.
/// TODO: Optimize
///
pub(crate) fn get_message_header_at_key<P: AsRef<Path>, K: AsRef<str>>(p: P, k: K) -> Result<String> {
    ::mailparse::parse_mail(::std::fs::read_to_string(p.as_ref())?.as_bytes())
        .context(format_err!("Cannot parse Email {}", p.as_ref().display()))?
        .headers
        .into_iter()
        .filter_map(|hdr| match hdr.get_key() {
            Err(e) => Some(Err(e).map_err(Error::from)),
            Ok(key) => {
                let lower_key = key.to_lowercase();
                trace!("Test: {} == {}", lower_key, k.as_ref());
                if lower_key == k.as_ref() {
                    Some(Ok(hdr))
                } else {
                    None
                }
            }
        })
        .next()
        .ok_or_else(|| format_err!("'{}' not found in {}", k.as_ref(), p.as_ref().display()))?
        .and_then(|hdr| hdr.get_value().map_err(Error::from))
}

pub(crate) fn get_message_id_for_mailfile<P: AsRef<Path>>(p: P) -> Result<String> {
    get_message_header_at_key(p, "message-id").map(strip_message_delimiters)
}

/// Strips message delimiters ('<' and '>') from a Message-ID field.
pub(crate) fn strip_message_delimiters<ID: AsRef<str>>(id: ID) -> String {
    let len  = id.as_ref().len();
    // We have to strip the '<' and '>' if there are any, because they do not belong to the
    // Message-Id at all
    id.as_ref()
        .chars()
        .enumerate()
        .filter(|(idx, chr)| !(*idx == 0 && *chr == '<' || *idx == len - 1 && *chr == '>'))
        .map(|tpl| tpl.1)
        .collect()
}

pub fn get_mail_text_content<P: AsRef<Path>>(p: P) -> Result<String> {
    ::mailparse::parse_mail(::std::fs::read_to_string(p.as_ref())?.as_bytes())
        .context(format_err!("Cannot parse Email {}", p.as_ref().display()))?
        .get_body()
        .map_err(Error::from)
}

