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
use predicates::prelude::*;

pub fn binary(tempdir: &TempDir) -> Command {
    crate::imag::binary(tempdir, "imag-header")
}

#[test]
fn test_no_header_besides_version_after_creation() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);
    crate::imag_create::call(&imag_home, &["test"]);

    let mut bin = binary(&imag_home);
    bin.arg("test");
    bin.arg("string");
    bin.arg("imag.version");
    bin.assert().success();
}

#[test]
fn test_imag_version_as_semver_string() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);
    crate::imag_create::call(&imag_home, &["test"]);

    let imag_version = version::version!();

    let mut bin = binary(&imag_home);
    bin.arg("test");
    bin.arg("string");
    bin.arg("imag.version");

    let expected_output_str = format!("test - {}", imag_version);
    bin.assert()
        .stdout(predicate::eq(expected_output_str.as_bytes()))
        .success();
}

