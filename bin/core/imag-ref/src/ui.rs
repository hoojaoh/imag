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

use clap::{Arg, App, ArgMatches, SubCommand};
use failure::Fallible as Result;

use libimagstore::storeid::StoreId;
use libimagstore::storeid::IntoStoreId;
use libimagrt::runtime::IdPathProvider;

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
        .subcommand(SubCommand::with_name("deref")
                    .about("'Dereference a ref. This prints the Path(es) of the referenced file(s)")
                    .version("0.1")
                    .arg(Arg::with_name("ID")
                         .index(1)
                         .takes_value(true)
                         .required(false)
                         .multiple(true)
                         .help("The id of the store entry to dereference.")
                         .value_name("ID"))

                    .arg(Arg::with_name("ignore-noref")
                         .long("ignore-noref")
                         .takes_value(false)
                         .required(false)
                         .multiple(false)
                         .help("Ignore store entries which are not refs and do not print error message"))

                    .arg(Arg::with_name("override-basepath")
                         .long("basepath-setting")
                         .short("B")
                         .takes_value(true)
                         .required(false)
                         .multiple(false)
                         .help("Override the basepath key to look up in the configuration"))
                    )

        .subcommand(SubCommand::with_name("remove")
                .about("Remove a reference from an entry")
                .version("0.1")
                .arg(Arg::with_name("ID")
                     .index(1)
                     .takes_value(true)
                     .required(false)
                     .multiple(true)
                     .help("Remove the reference from this store entry")
                     .value_name("ENTRIES"))

                .arg(Arg::with_name("ignore-noref")
                     .long("ignore-noref")
                     .takes_value(false)
                     .required(false)
                     .multiple(false)
                     .help("Ignore store entries which are not refs and do not print error message"))
                )

        .subcommand(SubCommand::with_name("create")
                .about("Create a reference to a file")
                .version("0.1")
                .arg(Arg::with_name("ID")
                     .index(1)
                     .takes_value(true)
                     .required(true)
                     .multiple(false)
                     .help("Create a reference with that ID in the store. If the store id exists, it will be made into a reference.")
                     .value_name("ID"))

                .arg(Arg::with_name("path")
                     .index(2)
                     .takes_value(true)
                     .required(true)
                     .multiple(false)
                     .help("The path to refer to. If there is no basepath configuration in the config file for the path this file is located at, the operation will error.")
                     .value_name("ID"))

                .arg(Arg::with_name("force")
                     .long("force")
                     .takes_value(false)
                     .required(false)
                     .multiple(false)
                     .help("Use force to override existing references"))
                )

        .subcommand(SubCommand::with_name("list-dead")
                .about("List all dead references")
                .version("0.1")
                .arg(Arg::with_name("ID")
                     .index(1)
                     .takes_value(true)
                     .required(false)
                     .multiple(true)
                     .help("Filter these IDs for dead ones")
                     .value_name("ID"))

                .arg(Arg::with_name("list-dead-pathes")
                     .long("pathes")
                     .takes_value(false)
                     .required(false)
                     .multiple(false)
                     .conflicts_with("list-dead-ids")
                     .help("List pathes which do not exist anymore but are referenced from imag entries"))

                .arg(Arg::with_name("list-dead-ids")
                     .long("ids")
                     .takes_value(false)
                     .required(false)
                     .multiple(false)
                     .conflicts_with("list-dead-pathes")
                     .help("List ids of entries which refer to a path that does not exist"))
                )

}

pub struct PathProvider;
impl IdPathProvider for PathProvider {
    fn get_ids(matches: &ArgMatches) -> Result<Option<Vec<StoreId>>> {
        fn get_id_paths(subm: &ArgMatches) -> Result<Option<Vec<StoreId>>> {
            subm.values_of("ID")
                .map(|v| v
                     .into_iter()
                     .map(PathBuf::from)
                     .map(|pb| pb.into_storeid())
                     .collect::<Result<Vec<_>>>()
                )
                .transpose()
        }

        match matches.subcommand() {
            ("deref", Some(subm)) => get_id_paths(subm),
            ("remove", Some(subm)) => get_id_paths(subm),
            ("list-dead", Some(subm)) => get_id_paths(subm),
            ("create", _) => Err(format_err!("Command does not get IDs as input")),
            (other, _) => Err(format_err!("Not a known command: {}", other)),
        }
    }
}
