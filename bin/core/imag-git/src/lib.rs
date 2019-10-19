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
extern crate toml;
extern crate toml_query;
#[macro_use] extern crate failure;

extern crate libimagrt;
extern crate libimagerror;

use std::io::ErrorKind;
use std::process::Command;

use toml::Value;
use toml_query::read::TomlValueReadExt;
use clap::App;
use failure::Fallible as Result;
use failure::ResultExt;
use failure::err_msg;

use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;

mod ui;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagGit {}
impl ImagApplication for ImagGit {
    fn run(rt: Runtime) -> Result<()> {
        let execute_in_store = rt
            .config()
            .ok_or_else(|| err_msg("No configuration. Please use git yourself, not via imag-git"))
            .context("Won't continue without configuration.")
            ?
            .read("git.execute_in_store")
            .context("Failed to read config setting 'git.execute_in_store'")
            ?
            .ok_or_else(|| err_msg("Missing config setting 'git.execute_in_store'"))
            ?;

        let execute_in_store = match *execute_in_store {
            Value::Boolean(b) => Ok(b),
            _ => Err(err_msg("Type error: 'git.execute_in_store' is not a boolean!")),
        }?;

        let execpath = if execute_in_store {
            rt.store().path().to_str()
        } else {
            rt.rtp().to_str()
        }
        .map(String::from)
        .ok_or_else(|| format_err!("Cannot parse to string: {:?}", rt.store().path()))?;

        let mut command = Command::new("git");
        command
            .stdin(::std::process::Stdio::inherit())
            .stdout(::std::process::Stdio::inherit())
            .stderr(::std::process::Stdio::inherit())
            .arg("-C").arg(&execpath);

        let args = rt
            .cli()
            .values_of("")
            .map(|vs| vs.map(String::from).collect())
            .unwrap_or_else(|| vec![]);

        debug!("Adding args = {:?}", args);
        command.args(&args);

        if let (external, Some(ext_m)) = rt.cli().subcommand() {
            command.arg(external);
            let args = ext_m
                .values_of("")
                .map(|vs| vs.map(String::from).collect())
                .unwrap_or_else(|| vec![]);

            debug!("Adding subcommand '{}' and args = {:?}", external, args);
            command.args(&args);
        }

        debug!("Calling: {:?}", command);

        match command.spawn().and_then(|mut c| c.wait()) {
            Ok(exit_status) => {
                if !exit_status.success() {
                    debug!("git exited with non-zero exit code: {:?}", exit_status);
                    Err(format_err!("git exited with non-zero exit code: {:?}", exit_status))
                } else {
                    debug!("Successful exit!");
                    Ok(())
                }
            },

            Err(e) => {
                debug!("Error calling git");
                Err(match e.kind() {
                    ErrorKind::NotFound         => err_msg("Cannot find 'git' executable"),
                    ErrorKind::PermissionDenied => err_msg("No permission to execute: 'git'"),
                    _                           => format_err!("Error spawning: {:?}", e),
                })
            }
        }
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Helper to call git in the store"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
