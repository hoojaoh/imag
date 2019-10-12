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

use assert_fs::fixture::TempDir;
use assert_cmd::prelude::*;

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

