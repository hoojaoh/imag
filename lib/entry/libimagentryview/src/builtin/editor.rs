//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2018 Matthias Beyer <mail@beyermatthias.de> and contributors
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

use std::io::Write;

use libimagstore::store::Entry;
use libimagrt::runtime::Runtime;
use libimagentryedit::edit::edit_in_tmpfile;

use viewer::Viewer;
use error::Result;
use error::ResultExt;
use error::ViewErrorKind as VEK;

pub struct EditorView<'a>(&'a Runtime<'a>);

impl<'a> EditorView<'a> {
    pub fn new(rt: &'a Runtime) -> EditorView<'a> {
        EditorView(rt)
    }
}

impl<'a> Viewer for EditorView<'a> {
    fn view_entry<W>(&self, e: &Entry, _sink: &mut W) -> Result<()>
        where W: Write
    {
        let mut entry = e.to_str()?.clone().to_string();
        edit_in_tmpfile(self.0, &mut entry).chain_err(|| VEK::ViewError)
    }
}

