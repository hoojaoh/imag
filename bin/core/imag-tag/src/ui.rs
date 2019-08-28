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

use clap::{Arg, ArgMatches, ArgGroup, App, SubCommand};
use failure::Fallible as Result;

use libimagstore::storeid::StoreId;
use libimagstore::storeid::IntoStoreId;
use libimagrt::runtime::IdPathProvider;
use libimagentrytag::tag::is_tag;

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app.arg(Arg::with_name("id")
                .index(1)
                .takes_value(true)
                .required(false)
                .multiple(true)
                .value_name("ID")
                .help("Entry to use"))

        .subcommand(SubCommand::with_name("add")
                   .about("Add tags")
                   .version("0.1")
                   .arg(Arg::with_name("add-tags")
                           .index(1)
                           .takes_value(true)
                           .required(true)
                           .multiple(true)
                           .value_name("tags")
                           .validator(is_tag)
                           .help("Add these tags"))
                   )

        .subcommand(SubCommand::with_name("remove")
                   .about("Remove tags")
                   .version("0.1")
                   .arg(Arg::with_name("remove-tags")
                           .index(1)
                           .takes_value(true)
                           .required(true)
                           .multiple(true)
                           .value_name("tags")
                           .validator(is_tag)
                           .help("Remove these tags"))
                   )

       .subcommand(SubCommand::with_name("list")
                   .about("List tags (default)")
                   .version("0.1")
                   .arg(Arg::with_name("json")
                        .long("json")
                        .short("j")
                        .takes_value(false)
                        .required(false)
                        .help("List as JSON"))
                   .arg(Arg::with_name("linewise")
                        .long("linewise")
                        .short("l")
                        .takes_value(false)
                        .required(false)
                        .help("One tag per line"))
                   .arg(Arg::with_name("commasep")
                        .long("comma")
                        .short("c")
                        .takes_value(false)
                        .required(false)
                        .help("Commaseperated (default)"))
                   .arg(Arg::with_name("sep")
                        .long("sep")
                        .short("s")
                        .takes_value(true)
                        .required(false)
                        .help("Separated by string")
                        .value_name("SEP"))

                   .group(ArgGroup::with_name("list-group")
                          .args(&[
                                "json",
                                "linewise",
                                "commasep",
                                "sep",
                          ])
                          .required(true))
                   )

}

pub struct PathProvider;
impl IdPathProvider for PathProvider {
    fn get_ids(matches: &ArgMatches) -> Result<Option<Vec<StoreId>>> {
        matches.values_of("id")
            .map(|v| v
                 .map(PathBuf::from)
                 .map(|pb| pb.into_storeid())
                 .collect::<Result<Vec<_>>>()
            )
            .transpose()
    }
}

