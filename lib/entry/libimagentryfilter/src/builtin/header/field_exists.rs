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

use libimagstore::store::Entry;

use toml_query::read::TomlValueReadExt;
use filters::failable::filter::FailableFilter;

use failure::Fallible as Result;
use failure::Error;

use crate::builtin::header::field_path::FieldPath;

pub struct FieldExists {
    header_field_path: FieldPath,
}

impl FieldExists {

    pub fn new(path: FieldPath) -> FieldExists {
        FieldExists {
            header_field_path: path,
        }
    }

}

impl FailableFilter<Entry> for FieldExists {
    type Error = Error;

    fn filter(&self, e: &Entry) -> Result<bool> {
        e.get_header()
            .read(&self.header_field_path[..])
            .map_err(Error::from)
            .map(|o| o.is_some())
    }

}


