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
#[macro_use] extern crate failure;
extern crate walkdir;
extern crate toml;
extern crate toml_query;

#[macro_use] extern crate libimagrt;
extern crate libimagerror;

use std::env;
use std::process::Command;
use std::process::Stdio;
use std::io::ErrorKind;
use std::io::{stdout, Write};
use std::collections::BTreeMap;
use std::path::PathBuf;

use walkdir::WalkDir;
use clap::{Arg, ArgMatches, AppSettings, SubCommand};
use toml::Value;
use toml_query::read::TomlValueReadExt;
use failure::Error;
use failure::ResultExt;
use failure::err_msg;
use failure::Fallible as Result;

use libimagrt::runtime::Runtime;
use libimagrt::spec::CliSpec;
use libimagrt::configuration::InternalConfiguration;

/// Returns the helptext, putting the Strings in cmds as possible
/// subcommands into it
fn help_text(cmds: Vec<String>) -> String {
    format!(r#"

     _
    (_)_ __ ___   __ _  __ _
    | | '_ \` _ \/ _\`|/ _\`|
    | | | | | | | (_| | (_| |
    |_|_| |_| |_|\__,_|\__, |
                       |___/
    -------------------------

    Usage: imag [--version | --versions | -h | --help] <command> <args...>

    imag - the personal information management suite for the commandline

    imag is a PIM suite for the commandline. It consists of several commands,
    called "modules". Each module implements one PIM aspect and all of these
    modules can be used independently.

    Available commands:

    {imagbins}

    Call a command with 'imag <command> <args>'
    Each command can be called with "--help" to get the respective helptext.

    Please visit https://github.com/matthiasbeyer/imag to view the source code,
    follow the development of imag or maybe even contribute to imag.

    imag is free software. It is released under the terms of LGPLv2.1

    (c) 2015-2018 Matthias Beyer and contributors"#,
        imagbins = cmds
            .into_iter()
            .map(|cmd| format!("\t{}\n", cmd))
            .fold(String::new(), |s, c| {
                s + c.as_str()
            }))
}

/// Returns the list of imag-* executables found in $PATH
fn get_commands() -> Result<Vec<String>> {
    let mut v = env::var("PATH")?
        .split(':')
        .flat_map(|elem| {
            WalkDir::new(elem)
                .max_depth(1)
                .into_iter()
                .filter(|path| match *path {
                    Ok(ref p) => p.file_name().to_str().map_or(false, |f| f.starts_with("imag-")),
                    Err(_)    => false,
                })
                .filter_map(|r| r.ok())
                .filter_map(|path| path
                    .file_name()
                   .to_str()
                   .and_then(|s| s.splitn(2, '-').nth(1).map(String::from))
                )
        })
        .filter(|path| if cfg!(debug_assertions) {
            // if we compile in debug mode during development, ignore everything that ends with
            // ".d", as developers might use the ./target/debug/ directory directly in `$PATH`.
            !path.ends_with(".d")
        } else {
            true
        })
        .collect::<Vec<String>>();

    v.sort();
    Ok(v)
}


fn main() -> Result<()> {
    // Initialize the Runtime and build the CLI
    let appname  = "imag";
    let version  = make_imag_version!();
    let about    = "imag - the PIM suite for the commandline";
    let commands = get_commands()?;
    let helptext = help_text(commands.clone());
    let mut app  = Runtime::get_default_cli_builder(appname, &version, about)
        .settings(&[AppSettings::AllowExternalSubcommands, AppSettings::ArgRequiredElseHelp])
        .arg(Arg::with_name("version")
             .long("version")
             .takes_value(false)
             .required(false)
             .multiple(false)
             .help("Get the version of imag"))
        .arg(Arg::with_name("versions")
             .long("versions")
             .takes_value(false)
             .required(false)
             .multiple(false)
             .help("Get the versions of the imag commands"))
        .subcommand(SubCommand::with_name("help").help("Show help"))
        .after_help(helptext.as_str());

    let long_help = {
        let mut v = vec![];
        app.write_long_help(&mut v)?;
        String::from_utf8(v).map_err(|_| err_msg("UTF8 Error"))?
    };
    let print_help = app.clone().get_matches().subcommand_name().map(|h| h == "help").unwrap_or(false);

    let mut out  = stdout();
    if print_help {
        writeln!(out, "{}", long_help).map_err(Error::from)
    } else {
        let enable_logging = app.enable_logging();
        let matches = app.matches();

        let rtp = ::libimagrt::runtime::get_rtp_match(&matches)?;
        let configpath = matches
            .value_of("config")
            .map_or_else(|| rtp.clone(), PathBuf::from);
        debug!("Config path = {:?}", configpath);
        let config = ::libimagrt::configuration::fetch_config(&configpath)?;

        if enable_logging {
            Runtime::init_logger(&matches, config.as_ref())
        }

        debug!("matches: {:?}", matches);

        // Begin checking for arguments

        if matches.is_present("version") {
            debug!("Showing version");
            writeln!(out, "imag {}", env!("CARGO_PKG_VERSION")).map_err(Error::from)
        } else {
            if matches.is_present("versions") {
                debug!("Showing versions");
                commands
                    .iter()
                    .map(|command| {
                        match Command::new(format!("imag-{}", command))
                            .stdin(::std::process::Stdio::inherit())
                            .stdout(::std::process::Stdio::piped())
                            .stderr(::std::process::Stdio::inherit())
                            .arg("--version")
                            .output()
                            .map(|v| v.stdout)
                        {
                            Ok(s) => match String::from_utf8(s) {
                                Ok(s) => format!("{:15} -> {}", command, s),
                                Err(e) => format!("UTF8 Error while working with output of imag{}: {:?}", command, e),
                            },
                            Err(e) => format!("Failed calling imag-{} -> {:?}", command, e),
                        }
                    })
                    .fold(Ok(()), |_, line| {
                        // The amount of newlines may differ depending on the subprocess
                        writeln!(out, "{}", line.trim()).map_err(Error::from)
                    })
            } else {
                let aliases = fetch_aliases(config.as_ref())
                    .map_err(Error::from)
                    .context("Error while fetching aliases from configuration file")?;

                // Matches any subcommand given, except calling for example 'imag --versions', as this option
                // does not exit. There's nothing to do in such a case
                if let (subcommand, Some(scmd)) = matches.subcommand() {
                    // Get all given arguments and further subcommands to pass to
                    // the imag-<> binary
                    // Providing no arguments is OK, and is therefore ignored here
                    let mut subcommand_args : Vec<String> = match scmd.values_of("") {
                        Some(values) => values.map(String::from).collect(),
                        None => Vec::new()
                    };

                    debug!("Processing forwarding of commandline arguments");
                    forward_commandline_arguments(&matches, &mut subcommand_args);

                    let subcommand = String::from(subcommand);
                    let subcommand = aliases.get(&subcommand).cloned().unwrap_or(subcommand);

                    debug!("Calling 'imag-{}' with args: {:?}", subcommand, subcommand_args);

                    // Create a Command, and pass it the gathered arguments
                    match Command::new(format!("imag-{}", subcommand))
                        .stdin(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit())
                        .args(&subcommand_args[..])
                        .spawn()
                        .and_then(|mut c| c.wait())
                    {
                        Ok(exit_status) => if !exit_status.success() {
                            debug!("imag-{} exited with non-zero exit code: {:?}", subcommand, exit_status);
                            Err(format_err!("imag-{} exited with non-zero exit code", subcommand))
                        } else {
                            debug!("Successful exit!");
                            Ok(())
                        },

                        Err(e) => {
                            debug!("Error calling the subcommand");
                            match e.kind() {
                                ErrorKind::NotFound => {
                                    writeln!(out, "No such command: 'imag-{}'", subcommand)?;
                                    writeln!(out, "See 'imag --help' for available subcommands").map_err(Error::from)
                                },
                                ErrorKind::PermissionDenied => {
                                    writeln!(out, "No permission to execute: 'imag-{}'", subcommand).map_err(Error::from)
                                },
                                _ => writeln!(out, "Error spawning: {:?}", e).map_err(Error::from),
                            }
                        }
                    }
                } else {
                    Ok(())
                }
            }
        }
    }
}

fn fetch_aliases(config: Option<&Value>) -> Result<BTreeMap<String, String>> {
    let cfg   = config.ok_or_else(|| err_msg("No configuration found"))?;
    let value = cfg
        .read("imag.aliases")
        .map_err(|_| err_msg("Reading from config failed"))?;

    match value {
        None                         => Ok(BTreeMap::new()),
        Some(&Value::Table(ref tbl)) => {
            let mut alias_mappings = BTreeMap::new();

            for (k, v) in tbl {
                match *v {
                    Value::String(ref alias)      => {
                        alias_mappings.insert(alias.clone(), k.clone());
                    },
                    Value::Array(ref aliases) => {
                        for alias in aliases {
                            match *alias {
                                Value::String(ref s) => {
                                    alias_mappings.insert(s.clone(), k.clone());
                                },
                                _ => {
                                    let e = format_err!("Not all values are a String in 'imag.aliases.{}'", k);
                                    return Err(e);
                                }
                            }
                        }
                    },

                    _ => {
                        let msg = format_err!("Type Error: 'imag.aliases.{}' is not a table or string", k);
                        return Err(msg);
                    },
                }
            }

            Ok(alias_mappings)
        },

        Some(_) => Err(err_msg("Type Error: 'imag.aliases' is not a table")),
    }
}

fn forward_commandline_arguments(m: &ArgMatches, scmd: &mut Vec<String>) {
    let push = |flag: Option<&str>, val_name: &str, m: &ArgMatches, v: &mut Vec<String>| {
        debug!("Push({flag:?}, {val_name:?}, {matches:?}, {v:?}",
               flag = flag, val_name = val_name, matches = m, v = v);

        if m.is_present(val_name) {
            m
                .value_of(val_name)
                .map(|val| {
                    debug!("Found '{:?}' = {:?}", val_name, val);
                    let flag = format!("--{}", flag.unwrap_or(val_name));
                    v.insert(0, String::from(val));
                    v.insert(0, flag);
                })
                .unwrap_or_else(|| {
                    let flag = format!("--{}", flag.unwrap_or(val_name));
                    v.insert(0, flag);
                });
        }
    };

    push(Some("verbose"), "verbosity", m , scmd);
    push(Some("debug"), "debugging", m , scmd);
    push(Some("no-color"), "no-color-output", m , scmd);
    push(Some("config"), "config", m , scmd);
    push(Some("override-config"), "config-override", m , scmd);
    push(Some("rtp"), "runtimepath", m , scmd);
    push(Some("store"), "storepath", m , scmd);
    push(Some("editor"), "editor", m , scmd);
    push(Some("ignore-ids"), "ignore-ids", m , scmd);
}

