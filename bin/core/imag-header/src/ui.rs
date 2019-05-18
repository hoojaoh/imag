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
        .arg(Arg::with_name("id")
             .index(1)
             .takes_value(true)
             .required(false)
             .multiple(true)
             .help("The id of the entry/entries to edit")
             .value_name("ENTRY"))

        .arg(Arg::with_name("list-id")
             .long("list-id")
             .takes_value(false)
             .required(false)
             .multiple(false)
             .help("List Store Id in output (format: '<id> - <value>'"))
        .arg(Arg::with_name("list-id-format")
             .long("list-id-format")
             .takes_value(true)
             .required(false)
             .multiple(false)
             .help("List Store Id in output with format"))

        .subcommand(SubCommand::with_name("read")
                    .about("Read a header value by path")
                    .version("0.1")
                    .arg(Arg::with_name("header-value-path")
                         .index(1)
                         .takes_value(true)
                         .required(true)
                         .multiple(false)
                         .help("Path of the header value")
                         .value_name("PATH"))
                   )

        .subcommand(SubCommand::with_name("has")
                    .about("Check whether a header value is present")
                    .version("0.1")
                    .arg(Arg::with_name("header-value-path")
                         .index(1)
                         .takes_value(true)
                         .required(true)
                         .multiple(false)
                         .help("Path of the header value")
                         .value_name("PATH"))
                   )

        .subcommand(SubCommand::with_name("hasnt")
                    .about("Check whether a header value is not present")
                    .version("0.1")
                    .arg(Arg::with_name("header-value-path")
                         .index(1)
                         .takes_value(true)
                         .required(true)
                         .multiple(false)
                         .help("Path of the header value")
                         .value_name("PATH"))
                   )

        .subcommand(SubCommand::with_name("int")
                    .about("Check whether a header value is a number")
                    .version("0.1")
                    .arg(Arg::with_name("header-value-path")
                         .index(1)
                         .takes_value(true)
                         .required(true)
                         .multiple(false)
                         .help("Path of the header value")
                         .value_name("PATH"))

                    .arg(Arg::with_name("header-int-eq")
                         .long("eq")
                         .takes_value(true)
                         .required(false)
                         .help("Check whether the value is equal to VALUE")
                         .validator(::libimagutil::cli_validators::is_integer)
                         .value_name("VALUE"))
                    .arg(Arg::with_name("header-int-neq")
                         .long("neq")
                         .takes_value(true)
                         .required(false)
                         .help("Check whether the value is not equal to VALUE")
                         .validator(::libimagutil::cli_validators::is_integer)
                         .value_name("VALUE"))
                    .arg(Arg::with_name("header-int-lt")
                         .long("lt")
                         .takes_value(true)
                         .required(false)
                         .help("Check whether the value is lower than VALUE")
                         .validator(::libimagutil::cli_validators::is_integer)
                         .value_name("VALUE"))
                    .arg(Arg::with_name("header-int-gt")
                         .long("gt")
                         .takes_value(true)
                         .required(false)
                         .help("Check whether the value is greater than VALUE")
                         .validator(::libimagutil::cli_validators::is_integer)
                         .value_name("VALUE"))
                    .arg(Arg::with_name("header-int-lte")
                         .long("lte")
                         .takes_value(true)
                         .required(false)
                         .help("Check whether the value is lower than or equal VALUE")
                         .validator(::libimagutil::cli_validators::is_integer)
                         .value_name("VALUE"))
                    .arg(Arg::with_name("header-int-gte")
                         .long("gte")
                         .takes_value(true)
                         .required(false)
                         .help("Check whether the value is greater than or equal VALUE")
                         .validator(::libimagutil::cli_validators::is_integer)
                         .value_name("VALUE"))
                   )

        .subcommand(SubCommand::with_name("float")
                    .about("Check whether a header value is a floating number")
                    .version("0.1")
                    .arg(Arg::with_name("header-value-path")
                         .index(1)
                         .takes_value(true)
                         .required(true)
                         .multiple(false)
                         .help("Path of the header value")
                         .value_name("PATH"))

                    .arg(Arg::with_name("header-float-eq")
                         .long("eq")
                         .takes_value(true)
                         .required(false)
                         .help("Check whether the value is equal to VALUE")
                         .validator(::libimagutil::cli_validators::is_float)
                         .value_name("VALUE"))
                    .arg(Arg::with_name("header-float-neq")
                         .long("neq")
                         .takes_value(true)
                         .required(false)
                         .help("Check whether the value is not equal to VALUE")
                         .validator(::libimagutil::cli_validators::is_float)
                         .value_name("VALUE"))
                    .arg(Arg::with_name("header-float-lt")
                         .long("lt")
                         .takes_value(true)
                         .required(false)
                         .help("Check whether the value is lower than VALUE")
                         .validator(::libimagutil::cli_validators::is_float)
                         .value_name("VALUE"))
                    .arg(Arg::with_name("header-float-gt")
                         .long("gt")
                         .takes_value(true)
                         .required(false)
                         .help("Check whether the value is greater than VALUE")
                         .validator(::libimagutil::cli_validators::is_float)
                         .value_name("VALUE"))
                   )

        .subcommand(SubCommand::with_name("string")
                    .about("Check whether a header value is a string")
                    .version("0.1")
                    .arg(Arg::with_name("header-value-path")
                         .index(1)
                         .takes_value(true)
                         .required(true)
                         .multiple(false)
                         .help("Path of the header value")
                         .value_name("PATH"))

                    .arg(Arg::with_name("header-string-eq")
                         .long("eq")
                         .takes_value(true)
                         .required(false)
                         .help("Check whether the value is equal to VALUE")
                         .value_name("VALUE"))
                    .arg(Arg::with_name("header-string-neq")
                         .long("neq")
                         .takes_value(true)
                         .required(false)
                         .help("Check whether the value is not equal to VALUE")
                         .value_name("VALUE"))
                   )

        .subcommand(SubCommand::with_name("bool")
                    .about("Check whether a header value is a bool")
                    .version("0.1")
                    .arg(Arg::with_name("header-value-path")
                         .index(1)
                         .takes_value(true)
                         .required(true)
                         .multiple(false)
                         .help("Path of the header value")
                         .value_name("PATH"))

                    .arg(Arg::with_name("header-bool-set")
                         .short("t")
                         .long("set")
                         .takes_value(false)
                         .required(false)
                         .help("Check whether the flag is set (true)"))
                    .arg(Arg::with_name("header-bool-unset")
                         .short("f")
                         .long("unset")
                         .takes_value(false)
                         .required(false)
                         .help("Check whether the flag is unset (false)"))
                   )
}

pub struct PathProvider;
impl IdPathProvider for PathProvider {
    fn get_ids(matches: &ArgMatches) -> Result<Option<Vec<StoreId>>> {
        matches.values_of("id")
            .map(|v| v
                 .into_iter()
                 .map(PathBuf::from)
                 .map(|pb| pb.into_storeid())
                 .collect::<Result<Vec<_>>>()
            )
            .transpose()
    }
}
