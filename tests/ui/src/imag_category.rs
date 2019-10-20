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

pub fn binary(tempdir: &TempDir) -> Command {
    crate::imag::binary(tempdir, "imag-category")
}

pub fn call(tmpdir: &TempDir, args: &[&str]) -> Vec<String> {
    let mut binary = binary(tmpdir);
    binary.stdin(std::process::Stdio::inherit());
    binary.arg("--ignore-ids");
    binary.args(args);
    debug!("Command = {:?}", binary);
    crate::imag::stdout_of_command(binary)
}

#[test]
fn test_new_entry_has_no_category() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);
    crate::imag_create::call(&imag_home, &["test"]);

    let (assert, stderr_output) = {
        let mut binary = binary(&imag_home);
        binary.stdin(std::process::Stdio::inherit());
        binary.args(&["--ignore-ids", "get", "test"]);
        crate::imag::stderr_of_command(&mut binary)
    };

    assert.failure();
    assert!(stderr_output.iter().any(|substr| substr.contains("Category name missing")));
}

#[test]
fn test_after_setting_a_new_category_there_is_a_category() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);
    crate::imag_create::call(&imag_home, &["test"]);

    let _ = call(&imag_home, &["create-category", "cat"]);
    let _ = call(&imag_home, &["set", "cat", "test"]);
    let output = call(&imag_home, &["get", "test"]);
    debug!("output = {:?}", output);
    assert!(!output.is_empty());
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], "cat");
}

