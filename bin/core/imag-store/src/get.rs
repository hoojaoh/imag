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

use std::path::PathBuf;

use libimagrt::runtime::Runtime;
use libimagerror::trace::MapErrTrace;
use libimagerror::exit::ExitUnwrap;
use libimagstore::storeid::StoreId;

use crate::retrieve::print_entry;

pub fn get(rt: &Runtime) {
    let scmd = rt.cli().subcommand_matches("get").unwrap();

    let id    = scmd.value_of("id").unwrap(); // safe by clap
    let path  = PathBuf::from(id);
    let path  = StoreId::new(path).map_err_trace_exit_unwrap();
    debug!("path = {:?}", path);

    match rt.store().get(path.clone()).map_err_trace_exit_unwrap() {
        Some(entry) => {
            print_entry(rt, scmd, entry);
            rt.report_touched(&path).unwrap_or_exit();
        },
        None        => info!("No entry found"),
    };
}

