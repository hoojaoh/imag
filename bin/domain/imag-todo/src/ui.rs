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
        .subcommand(SubCommand::with_name("create")
                    .about("Create task")
                    .version("0.1")

                    .arg(Arg::with_name("create-scheduled")
                         .long("scheduled")
                         .short("s")
                         .takes_value(true)
                         .required(false)
                         .help("Set a 'scheduled' date/time")
                        )

                    .arg(Arg::with_name("create-hidden")
                         .long("hidden")
                         .short("h")
                         .takes_value(true)
                         .required(false)
                         .help("Set a 'hidden' date/time")
                        )

                    .arg(Arg::with_name("create-due")
                         .long("due")
                         .short("d")
                         .takes_value(true)
                         .required(false)
                         .help("Set a 'due' date/time")
                        )

                    .arg(Arg::with_name("create-prio")
                         .long("prio")
                         .short("p")
                         .takes_value(true)
                         .required(false)
                         .help("Set a priority")
                         .possible_values(&["h", "m", "l"])
                        )

                    .arg(Arg::with_name("create-status")
                         .long("status")
                         .takes_value(true)
                         .required(false)
                         .help("Set a status, useful if the task is already done")
                         .default_value("pending")
                         .possible_values(&["pending", "done", "deleted"])
                        )

                    .arg(Arg::with_name("create-edit")
                         .long("edit")
                         .short("e")
                         .takes_value(false)
                         .required(false)
                         .help("Create and then edit the entry")
                        )

                    .arg(Arg::with_name("text")
                         .index(1)
                         .multiple(true)
                         .required(true)
                         .help("Text for the todo")
                        )
                    )

        .subcommand(SubCommand::with_name("pending")
                    .arg(Arg::with_name("todos")
                         .index(1)
                         .takes_value(true)
                         .required(false)
                         .help("List pending todos (same as 'list' command without arguments)")
                        )
                    )

        .subcommand(SubCommand::with_name("list")
                    .about("List tasks (default)")
                    .version("0.1")

                    .arg(Arg::with_name("list-table")
                         .long("table")
                         .short("T")
                         .takes_value(false)
                         .required(false)
                         .help("Print a nice ascii-table")
                        )

                    .arg(Arg::with_name("list-hidden")
                         .long("hidden")
                         .short("H")
                         .takes_value(false)
                         .required(false)
                         .help("Print also hidden todos")
                        )

                    .arg(Arg::with_name("list-done")
                         .long("done")
                         .short("D")
                         .takes_value(false)
                         .required(false)
                         .help("Print also done todos")
                        )

                    .arg(Arg::with_name("list-nopending")
                         .long("no-pending")
                         .short("P")
                         .takes_value(false)
                         .required(false)
                         .help("Do not print pending tasks")
                        )

                    )

        .subcommand(SubCommand::with_name("show")
                    .arg(Arg::with_name("todos")
                         .index(1)
                         .takes_value(true)
                         .required(false)
                         .help("Show the passed todos")
                        )
                    )

        .subcommand(SubCommand::with_name("mark")
                    .about("Mark tasks as pending, done or deleted")
                    .version("0.1")

                    .subcommand(SubCommand::with_name("pending")
                                .arg(Arg::with_name("todos")
                                     .index(1)
                                     .takes_value(true)
                                     .required(false)
                                     .help("List pending todos (same as 'list' command without arguments)")
                                    )
                                )

                    .subcommand(SubCommand::with_name("done")
                                .arg(Arg::with_name("todos")
                                     .index(1)
                                     .takes_value(true)
                                     .required(false)
                                     .help("Mark the passed todos as done")
                                    )
                                )

                    .subcommand(SubCommand::with_name("deleted")
                                .arg(Arg::with_name("todos")
                                     .index(1)
                                     .takes_value(true)
                                     .required(false)
                                     .help("Mark the passed todos as deleted")
                                    )
                                )
        )


        .subcommand(SubCommand::with_name("import")
                    .about("Import todos from other tool")
                    .version("0.1")
                   .subcommand(SubCommand::with_name("taskwarrior")
                               .about("Import from taskwarrior by piping 'task export' to this subcommand.")
                               .version("0.1")
                              )
                   )

}

pub struct PathProvider;
impl IdPathProvider for PathProvider {
    fn get_ids(matches: &ArgMatches) -> Result<Option<Vec<StoreId>>> {
        match matches.subcommand() {
            ("show", Some(scmd)) => scmd.values_of("todos"),
            ("show", None)       => unimplemented!(),
            _                    => unimplemented!()
        }
        .map(|v| v
             .into_iter()
             .map(PathBuf::from)
             .map(|pb| pb.into_storeid())
             .collect::<Result<Vec<_>>>()
        )
        .transpose()
    }
}

