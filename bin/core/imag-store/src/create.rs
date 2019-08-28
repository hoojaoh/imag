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
use std::io::stdin;
use std::fs::OpenOptions;
use std::io::Read;
use std::ops::DerefMut;

use clap::ArgMatches;
use toml::Value;
use failure::Fallible as Result;
use failure::err_msg;

use libimagrt::runtime::Runtime;
use libimagstore::store::Entry;
use libimagstore::storeid::StoreId;
use libimagerror::trace::MapErrTrace;
use libimagerror::exit::ExitUnwrap;
use libimagutil::debug_result::*;

use crate::util::build_toml_header;

pub fn create(rt: &Runtime) {
    let scmd = rt.cli().subcommand_matches("create").unwrap();
    debug!("Found 'create' subcommand...");

    // unwrap is safe as value is required
    let path  = scmd.value_of("path").unwrap();
    let path  = PathBuf::from(path);
    let path  = StoreId::new(path).map_err_trace_exit_unwrap();

    debug!("path = {:?}", path);

    if scmd.subcommand_matches("entry").is_some() {
        debug!("Creating entry from CLI specification");
        create_from_cli_spec(rt, scmd, &path)
            .or_else(|_| create_from_source(rt, scmd, &path))
            .or_else(|_| create_with_content_and_header(rt,
                                                        &path,
                                                        String::new(),
                                                        Entry::default_header()))
    } else {
        debug!("Creating entry");
        create_with_content_and_header(rt, &path, String::new(),
            Entry::default_header())
    }
    .map_err_trace_exit_unwrap();

    rt.report_touched(&path).unwrap_or_exit();
}

fn create_from_cli_spec(rt: &Runtime, matches: &ArgMatches, path: &StoreId) -> Result<()> {
    let content = matches.subcommand_matches("entry")
        .map_or_else(|| {
            debug!("Didn't find entry subcommand, getting raw content");
            matches.value_of("from-raw")
                .map_or_else(String::new, string_from_raw_src)
        }, |entry_subcommand| {
            debug!("Found entry subcommand, parsing content");
            entry_subcommand
                .value_of("content")
                .map_or_else(|| {
                    entry_subcommand.value_of("content-from")
                        .map_or_else(String::new, string_from_raw_src)
                }, String::from)
        });
    debug!("Got content with len = {}", content.len());

    let header = matches.subcommand_matches("entry")
        .map_or_else(Entry::default_header,
            |entry_matches| build_toml_header(entry_matches, Entry::default_header()));

    create_with_content_and_header(rt, path, content, header)
}

fn create_from_source(rt: &Runtime, matches: &ArgMatches, path: &StoreId) -> Result<()> {
    let content = matches
        .value_of("from-raw")
        .ok_or_else(|| err_msg("No Commandline call"))
        .map(string_from_raw_src)?;

    debug!("Content with len = {}", content.len());

    Entry::from_str(path.clone(), &content[..])
        .map_dbg_err(|e| format!("Error building entry: {:?}", e))
        .and_then(|new_e| {
            let r = rt.store()
                .create(path.clone())
                .map_dbg_err(|e| format!("Error in Store::create(): {:?}", e))
                .map(|mut old_e| {
                    *old_e.deref_mut() = new_e;
                });

            debug!("Entry build");
            r
        })
        .map_dbg_err(|e| format!("Error storing entry: {:?}", e))
}

fn create_with_content_and_header(rt: &Runtime,
                                  path: &StoreId,
                                  content: String,
                                  header: Value) -> Result<()>
{
    debug!("Creating entry with content at {:?}", path);
    rt.store()
        .create(path.clone())
        .map_dbg_err(|e| format!("Error in Store::create(): {:?}", e))
        .map(|mut element| {
            {
                let e_content = element.get_content_mut();
                *e_content = content;
                debug!("New content set");
            }
            {
                let e_header  = element.get_header_mut();
                *e_header = header;
                debug!("New header set");
            }
        })
}

fn string_from_raw_src(raw_src: &str) -> String {
    let mut content = String::new();
    if raw_src == "-" {
        debug!("Reading entry from stdin");
        let res = stdin().read_to_string(&mut content);
        debug!("Read {:?} bytes", res);
    } else {
        debug!("Reading entry from file at {:?}", raw_src);
        let _ = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(raw_src)
            .and_then(|mut f| f.read_to_string(&mut content));
    }
    content
}

#[cfg(test)]
mod tests {
    use super::create;

    use std::path::PathBuf;
    use toml_query::read::TomlValueReadExt;
    use toml::Value;

    make_mock_app! {
        app "imag-store";
        modulename mock;
        version env!("CARGO_PKG_VERSION");
        with help "imag-store mocking app";
    }
    use self::mock::generate_test_runtime;

    #[test]
    fn test_create_simple() {
        let test_name = "test_create_simple";
        let rt = generate_test_runtime(vec!["create", "test_create_simple"]).unwrap();

        create(&rt);

        let e = rt.store().get(PathBuf::from(test_name));
        assert!(e.is_ok());
        let e = e.unwrap();
        assert!(e.is_some());
        let e = e.unwrap();

        let version = e.get_header().read("imag.version").map(Option::unwrap).unwrap();
        assert_eq!(Value::String(String::from(env!("CARGO_PKG_VERSION"))), *version);
    }

}

