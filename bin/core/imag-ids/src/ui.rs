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

use clap::{Arg, ArgMatches, App, SubCommand};
use failure::Fallible as Result;

use libimagstore::storeid::StoreId;
use libimagrt::runtime::IdPathProvider;

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
        .arg(Arg::with_name("print-storepath")
             .long("with-storepath")
             .takes_value(false)
             .required(false)
             .multiple(false)
             .help("Print the storepath for each id"))

        .arg(Arg::with_name("in-collection-filter")
             .long("in-collection")
             .short("c")
             .required(false)
             .takes_value(true)
             .multiple(true)
             .value_names(&["COLLECTION"])
             .help("Filter for ids which are only in these collections"))

        .subcommand(SubCommand::with_name("where")
                    .arg(Arg::with_name("where-filter")
                         .index(1)
                         .required(true)
                         .takes_value(true)
                         .multiple(false)
                         .value_names(&["QUERY"])
                         .help("Query the header of the entries and filter them"))
                   )
        .after_help(include_str!("../static/language-doc.md"))
}

pub struct PathProvider;
impl IdPathProvider for PathProvider {
    fn get_ids(_matches: &ArgMatches) -> Result<Option<Vec<StoreId>>> {
        Err(format_err!("imag-ids does not get IDs via CLI, only via stdin if applying a filter!"))
    }
}
