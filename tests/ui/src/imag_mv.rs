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


pub fn binary(tempdir: &TempDir) -> Command {
    crate::imag::binary(tempdir, "imag-mv")
}

pub fn call(tmpdir: &TempDir, src: &str, dst: &str) {
    let mut binary = binary(tmpdir);
    binary.stdin(std::process::Stdio::inherit());
    binary.arg("--ignore-ids");
    binary.arg(src);
    binary.arg(dst);

    debug!("Command = {:?}", binary);

    binary.assert().success();
}

#[test]
fn test_after_moving_entry_is_moved() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);
    crate::imag_create::call(&imag_home, &["test"]);

    call(&imag_home, "test", "moved");
    {
        let entry_path = crate::imag::store_path(&imag_home, &["moved"]);
        assert!(entry_path.exists(), "Entry was not created: {:?}", entry_path);
        assert!(entry_path.is_file() , "Entry is not a file: {:?}", entry_path);
    }
    {
        let entry_path = crate::imag::store_path(&imag_home, &["test"]);
        assert!(!entry_path.exists(), "Entry still exists: {:?}", entry_path);
    }
}

