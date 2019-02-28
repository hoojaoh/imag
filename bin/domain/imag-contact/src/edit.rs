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

#![deny(
    non_camel_case_types,
    non_snake_case,
    path_statements,
    trivial_numeric_casts,
    unstable_features,
    unused_allocation,
    unused_import_braces,
    unused_imports,
    unused_must_use,
    unused_mut,
    unused_qualifications,
    while_true,
)]

use std::process::exit;
use std::io::Read;
use std::io::Write;

use failure::Error;
use failure::err_msg;
use failure::Fallible as Result;

use libimagrt::runtime::Runtime;
use libimagerror::trace::MapErrTrace;
use libimagstore::store::FileLockEntry;
use libimagcontact::store::ContactStore;
use libimagentryref::reference::fassade::RefFassade;
use libimagentryref::hasher::default::DefaultHasher;
use libimagentryref::reference::Ref;
use libimagentryref::reference::Config as RefConfig;

pub fn edit(rt: &Runtime) {
    let scmd            = rt.cli().subcommand_matches("edit").unwrap();
    let collection_name = rt.cli().value_of("contact-ref-collection-name").unwrap(); // default by clap
    let ref_config      = libimagentryref::util::get_ref_config(&rt, "imag-contact").map_err_trace_exit_unwrap();
    let hash            = scmd.value_of("hash").map(String::from).unwrap(); // safed by clap
    let force_override  = true; // when editing, we want to override, right?
    let retry           = !scmd.is_present("fail-on-parse-error");

    if rt.output_is_pipe() {
        error!("Cannot spawn editor if output is a pipe!");
        exit(1);
    }

    let mut output = rt.stdout();
    let mut input  = rt.stdin().unwrap_or_else(|| {
        error!("No input stream. Cannot ask for permission.");
        exit(1)
    });

    crate::util::find_contact_by_hash(rt, hash)
        .for_each(|contact| {
            loop {
                let res = edit_contact(&rt, &contact, &ref_config, collection_name, force_override);
                if !retry {
                    let _ = res.map_err_trace_exit_unwrap();
                } else {
                    if ask_continue(&mut input, &mut output) {
                        continue;
                    } else {
                        exit(1)
                    }
                }
            }
        });
}

fn edit_contact<'a>(rt: &Runtime, contact: &FileLockEntry<'a>, ref_config: &RefConfig, collection_name: &str, force_override: bool) -> Result<()> {
    let filepath = contact
        .as_ref_with_hasher::<DefaultHasher>()
        .get_path(ref_config)
        .map_err_trace_exit_unwrap();

    let success = rt.editor()
        .map_err_trace_exit_unwrap()
        .ok_or_else(|| {
            err_msg("I have no editor configured. Cannot continue!")
        })
        .map_err_trace_exit_unwrap()
        .arg(&filepath)
        .status()
        .map_err(Error::from)
        .map_err_trace_exit_unwrap()
        .success();

    if !success {
        error!("Editor failed!");
        exit(1);
    }

    rt.store()
        .retrieve_from_path(&filepath, &ref_config, &collection_name, force_override)
        .map(|_| ())
}

fn ask_continue(inputstream: &mut Read, outputstream: &mut Write) -> bool {
    ::libimaginteraction::ask::ask_bool("Edit vcard", Some(true), inputstream, outputstream)
        .map_err_trace_exit_unwrap()
}

