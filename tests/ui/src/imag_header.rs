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
use std::str::FromStr;

use assert_cmd::prelude::*;
use assert_fs::fixture::TempDir;

pub fn binary(tempdir: &TempDir) -> Command {
    crate::imag::binary(tempdir, "imag-header")
}

pub fn call(tmpdir: &TempDir, args: &[&str]) -> Vec<String> {
    let mut binary = binary(tmpdir);
    binary.stdin(std::process::Stdio::inherit());
    binary.args(args);
    debug!("Command = {:?}", binary);
    crate::imag::stdout_of_command(binary)
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

    let output              = call(&imag_home, &["--ignore-ids", "test", "read", "imag.version"]);
    let version             = version::version!();
    let imag_version        = format!("\"{}\"", version);
    debug!("output =  {:?}", output);
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], imag_version);

    let version = semver::Version::from_str(&version).unwrap();
    let parsed  = {
        let output_version = output[0].replace("\"", "");
        semver::Version::from_str(&output_version)
    };

    assert!(parsed.is_ok());
    let parsed = parsed.unwrap();

    assert_eq!(parsed.major, version.major);
    assert_eq!(parsed.minor, version.minor);
    assert_eq!(parsed.patch, version.patch);
    assert_eq!(parsed.pre,   version.pre);
}

