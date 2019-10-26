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
#[macro_use] extern crate failure;

#[cfg(test)]
extern crate toml;

#[macro_use] extern crate libimagrt;
extern crate libimagerror;

mod ui;

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::path::Path;
use std::process::Command;

use failure::Fallible as Result;
use failure::ResultExt;
use failure::Error;
use failure::err_msg;

use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;

use clap::App;

const CONFIGURATION_STR : &str = include_str!("../imagrc.toml");

const GITIGNORE_STR : &str = r#"
# We ignore the imagrc.toml file by default
#
# That is because we expect the user to put
# this dotfile into his dotfile repository
# and symlink it here.
# If you do not do this, feel free to remove
# this line from the gitignore and add the
# configuration to this git repository.

imagrc.toml
"#;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagInit {}
impl ImagApplication for ImagInit {
    fn run(_rt: Runtime) -> Result<()> {
        panic!("imag-init needs to be run as a seperate binary, or we'll need to figure something out here!");
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Intialize the imag store"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

pub fn imag_init() -> Result<()> {
    let version = make_imag_version!();
    let app     = ui::build_ui(Runtime::get_default_cli_builder(
        "imag-init",
        version.as_str(),
        "Intializes the imag store, optionally with git"));
    let matches = app.get_matches();
    let mut out = ::std::io::stdout();

    let path = if let Some(p) = matches.value_of("path") {
        PathBuf::from(String::from(p))
    } else {
        ::std::env::var("HOME")
            .map_err(Error::from)
            .map(PathBuf::from)
            .map(|mut p| { p.push(".imag"); p })
            .and_then(|path| if path.exists() {
                Err(format_err!("Cannot continue: Path '{}' already exists", path.display()))
            } else {
                Ok(path)
            })
            .map_err(|_| err_msg("Failed to retrieve/build path for imag directory."))?
    };

    {
        let mut store_path = path.clone();
        store_path.push("store");
        println!("Creating {}", store_path.display());

        ::std::fs::create_dir_all(store_path).context("Failed to create directory")?;
    }

    let config_path = {
        let mut config_path = path.clone();
        config_path.push("imagrc.toml");
        config_path
    };

    let _ = OpenOptions::new()
        .write(true)
        .create(true)
        .open(config_path)
        .map_err(Error::from)
        .and_then(|mut f| {
            let content = if matches.is_present("devel") {
                get_config_devel()
            } else {
                get_config()
            };

            f.write_all(content.as_bytes())
                .context("Failed to write complete config to file")
                .map_err(Error::from)
        })
        .context("Failed to open new configuration file")?;

    if find_command("git").is_some() && !matches.is_present("nogit") {
        // we initialize a git repository
        writeln!(out, "Going to initialize a git repository in the imag directory...")?;

        let gitignore_path = {
            let mut gitignore_path = path.clone();
            gitignore_path.push(".gitignore");
            gitignore_path.to_str().map(String::from)
        }.ok_or_else(|| err_msg("Cannot convert path to string"))?;

        let _ = OpenOptions::new()
            .write(true)
            .create(true)
            .open(gitignore_path.clone())
            .map_err(Error::from)
            .and_then(|mut f| {
                f.write_all(GITIGNORE_STR.as_bytes())
                    .context("Failed to write complete gitignore to file")
                    .map_err(Error::from)
            })
            .context("Failed to open new configuration file")?;

        let path_str = path.to_str().map(String::from).ok_or_else(|| err_msg("Cannot convert path to string"))?;
        let worktree = format!("--work-tree={}", path_str);
        let gitdir   = format!("--git-dir={}/.git", path_str);

        {
            let output = Command::new("git")
                .args(&[&worktree, &gitdir, "--no-pager", "init"])
                .output()
                .context("Calling 'git init' failed")?;

            if output.status.success() {
                writeln!(out, "{}", String::from_utf8(output.stdout).expect("No UTF-8 output"))?;
                writeln!(out, "'git {} {} --no-pager init' succeeded", worktree, gitdir)?;
            } else {
                writeln!(out, "{}", String::from_utf8(output.stderr).expect("No UTF-8 output"))?;
                if !output.status.success() {
                    return Err(err_msg("Failed to execute git command"));
                }
            }
        }

        {
            let output = Command::new("git")
                .args(&[&worktree, &gitdir, "--no-pager", "add", &gitignore_path])
                .output()
                .context("Calling 'git add' failed")?;

            if output.status.success() {
                writeln!(out, "{}", String::from_utf8(output.stdout).expect("No UTF-8 output"))?;
                writeln!(out, "'git {} {} --no-pager add {}' succeeded", worktree, gitdir, gitignore_path)?;
            } else {
                writeln!(out, "{}", String::from_utf8(output.stderr).expect("No UTF-8 output"))?;
                if !output.status.success() {
                    return Err(err_msg("Failed to execute git command"));
                }
            }
        }

        {
            let output = Command::new("git")
                .args(&[&worktree, &gitdir, "--no-pager", "commit", &gitignore_path, "-m", "'Initial import'"])
                .output()
                .context("Calling 'git commit' failed")?;
            if output.status.success() {
                writeln!(out, "{}", String::from_utf8(output.stdout).expect("No UTF-8 output"))?;
                writeln!(out, "'git {} {} --no-pager commit {} -m 'Initial import'' succeeded", worktree, gitdir, gitignore_path)?;
            } else {
                writeln!(out, "{}", String::from_utf8(output.stderr).expect("No UTF-8 output"))?;
                if !output.status.success() {
                    return Err(err_msg("Failed to execute git command"));
                }
            }
        }

        writeln!(out, "git stuff finished!")?;
    } else {
        writeln!(out, "No git repository will be initialized")?;
    }

    writeln!(out, "Ready. Have fun with imag!").map_err(Error::from)
}

fn get_config() -> String {
    get_config_devel()
        .replace(
            r#"level = "debug""#,
            r#"level = "info""#
        )
}

fn get_config_devel() -> String {
    String::from(CONFIGURATION_STR)
}

fn find_command<P: AsRef<Path>>(exe_name: P) -> Option<PathBuf> {
    ::std::env::var_os("PATH")
        .and_then(|paths| {
            ::std::env::split_paths(&paths)
                .filter_map(|dir| {
                    let full_path = dir.join(&exe_name);
                    if full_path.is_file() {
                        Some(full_path)
                    } else {
                        None
                    }
                })
                .next()
        })
}

#[cfg(test)]
mod tests {
    use toml::from_str;
    use toml::Value;
    use super::get_config;
    use super::get_config_devel;

    #[test]
    fn test_config() {
        let val = from_str::<Value>(&get_config()[..]);
        assert!(val.is_ok(), "Config parsing errored: {:?}", val);

        let val = from_str::<Value>(&get_config_devel()[..]);
        assert!(val.is_ok(), "Config parsing errored: {:?}", val);
    }

}
