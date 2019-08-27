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

use std::io::Write;

use libimagstore::store::Entry;

use toml::ser::to_string;

use crate::viewer::Viewer;
use crate::error::Result;

pub struct StdoutViewer {
    view_header: bool,
    view_content: bool,
    trim_right: bool,
    wrap_content: Option<usize>,
}

impl StdoutViewer {

    pub fn new(view_header: bool, view_content: bool) -> StdoutViewer {
        StdoutViewer {
            view_header,
            view_content,
            trim_right: false,
            wrap_content: None,
        }
    }

    pub fn wrap_at(&mut self, wrap: usize) {
        self.wrap_content = Some(wrap)
    }

    pub fn trim_right(&mut self, trim: bool) {
        self.trim_right = trim;
    }

}

impl Viewer for StdoutViewer {

    fn view_entry<W>(&self, e: &Entry, sink: &mut W) -> Result<()>
        where W: Write
    {
        if self.view_header {
            let header = to_string(e.get_header()).unwrap_or(String::from("TOML Parser error"));
            writeln!(sink, "{}", header)?;
        }

        if self.view_content {
            let content = if self.trim_right {
                e.get_content().trim_end()
            } else {
                &e.get_content()
            };

            match self.wrap_content {

                Some(limit) => for line in ::textwrap::wrap(content, limit).iter() {
                    writeln!(sink, "{}", line)?;
                },
                None => writeln!(sink, "{}", content)?,
            }
        }

        Ok(())
    }

}
