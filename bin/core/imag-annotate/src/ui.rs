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

use libimagstore::storeid::StoreId;
use libimagstore::storeid::IntoStoreId;
use libimagrt::runtime::IdPathProvider;

use failure::Fallible as Result;

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
        .subcommand(SubCommand::with_name("add")
                    .about("Add annotation to an entry")
                    .version("0.1")

                    .arg(Arg::with_name("dont-print-name")
                         .short("N")
                         .long("no-name")
                         .takes_value(false)
                         .required(false)
                         .multiple(false)
                         .help("Do not print the name of the annotation after annotating.")
                         )
                    .arg(Arg::with_name("entry")
                         .index(1)
                         .takes_value(true)
                         .required(true)
                         .multiple(false)
                         .help("The entry to add the latitude/longitude to")
                         .value_name("ENTRY"))
                   )

        .subcommand(SubCommand::with_name("remove")
                    .about("Remove annotation from an entry")
                    .version("0.1")
                    .arg(Arg::with_name("entry")
                         .index(1)
                         .takes_value(true)
                         .required(true)
                         .multiple(false)
                         .help("The entry to remove the latitude/longitude from")
                         .value_name("ENTRY"))
                    .arg(Arg::with_name("annotation_name")
                         .index(2)
                         .takes_value(true)
                         .required(true)
                         .multiple(false)
                         .help("Name of the annotation to remove")
                         .value_name("NAME"))
                    .arg(Arg::with_name("delete-annotation")
                         .short("D")
                         .long("delete")
                         .takes_value(false)
                         .required(false)
                         .multiple(false)
                         .help("Do not only 'unlink' the annotation, but also delete it from the store"))
                   )

        .subcommand(SubCommand::with_name("list")
                    .about("List annotations")
                    .version("0.1")
                    .arg(Arg::with_name("entry")
                         .index(1)
                         .takes_value(true)
                         .required(false)
                         .multiple(false)
                         .help("The entry to list annotations for (all annotations if not passed)")
                         .value_name("ENTRY"))
                    .arg(Arg::with_name("list-with-text")
                         .long("text")
                         .short("t")
                         .takes_value(false)
                         .required(false)
                         .multiple(false)
                         .help("List annotations with text"))
                   )
}

pub struct PathProvider;
impl IdPathProvider for PathProvider {
    fn get_ids(matches: &ArgMatches) -> Result<Option<Vec<StoreId>>> {
        fn get_id_paths(subm: &ArgMatches) -> Result<Option<Vec<StoreId>>> {
            subm.values_of("entry")
                .map(|v| v
                     .into_iter()
                     .map(PathBuf::from)
                     .map(|pb| pb.into_storeid())
                     .collect::<Result<Vec<_>>>()
                )
                .transpose()
        }

        match matches.subcommand() {
            ("add", Some(subm)) => get_id_paths(subm),
            ("remove", Some(subm)) => get_id_paths(subm),
            ("list", Some(subm)) => get_id_paths(subm),
            (other, _) => {
                Err(format_err!("Not a known command: {}", other))
            }
        }
    }
}
