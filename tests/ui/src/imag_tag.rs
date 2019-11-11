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
    crate::imag::binary(tempdir, "imag-tag")
}

pub fn call(tmpdir: &TempDir, args: &[&str]) -> Vec<String> {
    let mut binary = binary(tmpdir);
    binary.stdin(std::process::Stdio::inherit());
    binary.arg("--ignore-ids");
    binary.args(args);
    debug!("Command = {:?}", binary);
    crate::imag::stdout_of_command(binary)
}

pub fn call_give_ids(tmpdir: &TempDir, args: &[&str]) -> Vec<String> {
    let mut binary = binary(tmpdir);
    binary.stdin(std::process::Stdio::inherit());
    binary.args(args);
    debug!("Command = {:?}", binary);
    crate::imag::stdout_of_command(binary)
}

#[test]
fn test_new_entry_has_no_tags() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);
    crate::imag_create::call(&imag_home, &["test"]);

    let output = call(&imag_home, &["test", "list", "--linewise"]);
    debug!("output = {:?}", output);
    assert!(output.is_empty());
}

#[test]
fn test_after_adding_tag_there_is_tag() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);
    crate::imag_create::call(&imag_home, &["test"]);
    let _      = call(&imag_home, &["test", "add", "tag"]);

    let output = call(&imag_home, &["test", "list", "--linewise"]);
    debug!("output = {:?}", output);

    assert!(!output.is_empty());
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], "tag");
}

#[test]
fn test_after_adding_and_removing_there_is_no_tag() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);
    crate::imag_create::call(&imag_home, &["test"]);
    let _      = call(&imag_home, &["test", "add", "tag"]);
    let _      = call(&imag_home, &["test", "remove", "tag"]);

    let output = call(&imag_home, &["test", "list", "--linewise"]);
    debug!("output = {:?}", output);

    assert!(output.is_empty());
}

#[test]
fn test_adding_twice_does_not_add_twice() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);
    crate::imag_create::call(&imag_home, &["test"]);
    let _      = call(&imag_home, &["test", "add", "tag"]);
    let _      = call(&imag_home, &["test", "add", "tag"]);

    let output = call(&imag_home, &["test", "list", "--linewise"]);
    debug!("output = {:?}", output);

    assert!(!output.is_empty());
    assert_eq!(output.len(), 1);
    assert_eq!(output[0], "tag");
}

#[test]
fn test_listing_entries_with_certain_tag() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);
    crate::imag_create::call(&imag_home, &["test1", "test2", "test3"]);
    let _      = call(&imag_home, &["test1", "add", "tag1"]);
    let _      = call(&imag_home, &["test2", "add", "tag2"]);
    let _      = call(&imag_home, &["test3", "add", "tag3"]);

    let output = call_give_ids(&imag_home, &["present", "tag1"]);
    debug!("output = {:?}", output);

    assert!(!output.is_empty());
    assert_eq!(output.len(), 1);
    assert!(output[0].contains("test1"));
    assert!(output.iter().all(|s| !s.contains("test2")));
    assert!(output.iter().all(|s| !s.contains("test3")));
}

#[test]
fn test_listing_entries_without_certain_tag() {
    crate::setup_logging();
    let imag_home = crate::imag::make_temphome();
    crate::imag_init::call(&imag_home);
    crate::imag_create::call(&imag_home, &["test1", "test2", "test3"]);
    let _      = call(&imag_home, &["test1", "add", "tag1"]);
    let _      = call(&imag_home, &["test2", "add", "tag2"]);
    let _      = call(&imag_home, &["test3", "add", "tag3"]);

    let output = call_give_ids(&imag_home, &["missing", "tag1"]);
    debug!("output = {:?}", output);

    assert!(!output.is_empty());
    assert_eq!(output.len(), 2);
    assert!(output.iter().any(|s| s.contains("test2")));
    assert!(output.iter().any(|s| s.contains("test3")));
    assert!(output.iter().all(|s| !s.contains("test1")));
}

