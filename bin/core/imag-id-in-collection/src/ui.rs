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

use clap::{Arg, ArgMatches, App};
use failure::Fallible as Result;

use libimagstore::storeid::StoreId;
use libimagrt::runtime::IdPathProvider;

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
        .arg(Arg::with_name("in-collection-filter")
             .index(1)
             .required(true)
             .takes_value(true)
             .multiple(false)
             .value_name("COLLECTION")
             .help("Filter for ids which are in this collection"))

        .arg(Arg::with_name("ids")
             .index(2)
             .required(false)
             .takes_value(true)
             .multiple(true)
             .value_names(&["IDs"])
             .help("Ids to filter"))
}

pub struct PathProvider;
impl IdPathProvider for PathProvider {
    fn get_ids(matches: &ArgMatches) -> Result<Option<Vec<StoreId>>> {
        if let Some(ids) = matches.values_of("ids") {
            ids.map(|i| StoreId::new(PathBuf::from(i)))
                .collect::<Result<Vec<StoreId>>>()
                .map(Some)
        } else {
            Ok(None)
        }
    }
}
