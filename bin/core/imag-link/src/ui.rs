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

use libimagstore::storeid::StoreId;
use libimagstore::storeid::IntoStoreId;
use libimagrt::runtime::IdPathProvider;

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
        .subcommand(SubCommand::with_name("remove")
                .about("Remove a link between two or more entries")
                .version("0.1")
                .arg(Arg::with_name("from")
                     .index(1)
                     .takes_value(true)
                     .required(true)
                     .multiple(false)
                     .help("Remove Link from this entry")
                     .value_name("ENTRY"))
                .arg(Arg::with_name("to")
                     .index(2)
                     .takes_value(true)
                     .required(true)
                     .multiple(true)
                     .help("Remove links to these entries")
                     .value_name("ENTRIES"))
                )
        .subcommand(SubCommand::with_name("unlink")
                .about("Remove all links from an entry")
                .version("0.1")
                .arg(Arg::with_name("from")
                     .index(1)
                     .takes_value(true)
                     .required(true)
                     .multiple(true)
                     .help("Remove links from these entries")
                     .value_name("ENTRY"))
                )

        .subcommand(SubCommand::with_name("list")
                .about("List links to this entry")
                .version("0.1")
                .arg(Arg::with_name("entries")
                     .index(1)
                     .takes_value(true)
                     .multiple(true)
                     .required(true)
                     .help("List these entries, seperate by comma")
                     .value_name("ENTRIES"))

                .arg(Arg::with_name("list-externals-too")
                     .long("list-external")
                     .takes_value(false)
                     .required(false)
                     .help("Also list external links (debugging helper that might be removed at some point"))

                .arg(Arg::with_name("list-plain")
                     .long("plain")
                     .multiple(false)
                     .takes_value(false)
                     .required(false)
                     .help("List plain rather than in ASCII table"))
                )

        .arg(Arg::with_name("check-consistency")
             .long("check-consistency")
             .short("C")
             .takes_value(false)
             .required(false)
             .help("Check the link-consistency in the store (might be time-consuming)"))

        .arg(Arg::with_name("from")
             .index(1)
             .takes_value(true)
             .required(false)
             .multiple(false)
             .help("Link from this entry")
             .requires("to")
             .value_name("ENTRY"))

        .arg(Arg::with_name("to")
             .index(2)
             .takes_value(true)
             .required(false)
             .multiple(true)
             .help("Link to this entries")
             .requires("from")
             .value_name("ENTRIES"))

        .arg(Arg::with_name("directional")
             .long("direction")
             .takes_value(false)
             .required(false)
             .multiple(false)
             .help("When creating links, make them directional")
             .requires_all(&["from", "to"]))

}

/// PathProvider
///
/// This PathProvider does _not_ return the "from" value of the commandline call if no subcommand
/// is given.
///
/// It has to be fetched by main() by hand.
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
        let ids = match matches.subcommand() {
            ("remove", Some(subm)) => Some(get_id_paths("to", subm)),
            ("unlink", Some(subm)) => Some(get_id_paths("from", subm)),
            ("list", Some(subm)) => Some(get_id_paths("entries", subm)),
            _ => None,
        };

        ids
            .unwrap_or_else(|| get_id_paths("to", matches))
    }
}
