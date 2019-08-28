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

use clap::{Arg, ArgMatches, App, SubCommand};
use failure::Fallible as Result;

use libimagstore::storeid::IntoStoreId;
use libimagstore::storeid::StoreId;
use libimagrt::runtime::IdPathProvider;

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
        .subcommand(SubCommand::with_name("add")
                    .about("Add GPS coordinates to an entry")
                    .version("0.1")
                    .arg(Arg::with_name("longitude")
                         .long("long")
                         .takes_value(true)
                         .required(true)
                         .multiple(false)
                         .help("Set the longitude value. Format: <degrees>.<minutes>.<seconds>")
                         .value_name("LONGITUDE"))
                    .arg(Arg::with_name("latitude")
                         .long("lat")
                         .takes_value(true)
                         .required(true)
                         .multiple(false)
                         .help("Set the latitude. Format: <degrees>.<minutes>.<seconds>")
                         .value_name("LATITUDE"))
                    .arg(Arg::with_name("entry")
                         .index(1)
                         .takes_value(true)
                         .required(true)
                         .multiple(true)
                         .help("The entry to add the latitude/longitude to")
                         .value_name("ENTRY"))
                    )

        .subcommand(SubCommand::with_name("remove")
                .about("Remove a GPS coordinate pair from an entry")
                .version("0.1")
                .arg(Arg::with_name("print-removed")
                     .long("print-removed")
                     .short("p")
                     .takes_value(false)
                     .required(false)
                     .help("Print the removed values after removing them"))
                .arg(Arg::with_name("entry")
                     .index(1)
                     .takes_value(true)
                     .required(true)
                     .multiple(true)
                     .help("The entry to remove the latitude/longitude from")
                     .value_name("ENTRY"))
                )

        .subcommand(SubCommand::with_name("get")
                .about("Get a GPS coordinate pair from an entry")
                .version("0.1")
                .arg(Arg::with_name("entry")
                     .index(1)
                     .takes_value(true)
                     .required(true)
                     .multiple(true)
                     .help("The entry to get the latitude/longitude from")
                     .value_name("ENTRY"))
                .arg(Arg::with_name("format-json")
                     .long("json")
                     .takes_value(false)
                     .required(false)
                     .multiple(false)
                     .help("Get as JSON Object"))
                .arg(Arg::with_name("format-print")
                     .long("print")
                     .takes_value(false)
                     .required(false)
                     .multiple(false)
                     .help("Print as <key>=<value> pairs (2 lines, default)"))
                )
}

pub struct PathProvider;
impl IdPathProvider for PathProvider {
    fn get_ids(matches: &ArgMatches) -> Result<Option<Vec<StoreId>>> {
        fn get_id_paths(field: &str, subm: &ArgMatches) -> Result<Option<Vec<StoreId>>> {
            subm.values_of(field)
                .map(|v| v
                     .map(PathBuf::from)
                     .map(|pb| pb.into_storeid())
                     .collect::<Result<Vec<_>>>()
                )
                .transpose()
        }

        match matches.subcommand() {
            ("add", Some(subm)) => get_id_paths("entry", subm),
            ("remove", Some(subm)) => get_id_paths("entry", subm),
            ("get", Some(subm)) => get_id_paths("get-ids", subm),
            (other, _) => Err(format_err!("Not a known command: {}", other)),
        }
    }
}
