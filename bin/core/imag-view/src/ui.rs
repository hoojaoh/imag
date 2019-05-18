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
use libimagstore::storeid::IntoStoreId;
use libimagrt::runtime::IdPathProvider;

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
        .arg(Arg::with_name("id")
             .index(1)
             .takes_value(true)
             .required(false)
             .multiple(true)
             .help("View these entries at this store path")
             .value_name("IDs"))

        .arg(Arg::with_name("autowrap")
            .long("autowrap")
            .short("w")
            .takes_value(true)
            .required(false)
            .multiple(false)
            .value_name("WIDTH")
            .default_value("80")
            .validator(::libimagutil::cli_validators::is_integer)
            .help("Automatically wrap long lines. Has only an effect when using stdout as output."))

        .arg(Arg::with_name("view-header")
            .long("header")
            .short("h")
            .takes_value(false)
            .required(false)
            .help("View header"))
        .arg(Arg::with_name("not-view-content")
            .long("no-content")
            .short("C")
            .takes_value(false)
            .required(false)
            .help("Do not view content"))

        .arg(Arg::with_name("compile-md")
            .long("compile")
            .short("c")
            .takes_value(false)
            .required(false)
            .help("Do compile markdown to be nice")
            .conflicts_with("not-view-content")
            .conflicts_with("autowrap")) // markdown viewer does not support wrapping

        .arg(Arg::with_name("seperator")
            .long("seperate")
            .short("s")
            .required(false)
            .takes_value(true)
            .value_name("SEPCHR")
            .default_value("-")
            .help("Do seperate entries with a string if viewing multiple entries"))


        .arg(Arg::with_name("in")
            .long("in")
            .takes_value(true)
            .required(false)
            .multiple(false)
            .help("View content. If no value is given, this fails. Possible viewers are configured via the config file."))

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
