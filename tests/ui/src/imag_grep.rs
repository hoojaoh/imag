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
use std::io::Write;

use assert_fs::fixture::TempDir;

pub fn call(tempdir: &TempDir, pattern: &str) -> Vec<String> {
    let mut binary = binary(tempdir);
    binary.arg("--ignore-ids");
    binary.arg(pattern);

    // ensure that stdin is not used by the child process
    binary.stdin(std::process::Stdio::inherit());
    crate::imag::stdout_of_command(binary)
}

pub fn binary(tempdir: &TempDir) -> Command {
    crate::imag::binary(tempdir, "imag-grep")
}

#[test]
fn test_grepping_in_empty_store() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);

    let output = call(&imag_home, "something");
    assert_eq!(output[0], "Processed 0 files, 0 matches, 0 nonmatches");
    assert_eq!(output.len(), 1);
}

#[test]
fn test_grepping_nonempty_store() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);
    crate::imag_create::call(&imag_home, &["something"]);

    let output = call(&imag_home, "something");
    assert_eq!(output[0], "Processed 1 files, 0 matches, 1 nonmatches");
    assert_eq!(output.len(), 1);
}

#[test]
fn test_grepping_not_available_string() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);

    let filename = &["something"];
    crate::imag_create::call(&imag_home, filename);
    let filepath = crate::imag::store_path(&imag_home, filename);

    {
        debug!("Appending to file = {}", filepath.display());
        let mut file = ::std::fs::OpenOptions::new()
            .append(true)
            .create(false)
            .open(&filepath)
            .unwrap();

        let _ = writeln!(file, "unavailable").unwrap();
    }

    let output = call(&imag_home, "something");
    assert_eq!(output[0], "Processed 1 files, 0 matches, 1 nonmatches");
    assert_eq!(output.len(), 1);
}

#[test]
fn test_grepping_available_string() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);

    let filename = &["something"];
    crate::imag_create::call(&imag_home, filename);
    let filepath = crate::imag::store_path(&imag_home, filename);

    let filetext = "some text is here";
    {
        debug!("Appending to file = {}", filepath.display());
        let mut file = ::std::fs::OpenOptions::new()
            .append(true)
            .create(false)
            .open(&filepath)
            .unwrap();

        let _ = writeln!(file, "{}", filetext).unwrap();
    }

    let output = call(&imag_home, filetext);
    debug!("output = {:?}", output);
    assert!(!output.is_empty());
    assert_eq!(output[0], format!("{}:", filename[0]));
    assert_eq!(output[1], format!(" '{}'", filetext));
    assert_eq!(output[2], "");
    assert_eq!(output[3], "Processed 1 files, 1 matches, 0 nonmatches");
    assert_eq!(output.len(), 4);
}

