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

use libimagrt::runtime::Runtime;

use crate::reference::Config as RefConfig;

pub fn get_ref_config(rt: &Runtime, app_name: &'static str) -> Result<RefConfig> {
    use toml_query::read::TomlValueReadExt;

    let setting_name = "ref.basepathes";

    rt.config()
        .ok_or_else(|| format_err!("No configuration, cannot find collection name for {}", app_name))?
        .read_deserialized::<RefConfig>(setting_name)?
        .ok_or_else(|| format_err!("Setting missing: {}", setting_name))
}


