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
extern crate libimagrt;
extern crate libimagentryref;
extern crate libimagerror;
extern crate libimaginteraction;
extern crate libimagutil;

mod ui;

use std::io::Write;

use failure::Fallible as Result;
use failure::Error;
use failure::err_msg;
use clap::App;

use libimagrt::application::ImagApplication;
use libimagrt::runtime::Runtime;
use libimagentryref::reference::Ref;
use libimagentryref::reference::RefFassade;
use libimagentryref::hasher::default::DefaultHasher;
use libimagentryref::util::get_ref_config;
use libimagentryref::reference::MutRef;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagRef {}
impl ImagApplication for ImagRef {
    fn run(rt: Runtime) -> Result<()> {
        if let Some(name) = rt.cli().subcommand_name() {
            debug!("Call: {}", name);
            match name {
                "deref"     => deref(&rt),
                "create"    => create(&rt),
                "remove"    => remove(&rt),
                "list-dead" => list_dead(&rt),
                other => {
                    debug!("Unknown command");
                    if rt.handle_unknown_subcommand("imag-ref", other, rt.cli())?.success() {
                        Ok(())
                    } else {
                        Err(format_err!("Subcommand failed"))
                    }
                },
            }
        } else {
            Ok(())
        }
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Reference files outside of the store"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

fn deref(rt: &Runtime) -> Result<()> {
    let cmd         = rt.cli().subcommand_matches("deref").unwrap();
    let basepath    = cmd.value_of("override-basepath");
    let cfg         = get_ref_config(&rt, "imag-ref")?;
    let out         = rt.stdout();
    let mut outlock = out.lock();

    rt.ids::<::ui::PathProvider>()?
        .ok_or_else(|| err_msg("No ids supplied"))?
        .into_iter()
        .map(|id| {
            match rt.store().get(id.clone())? {
                Some(entry) => {
                    let r_entry = entry.as_ref_with_hasher::<DefaultHasher>();

                    if let Some(alternative_basepath) = basepath {
                        r_entry.get_path_with_basepath_setting(&cfg, alternative_basepath)
                    } else {
                        r_entry.get_path(&cfg)
                    }?
                    .to_str()
                    .ok_or_else(|| Error::from(::libimagerror::errors::ErrorMsg::UTF8Error))
                    .and_then(|s| writeln!(outlock, "{}", s).map_err(Error::from))?;

                    rt.report_touched(&id).map_err(Error::from)
                },
                None => Err(format_err!("No entry for id '{}' found", id))
            }
        })
        .collect()
}

fn remove(rt: &Runtime) -> Result<()> {
    use libimaginteraction::ask::ask_bool;

    let cmd        = rt.cli().subcommand_matches("remove").unwrap();
    let yes        = cmd.is_present("yes");
    let mut input  = rt.stdin().ok_or_else(|| err_msg("No input stream. Cannot ask for permission"))?;
    let mut output = rt.stdout();

    rt.ids::<::ui::PathProvider>()?
        .ok_or_else(|| err_msg("No ids supplied"))?
        .into_iter()
        .map(|id| {
            match rt.store().get(id.clone())? {
                None            => Err(format_err!("No entry for id '{}' found", id)),
                Some(mut entry) => {
                    if yes || ask_bool(&format!("Delete ref from entry '{}'", id), None, &mut input, &mut output)?  {
                        entry.as_ref_with_hasher_mut::<DefaultHasher>().remove_ref()
                    } else {
                        info!("Aborted");
                        Ok(())
                    }
                },
            }
        })
        .collect()
}

fn list_dead(rt: &Runtime) -> Result<()> {
    let cfg        = get_ref_config(&rt, "imag-ref")?;
    let cmd        = rt.cli().subcommand_matches("list-dead").unwrap(); // safe by main()
    let list_path  = cmd.is_present("list-dead-pathes");
    let list_id    = cmd.is_present("list-dead-ids");
    let mut output = rt.stdout();

    rt.ids::<crate::ui::PathProvider>()?
        .ok_or_else(|| err_msg("No ids supplied"))?
        .into_iter()
        .map(|id| {
            match rt.store().get(id.clone())? {
                Some(entry) => {
                    let entry_ref = entry.as_ref_with_hasher::<DefaultHasher>();

                    if entry_ref.is_ref()? { // we only care if the entry is a ref
                        let entry_path = entry_ref.get_path(&cfg)?;

                        if !entry_path.exists() {
                            if list_id {
                                writeln!(output, "{}", entry.get_location().local().display())
                            } else if list_path {
                                writeln!(output, "{}", entry_path.display())
                            } else {
                                unimplemented!()
                            }?;

                            rt.report_touched(entry.get_location()).map_err(Error::from)
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                }

                None => Err(format_err!("Does not exist: {}", id.local().display())),
            }
        })
        .collect()
}

fn create(_rt: &Runtime) -> Result<()> {
    unimplemented!()
}

