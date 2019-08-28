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
use failure::Fallible as Result;

use libimagstore::storeid::StoreId;
use libimagrt::runtime::IdPathProvider;
use libimagstore::storeid::IntoStoreId;

use clap::{Arg, ArgMatches, App, SubCommand};

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
        .subcommand(SubCommand::with_name("import-mail")
                    .about("Import a mail (create a reference to it) (Maildir)")
                    .version("0.1")
                    .arg(Arg::with_name("ignore-existing-ids")
                         .long("ignore-existing")
                         .short("I")
                         .takes_value(false)
                         .required(false)
                         .help("Ignore errors that might occur when store entries exist already"))

                    .arg(Arg::with_name("path")
                         .index(1)
                         .takes_value(true)
                         .multiple(true)
                         .required(true)
                         .help("Path to the mail file(s) to import")
                         .value_name("PATH"))
                    )

        .subcommand(SubCommand::with_name("list")
                    .about("List all stored references to mails")
                    .version("0.1")

                    .arg(Arg::with_name("list-read")
                         .long("read")
                         .short("r")
                         .takes_value(false)
                         .required(false)
                         .multiple(false)
                         .help("Print the textual content of the mail itself as well"))

                    .arg(Arg::with_name("list-id")
                         .index(1)
                         .takes_value(true)
                         .required(false)
                         .multiple(true)
                         .help("The ids of the mails to list information for"))

                    )

        .subcommand(SubCommand::with_name("mail-store")
                    .about("Operations on (subsets of) all mails")
                    .version("0.1")
                    .subcommand(SubCommand::with_name("update-refs")
                                .about("Create references based on Message-IDs for all loaded mails")
                                .version("0.1"))
                    // TODO: We really should be able to filter here.
                    )
}

pub struct PathProvider;
impl IdPathProvider for PathProvider {
    fn get_ids(matches: &ArgMatches) -> Result<Option<Vec<StoreId>>> {
        matches.values_of("list-id")
            .map(|v| v
                 .map(PathBuf::from)
                 .map(|pb| pb.into_storeid())
                 .collect::<Result<Vec<_>>>()
            )
            .unwrap_or_else(|| Ok(Vec::new()))
            .map(Some)
    }
}

