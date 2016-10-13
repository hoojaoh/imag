//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015, 2016 Matthias Beyer <mail@beyermatthias.de> and contributors
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

use clap::{Arg, App, SubCommand};

use libimagentrytag::ui::tag_add_arg;
use libimagutil::cli_validators::*;

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
        .subcommand(SubCommand::with_name("add")
                   .about("Add annotation")
                   .version("0.1")
                   .arg(Arg::with_name("entry")
                        .long("entry")
                        .short("e")
                        .takes_value(true)
                        .required(true)
                        .multiple(true)
                        .value_name("ENTRY")
                        .help("Add annotation to this/these entry/entries"))
                   .arg(Arg::with_name("annotation")
                        .long("annotation")
                        .short("a")
                        .takes_value(true)
                        .required(false)
                        .multiple(false)
                        .value_name("TEXT")
                        .help("Add this annotation text. For passing the annotation text on the CLI."))
                   )

        .subcommand(SubCommand::with_name("remove")
                   .about("Remove annotation(s)")
                   .version("0.1")
                   .arg(Arg::with_name("entry")
                        .long("entry")
                        .short("e")
                        .takes_value(true)
                        .required(true)
                        .multiple(true)
                        .value_name("ENTRY")
                        .help("Remove annotations from this entry"))
                   .arg(Arg::with_name("annotation")
                        .long("annotation")
                        .short("a")
                        .takes_value(true)
                        .required(true)
                        .multiple(true)
                        .value_name("ID")
                        .help("Remove the annotation with this ID"))
                   .arg(Arg::with_name("no-gc")
                        .long("no-gc")
                        .short("G")
                        .takes_value(false)
                        .required(false)
                        .multiple(false)
                        .help("Do not remove the annotation object, even if it does not refer to any entry anymore"))
                   )

        .subcommand(SubCommand::with_name("list")
                   .about("List annotations for one or more entries")
                   .version("0.1")
                   .arg(Arg::with_name("entry")
                        .long("entry")
                        .short("e")
                        .takes_value(true)
                        .required(true)
                        .multiple(true)
                        .value_name("ENTRY")
                        .help("List annotations for this/these entry/entries"))
                   .arg(Arg::with_name("hashes")
                        .long("print-hashes")
                        .required(false)
                        .multiple(false)
                        .help("Show only the IDs of the annotations"))
                   )
}

