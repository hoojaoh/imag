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

extern crate clap;
#[macro_use] extern crate log;
#[macro_use] extern crate vobject;
extern crate toml;
extern crate toml_query;
extern crate handlebars;
extern crate walkdir;
extern crate uuid;
extern crate serde_json;
#[macro_use] extern crate failure;

extern crate libimagcontact;
extern crate libimagstore;
#[macro_use] extern crate libimagrt;
extern crate libimagerror;
extern crate libimagutil;
extern crate libimaginteraction;
extern crate libimagentryedit;

use std::process::exit;
use std::path::PathBuf;
use std::io::Write;

use handlebars::Handlebars;
use clap::ArgMatches;
use toml_query::read::TomlValueReadExt;
use toml_query::read::TomlValueReadTypeExt;
use toml_query::read::Partial;
use walkdir::WalkDir;
use failure::Error;
use failure::err_msg;

use libimagrt::runtime::Runtime;
use libimagrt::setup::generate_runtime_setup;
use libimagerror::trace::MapErrTrace;
use libimagerror::io::ToExitCode;
use libimagerror::exit::ExitUnwrap;
use libimagerror::iter::TraceIterator;
use libimagcontact::store::ContactStore;
use libimagcontact::contact::Contact;
use libimagcontact::deser::DeserVcard;

mod ui;
mod util;
mod create;

use ui::build_ui;
use util::build_data_object_for_handlebars;
use create::create;

fn main() {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-contact",
                                    &version,
                                    "Contact management tool",
                                    build_ui);


    rt.cli()
        .subcommand_name()
        .map(|name| {
            debug!("Call {}", name);
            match name {
                "list"   => list(&rt),
                "import" => import(&rt),
                "show"   => show(&rt),
                "find"   => find(&rt),
                "create" => create(&rt),
                other    => {
                    debug!("Unknown command");
                    let _ = rt.handle_unknown_subcommand("imag-contact", other, rt.cli())
                        .map_err_trace_exit_unwrap()
                        .code()
                        .map(::std::process::exit);
                },
            }
        });
}

fn list(rt: &Runtime) {
    let scmd        = rt.cli().subcommand_matches("list").unwrap();
    let list_format = get_contact_print_format("contact.list_format", rt, &scmd);
    debug!("List format: {:?}", list_format);

    let iterator = rt
        .store()
        .all_contacts()
        .map_err_trace_exit_unwrap()
        .into_get_iter()
        .trace_unwrap_exit()
        .map(|fle| fle.ok_or_else(|| Error::from(err_msg("StoreId not found".to_owned()))))
        .trace_unwrap_exit()
        .map(|fle| {
            let _ = rt.report_touched(fle.get_location()).unwrap_or_exit();
            fle
        })
        .map(|e| e.deser())
        .trace_unwrap_exit()
        .enumerate();

    if scmd.is_present("json") {
        debug!("Listing as JSON");
        let v : Vec<DeserVcard> = iterator.map(|tpl| tpl.1).collect();

        match ::serde_json::to_string(&v) {
            Ok(s) => writeln!(rt.stdout(), "{}", s).to_exit_code().unwrap_or_exit(),
            Err(e) => {
                error!("Error generating JSON: {:?}", e);
                ::std::process::exit(1)
            }
        }
    } else {
        debug!("Not listing as JSON");
        let output     = rt.stdout();
        let mut output = output.lock();
        iterator
            .map(|(i, dvcard)| build_data_object_for_handlebars(i, &dvcard))
            .map(|data| list_format.render("format", &data).map_err(Error::from))
            .trace_unwrap_exit()
            .for_each(|s| {
                writeln!(output, "{}", s).to_exit_code().unwrap_or_exit()
            });
    }
}

fn import(rt: &Runtime) {
    let scmd           = rt.cli().subcommand_matches("import").unwrap(); // secured by main
    let force_override = scmd.is_present("force-override");
    let path           = scmd.value_of("path").map(PathBuf::from).unwrap(); // secured by clap

    let collection_name = rt.cli().value_of("contact-ref-collection-name").unwrap(); // default by clap
    let ref_config = rt.config()
        .ok_or_else(|| format_err!("No configuration, cannot continue!"))
        .map_err_trace_exit_unwrap()
        .read_partial::<libimagentryref::reference::Config>()
        .map_err(Error::from)
        .map_err_trace_exit_unwrap()
        .ok_or_else(|| format_err!("Configuration missing: {}", libimagentryref::reference::Config::LOCATION))
        .map_err_trace_exit_unwrap();
    // TODO: Refactor the above to libimagutil or libimagrt?


    if !path.exists() {
        error!("Path does not exist");
        exit(1)
    }

    if path.is_file() {
        let entry = rt
            .store()
            .retrieve_from_path(&path, &ref_config, &collection_name, force_override)
            .map_err_trace_exit_unwrap();

        let _ = rt.report_touched(entry.get_location()).unwrap_or_exit();
    } else if path.is_dir() {
        for entry in WalkDir::new(path).min_depth(1).into_iter() {
            let entry = entry
                .map_err(Error::from)
                .map_err_trace_exit_unwrap();

            if entry.file_type().is_file() {
                let pb = PathBuf::from(entry.path());
                let fle = rt
                    .store()
                    .retrieve_from_path(&pb, &ref_config, &collection_name, force_override)
                    .map_err_trace_exit_unwrap();

                let _ = rt.report_touched(fle.get_location()).unwrap_or_exit();
                info!("Imported: {}", entry.path().to_str().unwrap_or("<non UTF-8 path>"));
            } else {
                warn!("Ignoring non-file: {}", entry.path().to_str().unwrap_or("<non UTF-8 path>"));
            }
        }
    } else {
        error!("Path is neither directory nor file");
        exit(1)
    }
}

fn show(rt: &Runtime) {
    let scmd        = rt.cli().subcommand_matches("show").unwrap();
    let hash        = scmd.value_of("hash").map(String::from).unwrap(); // safed by clap
    let show_format = get_contact_print_format("contact.show_format", rt, &scmd);
    let out         = rt.stdout();
    let mut outlock = out.lock();

    rt.store()
        .all_contacts()
        .map_err_trace_exit_unwrap()
        .into_get_iter()
        .trace_unwrap_exit()
        .map(|o| o.unwrap_or_else(|| {
            error!("Failed to get entry");
            exit(1)
        }))
        .filter_map(|entry| {
            let deser = entry.deser().map_err_trace_exit_unwrap();

            if deser.uid()
                .ok_or_else(|| {
                    error!("Could not get StoreId from Store::all_contacts(). This is a BUG!");
                    ::std::process::exit(1)
                })
                .unwrap() // exited above
                .starts_with(&hash)
            {
                let _ = rt.report_touched(entry.get_location()).unwrap_or_exit();
                Some(deser)
            } else {
                None
            }
        })
        .enumerate()
        .for_each(|(i, elem)| {
            let data = build_data_object_for_handlebars(i, &elem);

            let s = show_format
                .render("format", &data)
                .map_err(Error::from)
                .map_err_trace_exit_unwrap();
            let _ = writeln!(outlock, "{}", s).to_exit_code().unwrap_or_exit();
        });
}

fn find(rt: &Runtime) {
    let scmd       = rt.cli().subcommand_matches("find").unwrap();
    let grepstring = scmd
        .values_of("string")
        .unwrap() // safed by clap
        .map(String::from)
        .collect::<Vec<String>>();

    // We don't know yet which we need, but we pay that price for simplicity of the codebase
    let show_format = get_contact_print_format("contact.show_format", rt, &scmd);
    let list_format = get_contact_print_format("contact.list_format", rt, &scmd);

    let iterator = rt
        .store()
        .all_contacts()
        .map_err_trace_exit_unwrap()
        .into_get_iter()
        .map(|el| {
            el.map_err_trace_exit_unwrap()
                .ok_or_else(|| {
                    error!("Could not get StoreId from Store::all_contacts(). This is a BUG!");
                    ::std::process::exit(1)
                })
                .unwrap() // safed above
        })
        .filter_map(|entry| {
            let card = entry.deser().map_err_trace_exit_unwrap();

            let str_contains_any = |s: &String, v: &Vec<String>| {
                v.iter().any(|i| s.contains(i))
            };

            let take = card.adr().iter().any(|a| str_contains_any(a, &grepstring))
                || card.email().iter().any(|a| str_contains_any(&a.address, &grepstring))
                || card.fullname().iter().any(|a| str_contains_any(a, &grepstring));

            if take {
                let _ = rt.report_touched(entry.get_location()).unwrap_or_exit();

                // optimization so we don't have to parse again in the next step
                Some((entry, card))
            } else {
                None
            }
        })
        .enumerate();

    if !rt.output_is_pipe() || rt.ignore_ids() {
        if scmd.is_present("json") {
            let v : Vec<DeserVcard> = iterator.map(|(_, tlp)| tlp.1).collect();

            match ::serde_json::to_string(&v) {
                Ok(s) => writeln!(rt.stdout(), "{}", s).to_exit_code().unwrap_or_exit(),
                Err(e) => {
                    error!("Error generating JSON: {:?}", e);
                    ::std::process::exit(1)
                }
            }
        } else if scmd.is_present("find-id") {
            iterator
            .for_each(|(_i, (entry, _))| {
                writeln!(rt.stdout(), "{}", entry.get_location())
                    .to_exit_code()
                    .unwrap_or_exit();
            })
        } else if scmd.is_present("find-full-id") {
            let storepath = rt.store().path().display();
            iterator
            .for_each(|(_i, (entry, _))| {
                writeln!(rt.stdout(), "{}/{}", storepath, entry.get_location())
                    .to_exit_code()
                    .unwrap_or_exit();
            })
        } else {
            iterator
            .for_each(|(i, (_, card))| {
                let fmt = if scmd.is_present("find-show") {
                    &show_format
                } else if scmd.is_present("find-list") {
                    &list_format
                } else { // default: find-list
                    &list_format
                };

                let data = build_data_object_for_handlebars(i, &card);
                let s = fmt
                    .render("format", &data)
                    .map_err(Error::from)
                    .map_err_trace_exit_unwrap();

                let _ = writeln!(rt.stdout(), "{}", s)
                    .to_exit_code()
                    .unwrap_or_exit();
            });
        }
    } else { // if not printing, we still have to consume the iterator to report the touched IDs
        let _ = iterator.collect::<Vec<_>>();
    }
}

fn get_contact_print_format(config_value_path: &'static str, rt: &Runtime, scmd: &ArgMatches) -> Handlebars {
    let fmt = scmd
        .value_of("format")
        .map(String::from)
        .unwrap_or_else(|| {
            rt.config()
                .ok_or_else(|| Error::from(err_msg("No configuration file")))
                .map_err_trace_exit_unwrap()
                .read_string(config_value_path)
                .map_err(Error::from)
                .map_err_trace_exit_unwrap()
                .ok_or_else(|| Error::from(err_msg("Configuration 'contact.list_format' does not exist")))
                .map_err_trace_exit_unwrap()
        });

    let mut hb = Handlebars::new();
    let _ = hb
        .register_template_string("format", fmt)
        .map_err(Error::from)
        .map_err_trace_exit_unwrap();

    hb.register_escape_fn(::handlebars::no_escape);
    ::libimaginteraction::format::register_all_color_helpers(&mut hb);
    ::libimaginteraction::format::register_all_format_helpers(&mut hb);
    hb
}

