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

use std::ops::DerefMut;
use std::path::PathBuf;

use libimagrt::runtime::Runtime;
use libimagerror::trace::MapErrTrace;
use libimagerror::exit::ExitUnwrap;
use libimagstore::storeid::StoreId;

use crate::util::build_toml_header;

pub fn update(rt: &Runtime) {
    let scmd  = rt.cli().subcommand_matches("update").unwrap();
    let id    = scmd.value_of("id").unwrap(); // Safe by clap
    let path  = PathBuf::from(id);
    let path  = StoreId::new(path).map_err_trace_exit_unwrap();

    let _ = rt.store()
        .retrieve(path)
        .map(|mut locked_e| {
            {
                let e = locked_e.deref_mut();

                if let Some(new_content) = scmd.value_of("content") {
                    *e.get_content_mut() = String::from(new_content);
                    debug!("New content set");
                }

                *e.get_header_mut() = build_toml_header(scmd, e.get_header().clone());
                debug!("New header set");
            }

            rt.report_touched(locked_e.get_location()).unwrap_or_exit();
        });
}

