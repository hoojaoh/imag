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

use assert_cmd::prelude::*;
use assert_fs::fixture::TempDir;

/// Helper to call imag-init
pub fn call(tempdir: &TempDir) {
    binary(tempdir).assert().success();
}

pub fn binary(tempdir: &TempDir) -> Command {
    let path = tempdir.path()
        .to_str()
        .map(String::from)
        .unwrap_or_else(|| panic!("Cannot create imag home path string"));

    let mut cmd = Command::cargo_bin("imag-init").unwrap();
    cmd.arg("--path");
    cmd.arg(path);
    cmd
}


#[test]
fn test_init_makes_imag_dir() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    call(&imag_home);
    assert!(imag_home.path().exists(), "imag dir does not exist");
}

