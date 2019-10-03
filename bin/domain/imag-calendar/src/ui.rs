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

use clap::{Arg, App, SubCommand};

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
       .arg(Arg::with_name("calendar-ref-collection-name")
            .long("ref-collection")
            .takes_value(true)
            .required(false)
            .multiple(false)
            .default_value("calendars")
            .help("Name (Key) of the basepath setting in the configuration file to use"))

        .subcommand(SubCommand::with_name("import")
                   .about("Import directory of calendar files or files directl")
                   .version("0.1")
                   .arg(Arg::with_name("filesordirs")
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .multiple(true)
                        .value_name("PATH")
                        .validator(import_validator)
                        .help("Import files from this directory (or specify files directly)"))

                   .arg(Arg::with_name("import-fail")
                        .short("F")
                        .long("fail")
                        .takes_value(false)
                        .required(false)
                        .multiple(false)
                        .help("Fail if a file cannot be parsed (if directory is given, all files found must be icalendar files)"))

                   .arg(Arg::with_name("import-force-override")
                        .long("force")
                        .takes_value(false)
                        .required(false)
                        .multiple(false)
                        .help("Override if entry for event already exists"))
                   )

        .subcommand(SubCommand::with_name("list")
                   .about("List calendar entries")
                   .version("0.1")
                   .arg(Arg::with_name("format")
                        .long("format")
                        .short("F")
                        .takes_value(true)
                        .required(false)
                        .multiple(false)
                        .help("Override the format used to list one event"))

                   .arg(Arg::with_name("list-past")
                        .long("past")
                        .takes_value(false)
                        .required(false)
                        .multiple(false)
                        .help("List past events"))
                   )
}

fn import_validator<A: AsRef<str>>(s: A) -> Result<(), String> {
    use libimagutil::cli_validators::*;

    is_existing_path(s.as_ref())?;

    match (is_file(s.as_ref()), is_directory(s.as_ref())) {
        (Err(_), Err(_)) => Err(format!("Not a file or directory: {}", s.as_ref())),
        _                => Ok(())
    }
}

