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

/// Helper to call imag-create
pub fn call(tempdir: &TempDir, targets: &[&str]) {
    let mut bin = binary(tempdir);

    // ensure that stdin is not used by the child process
    bin.stdin(std::process::Stdio::inherit());

    for target in targets.iter() {
        bin.arg(target);
    }
    debug!("Command = {:?}", bin);

    bin.assert().success();
}

pub fn binary(tempdir: &TempDir) -> Command {
    crate::imag::binary(tempdir, "imag-create")
}


#[test]
fn test_creating_works() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);

    call(&imag_home, &["test"]);

    let entry_path = crate::imag::store_path(&imag_home, &["test"]);

    assert!(entry_path.exists(), "Entry was not created: {:?}", entry_path);
    assert!(entry_path.is_file() , "Entry is not a file: {:?}", entry_path);

    let contents  = std::fs::read_to_string(entry_path).unwrap();
    let mut lines = contents.lines();

    assert_eq!(lines.next(), Some("---"));
    assert_eq!(lines.next(), Some("[imag]"));
    {
        let version_line = lines.next().unwrap();
        assert!(version_line.starts_with("version = '"));
        assert!(version_line.ends_with("'"));
        let version = version_line.replace("version = '", "").replace("'", "");
        let semver = semver::Version::parse(&version);
        assert!(semver.is_ok());
        assert!(!semver.unwrap().is_prerelease()); // we only want full versions in the header

    }
    assert_eq!(lines.next(), Some("---"));
    assert_eq!(lines.next(), None);
}

