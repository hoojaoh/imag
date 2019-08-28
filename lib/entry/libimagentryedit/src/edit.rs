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

use libimagrt::runtime::Runtime;
use libimagstore::store::Entry;

use failure::Fallible as Result;
use failure::Error;
use failure::ResultExt;
use failure::err_msg;

use libimagerror::errors::ErrorMsg as EM;

pub trait Edit {
    fn edit_content(&mut self, rt: &Runtime) -> Result<()>;
}

pub trait EditHeader : Edit {
    fn edit_header(&mut self, rt: &Runtime)             -> Result<()>;
    fn edit_header_and_content(&mut self, rt: &Runtime) -> Result<()>;
}

impl Edit for String {

    fn edit_content(&mut self, rt: &Runtime) -> Result<()> {
        edit_in_tmpfile(rt, self).map(|_| ())
    }

}

impl Edit for Entry {

    fn edit_content(&mut self, rt: &Runtime) -> Result<()> {
        edit_in_tmpfile(rt, self.get_content_mut())
            .map(|_| ())
    }

}

impl EditHeader for Entry {

    fn edit_header(&mut self, rt: &Runtime) -> Result<()> {
        let mut header = ::toml::ser::to_string_pretty(self.get_header())?;
        edit_in_tmpfile(rt, &mut header)?;
        let header     = ::toml::de::from_str(&header)?;
        *self.get_header_mut() = header;
        Ok(())
    }

    fn edit_header_and_content(&mut self, rt: &Runtime) -> Result<()> {
        let mut header_and_content = self.to_str()?;
        edit_in_tmpfile(rt, &mut header_and_content)?;
        self.replace_from_buffer(&header_and_content)
            .context("Failed to replace header and content from buffer")
            .map_err(Error::from)
    }

}

pub fn edit_in_tmpfile(rt: &Runtime, s: &mut String) -> Result<()> {
    use libimagutil::edit::edit_in_tmpfile_with_command;

    let editor = rt
        .editor()
        .context(err_msg("No editor"))?
        .ok_or_else(|| err_msg("No editor"))?;

    edit_in_tmpfile_with_command(editor, s)
        .context(EM::IO)
        .map_err(Error::from)
        .and_then(|worked| if !worked {
            Err(Error::from(EM::ExternalProcessError))
        } else {
            Ok(())
        })
}

