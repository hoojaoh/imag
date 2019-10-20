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

use std::process::Command;
use std::path::PathBuf;

use assert_fs::fixture::TempDir;
use assert_cmd::prelude::*;
use assert_cmd::assert::Assert;

pub fn make_temphome() -> TempDir {
    TempDir::new().unwrap().persist_if(std::env::var("IMAG_UI_TEST_PERSIST").is_ok())
}

pub fn binary(tempdir: &TempDir, binary_name: &str) -> Command {
    let path = tempdir.path()
        .to_str()
        .map(String::from)
        .unwrap_or_else(|| panic!("Cannot create imag home path string"));

    let mut cmd = Command::cargo_bin(binary_name).unwrap();
    cmd.arg("--rtp");
    cmd.arg(path);
    cmd
}

/// Run the passed command and get the stdout of it.
///
/// This function does _not_ ensure that stdin is inherited.
pub fn stdout_of_command(mut command: Command) -> Vec<String> {
    let assert = command.assert();
    let lines = String::from_utf8(assert.get_output().stdout.clone())
        .unwrap()
        .lines()
        .map(String::from)
        .collect();
    assert.success();
    lines
}

/// Run the passed command and get the stderr of it.
///
/// This function does _not_ ensure that stdin is inherited.
pub fn stderr_of_command(command: &mut Command) -> (Assert, Vec<String>) {
    let assert = command.assert();
    let lines = String::from_utf8(assert.get_output().stderr.clone())
        .unwrap()
        .lines()
        .map(String::from)
        .collect();
    (assert, lines)
}

/// Create a PathBuf for a file in a TempDir
pub fn file_path(tempdir: &TempDir, path_elements: &[&str]) -> PathBuf {
    create_path_for(tempdir.path().to_path_buf(), path_elements)
}

pub fn store_path(tempdir: &TempDir, path_elements: &[&str]) -> PathBuf {
    let mut path = tempdir.path().to_path_buf();
    path.push("store");
    create_path_for(path, path_elements)
}

fn create_path_for(mut path: PathBuf, path_elements: &[&str]) -> PathBuf {
    path_elements.iter().for_each(|el| path.push(el));
    debug!("Calculated path = {:?}", path);
    path
}

