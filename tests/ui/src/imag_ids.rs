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

/// Helper to call imag-init
pub fn call(tempdir: &TempDir) -> Vec<String> {
    let mut binary = binary(tempdir);

    // ensure that stdin is not used by the child process
    binary.stdin(std::process::Stdio::inherit());
    crate::imag::stdout_of_command(binary)
}

pub fn binary(tempdir: &TempDir) -> Command {
    crate::imag::binary(tempdir, "imag-ids")
}


#[test]
fn test_no_ids_after_init() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);

    binary(&imag_home)
        .assert()
        .success()
        .stdout(predicate::eq(b"" as &[u8]));
}

#[test]
fn test_one_id_after_creating_one_entry() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);
    crate::imag_create::call(&imag_home, &["test"]);
    let ids = call(&imag_home);

    assert_eq!(ids.len(), 1);
    assert_eq!(ids[0], "test");
}
