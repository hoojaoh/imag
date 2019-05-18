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

#![forbid(unsafe_code)]

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

#[macro_use] extern crate log;
extern crate clap;
#[macro_use] extern crate failure;

extern crate libimagstore;
#[macro_use] extern crate libimagrt;
extern crate libimagentryref;
extern crate libimagerror;
extern crate libimaginteraction;
extern crate libimagutil;

mod ui;
use crate::ui::build_ui;

use std::process::exit;
use std::io::Write;

use failure::Error;

use libimagerror::trace::MapErrTrace;
use libimagerror::exit::ExitUnwrap;
use libimagrt::setup::generate_runtime_setup;
use libimagrt::runtime::Runtime;
use libimagentryref::reference::Ref;
use libimagentryref::reference::MutRef;
use libimagentryref::reference::RefFassade;
use libimagentryref::hasher::default::DefaultHasher;
use libimagentryref::util::get_ref_config;

fn main() {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-ref",
                                    &version,
                                    "Reference files outside of the store",
                                    build_ui);
    rt.cli()
        .subcommand_name()
        .map(|name| {
            debug!("Call: {}", name);
            match name {
                "deref"     => deref(&rt),
                "create"    => create(&rt),
                "remove"    => remove(&rt),
                "list-dead" => list_dead(&rt),
                other => {
                    debug!("Unknown command");
                    let _ = rt.handle_unknown_subcommand("imag-ref", other, rt.cli())
                        .map_err_trace_exit_unwrap()
                        .code()
                        .map(::std::process::exit);
                },
            };
        });
}

fn deref(rt: &Runtime) {
    let cmd         = rt.cli().subcommand_matches("deref").unwrap();
    let basepath    = cmd.value_of("override-basepath");
    let cfg         = get_ref_config(&rt, "imag-ref").map_err_trace_exit_unwrap();
    let out         = rt.stdout();
    let mut outlock = out.lock();

    rt
        .ids::<::ui::PathProvider>()
        .map_err_trace_exit_unwrap()
        .unwrap_or_else(|| {
            error!("No ids supplied");
            ::std::process::exit(1);
        })
        .into_iter()
        .for_each(|id| {
            match rt.store().get(id.clone()).map_err_trace_exit_unwrap() {
                Some(entry) => {
                    let r_entry = entry.as_ref_with_hasher::<DefaultHasher>();

                    if let Some(alternative_basepath) = basepath {
                        r_entry.get_path_with_basepath_setting(&cfg, alternative_basepath)
                    } else {
                        r_entry.get_path(&cfg)
                    }
                    .map_err_trace_exit_unwrap()
                    .to_str()
                    .ok_or_else(|| ::libimagerror::errors::ErrorMsg::UTF8Error)
                    .map_err(Error::from)
                    .and_then(|s| writeln!(outlock, "{}", s).map_err(Error::from))
                    .map_err_trace_exit_unwrap();

                    let _ = rt.report_touched(&id).unwrap_or_exit();
                },
                None => {
                    error!("No entry for id '{}' found", id);
                    exit(1)
                },
            }
        });
}

fn remove(rt: &Runtime) {
    use libimaginteraction::ask::ask_bool;

    let cmd = rt.cli().subcommand_matches("remove").unwrap();
    let yes = cmd.is_present("yes");

    let mut input = rt.stdin().unwrap_or_else(|| {
        error!("No input stream. Cannot ask for permission");
        exit(1);
    });

    let mut output = rt.stdout();

    rt
        .ids::<::ui::PathProvider>()
        .map_err_trace_exit_unwrap()
        .unwrap_or_else(|| {
            error!("No ids supplied");
            ::std::process::exit(1);
        })
        .into_iter()
        .for_each(|id| {
            match rt.store().get(id.clone()).map_err_trace_exit_unwrap() {
                Some(mut entry) => {
                    if yes ||
                        ask_bool(&format!("Delete ref from entry '{}'", id), None, &mut input, &mut output)
                            .map_err_trace_exit_unwrap()
                    {
                        let _ = entry.as_ref_with_hasher_mut::<DefaultHasher>()
                            .remove_ref()
                            .map_err_trace_exit_unwrap();
                    } else {
                        info!("Aborted");
                    }
                },
                None => {
                    error!("No entry for id '{}' found", id);
                    exit(1)
                },
            }
        });
}

fn list_dead(rt: &Runtime) {
    let cfg        = get_ref_config(&rt, "imag-ref").map_err_trace_exit_unwrap();
    let cmd        = rt.cli().subcommand_matches("list-dead").unwrap(); // safe by main()
    let list_path  = cmd.is_present("list-dead-pathes");
    let list_id    = cmd.is_present("list-dead-ids");
    let mut output = rt.stdout();

    rt
        .ids::<crate::ui::PathProvider>()
        .map_err_trace_exit_unwrap()
        .unwrap_or_else(|| {
            error!("No ids supplied");
            ::std::process::exit(1);
        })
        .into_iter()
        .for_each(|id| {
            match rt.store().get(id.clone()).map_err_trace_exit_unwrap() {
                Some(entry) => {
                    let entry_ref = entry.as_ref_with_hasher::<DefaultHasher>();

                    if entry_ref.is_ref().map_err_trace_exit_unwrap() { // we only care if the entry is a ref
                        let entry_path = entry_ref.get_path(&cfg).map_err_trace_exit_unwrap();

                        if !entry_path.exists() {
                            if list_id {
                                writeln!(output, "{}", entry.get_location().local().display())
                            } else if list_path {
                                writeln!(output, "{}", entry_path.display())
                            } else {
                                unimplemented!()
                            }
                            .map_err(Error::from)
                            .map_err_trace_exit_unwrap();

                            let _ = rt.report_touched(entry.get_location()).unwrap_or_exit();
                        }
                    }
                }

                None => {
                    error!("Does not exist: {}", id.local().display());
                    exit(1)
                }
            }
        });
}

fn create(_rt: &Runtime) {
    unimplemented!()
}

