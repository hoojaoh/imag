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
use std::io::Write;

use failure::Fallible as Result;
use failure::Error;
use clap::ArgMatches;

use libimagstore::store::FileLockEntry;
use libimagstore::storeid::StoreId;
use libimagrt::runtime::Runtime;

pub fn retrieve(rt: &Runtime) -> Result<()> {
    let scmd  = rt.cli().subcommand_matches("retrieve").unwrap();
    let id    = scmd.value_of("id").unwrap();
    let path  = PathBuf::from(id);
    let path  = StoreId::new(path)?;
    debug!("path = {:?}", path);

    rt.store()
        .retrieve(path.clone())
        .and_then(|e| print_entry(rt, scmd, e))?;

    rt.report_touched(&path).map_err(Error::from)
}

pub fn print_entry(rt: &Runtime, scmd: &ArgMatches, e: FileLockEntry) -> Result<()> {
    if do_print_raw(scmd) {
        debug!("Printing raw content...");
        writeln!(rt.stdout(), "{}", e.to_str()?)?;
    } else if do_filter(scmd) {
        debug!("Filtering...");
        warn!("Filtering via header specs is currently now supported.");
        warn!("Will fail now!");
        unimplemented!()
    } else {
        debug!("Printing structured...");
        if do_print_header(scmd) {
            debug!("Printing header...");
            if do_print_header_as_json(rt.cli()) {
                debug!("Printing header as json...");
                warn!("Printing as JSON currently not supported.");
                warn!("Will fail now!");
                unimplemented!()
            } else {
                debug!("Printing header as TOML...");
                writeln!(rt.stdout(), "{}", e.get_header())?;
            }
        }

        if do_print_content(scmd) {
            debug!("Printing content...");
            writeln!(rt.stdout(), "{}", e.get_content())?;
        }
    }

    Ok(())
}

fn do_print_header(m: &ArgMatches) -> bool {
    m.is_present("header")
}

fn do_print_header_as_json(m: &ArgMatches) -> bool {
    m.is_present("header-json")
}

fn do_print_content(m: &ArgMatches) -> bool {
    m.is_present("content")
}

fn do_print_raw(m: &ArgMatches) -> bool {
    m.is_present("raw")
}

fn do_filter(m: &ArgMatches) -> bool {
    m.subcommand_matches("filter-header").is_some()
}

