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

#![forbid(unsafe_code)]

#![deny(
    non_camel_case_types,
    non_snake_case,
    path_statements,
    trivial_numeric_casts,
    unstable_features,
    unused_allocation,
    unused_import_braces,
    unused_imports,
    unused_must_use,
    unused_mut,
    unused_qualifications,
    while_true,
)]

extern crate clap;
extern crate resiter;
#[macro_use] extern crate log;

#[cfg(test)] extern crate toml;
#[macro_use] extern crate failure;

extern crate libimagstore;
extern crate libimagrt;
extern crate libimagentrytag;
extern crate libimagerror;

#[cfg(test)]
#[macro_use]
extern crate libimagutil;

#[cfg(not(test))]
extern crate libimagutil;

#[cfg(test)]
extern crate toml_query;

#[cfg(test)]
extern crate env_logger;

use std::io::Write;

use failure::Fallible as Result;
use failure::Error;
use failure::err_msg;
use resiter::AndThen;
use resiter::Map;
use resiter::FilterMap;

use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
use libimagentrytag::tagable::Tagable;
use libimagentrytag::tag::is_tag_str;
use libimagentrytag::tag::Tag;
use libimagstore::storeid::StoreId;

use clap::{App, ArgMatches};

mod ui;


/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagTag {}
impl ImagApplication for ImagTag {
    fn run(rt: Runtime) -> Result<()> {
        let process = |iter: &mut dyn Iterator<Item = Result<StoreId>>| -> Result<()> {
            match rt.cli().subcommand() {
                ("list", _) => iter
                    .map_ok(|id| list(id, &rt))
                    .collect::<Result<Vec<_>>>()
                    .map(|_| ()),

                ("remove", _) => iter.and_then_ok(|id| {
                    let add = None;
                    let rem = get_remove_tags(rt.cli())?;
                    debug!("id = {:?}, add = {:?}, rem = {:?}", id, add, rem);
                    alter(&rt, id, add, rem)
                }).collect(),

                ("add", _) => iter.and_then_ok(|id| {
                    let add = get_add_tags(rt.cli())?;
                    let rem = None;
                    debug!("id = {:?}, add = {:?}, rem = {:?}", id, add, rem);
                    alter(&rt, id, add, rem)
                }).collect(),

                ("present", Some(scmd)) => {
                    let must_be_present = scmd
                        .values_of("present-tag")
                        .unwrap()
                        .map(String::from)
                        .collect::<Vec<String>>();

                    must_be_present.iter().map(|t| is_tag_str(t)).collect::<Result<Vec<_>>>()?;

                    iter.filter_map_ok(|id| {
                            match rt.store().get(id.clone()) {
                                Err(e) => Some(Err(e)),
                                Ok(None) => Some(Err(format_err!("No entry for id {}", id))),
                                Ok(Some(entry)) => {
                                    let entry_tags = match entry.get_tags() {
                                        Err(e) => return Some(Err(e)),
                                        Ok(e) => e,
                                    };

                                    if must_be_present.iter().all(|pres| entry_tags.contains(pres)) {
                                        Some(Ok(entry))
                                    } else {
                                        None
                                    }
                                }
                            }
                        })
                        .flatten()
                        .and_then_ok(|e| {
                            if !rt.output_is_pipe() {
                                writeln!(rt.stdout(), "{}", e.get_location())?;
                            }
                            Ok(e)
                        })
                        .and_then_ok(|e| rt.report_touched(e.get_location()).map_err(Error::from))
                        .collect::<Result<Vec<_>>>()
                        .map(|_| ())
                },

                ("missing", Some(scmd)) => {
                    let must_be_missing = scmd
                        .values_of("missing-tag")
                        .unwrap()
                        .map(String::from)
                        .collect::<Vec<String>>();

                    must_be_missing.iter().map(|t| is_tag_str(t)).collect::<Result<Vec<_>>>()?;

                    iter.filter_map_ok(|id| {
                            match rt.store().get(id.clone()) {
                                Err(e) => Some(Err(e)),
                                Ok(None) => Some(Err(format_err!("No entry for id {}", id))),
                                Ok(Some(entry)) => {
                                    let entry_tags = match entry.get_tags() {
                                        Err(e) => return Some(Err(e)),
                                        Ok(e) => e,
                                    };

                                    if must_be_missing.iter().all(|miss| !entry_tags.contains(miss)) {
                                        Some(Ok(entry))
                                    } else {
                                        None
                                    }
                                }
                            }
                        })
                        .flatten()
                        .and_then_ok(|e| {
                            if !rt.output_is_pipe() {
                                writeln!(rt.stdout(), "{}", e.get_location())?;
                            }
                            Ok(e)
                        })
                        .and_then_ok(|e| rt.report_touched(e.get_location()).map_err(Error::from))
                        .collect::<Result<Vec<_>>>()
                        .map(|_| ())
                },

                (other, _) => {
                    debug!("Unknown command");
                    if rt.handle_unknown_subcommand("imag-tag", other, rt.cli())?.success() {
                        Ok(())
                    } else {
                        Err(format_err!("Subcommand failed"))
                    }
                },
            }
        };

        match rt.ids::<crate::ui::PathProvider>()? {
            Some(ids) => process(&mut ids.into_iter().map(Ok)),
            None => process(&mut rt.store().entries()?),
        }
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Manage tags of entries"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

fn alter(rt: &Runtime, path: StoreId, add: Option<Vec<Tag>>, rem: Option<Vec<Tag>>) -> Result<()> {
    match rt.store().get(path.clone())? {
        Some(mut e) => {
            debug!("Entry header now = {:?}", e.get_header());

            if let Some(tags) = add {
                debug!("Adding tags = '{:?}'", tags);
                tags.into_iter().map(|tag| {
                    debug!("Adding tag '{:?}'", tag);
                    e.add_tag(tag)
                }).collect::<Result<Vec<_>>>()?;
            } // it is okay to ignore a None here

            debug!("Entry header now = {:?}", e.get_header());

            if let Some(tags) = rem {
                debug!("Removing tags = '{:?}'", tags);
                tags.into_iter().map(|tag| {
                    debug!("Removing tag '{:?}'", tag);
                    e.remove_tag(tag)
                }).collect::<Result<Vec<_>>>()?;
            } // it is okay to ignore a None here

            debug!("Entry header now = {:?}", e.get_header());
        },

        None => {
            info!("No entry found.");
        },
    }

    rt.report_touched(&path).map_err(Error::from)
}

fn list(path: StoreId, rt: &Runtime) -> Result<()> {
    let entry        = rt.store().get(path.clone())?.ok_or_else(|| err_msg("No entry found"))?;
    let scmd         = rt.cli().subcommand_matches("list").unwrap(); // safe, we checked in main()
    let json_out     = scmd.is_present("json");
    let line_out     = scmd.is_present("linewise");
    let sepp_out     = scmd.is_present("sep");
    let mut comm_out = scmd.is_present("commasep");

    if !vec![json_out, line_out, comm_out, sepp_out].iter().any(|v| *v) {
        // None of the flags passed, go to default
        comm_out = true;
    }

    let tags = entry.get_tags()?;

    if json_out {
        unimplemented!()
    }

    if line_out {
        for tag in &tags {
            writeln!(rt.stdout(), "{}", tag)?;
        }
    }

    if sepp_out {
        let sepp = scmd.value_of("sep").unwrap(); // we checked before
        writeln!(rt.stdout(), "{}", tags.join(sepp))?;
    }

    if comm_out {
        writeln!(rt.stdout(), "{}", tags.join(", "))?;
    }

    rt.report_touched(&path).map_err(Error::from)
}

/// Get the tags which should be added from the commandline
///
/// Returns none if the argument was not specified
fn get_add_tags(matches: &ArgMatches) -> Result<Option<Vec<Tag>>> {
    retrieve_tags(matches, "add", "add-tags")
}

/// Get the tags which should be removed from the commandline
///
/// Returns none if the argument was not specified
fn get_remove_tags(matches: &ArgMatches) -> Result<Option<Vec<Tag>>> {
    retrieve_tags(matches, "remove", "remove-tags")
}

fn retrieve_tags(m: &ArgMatches, s: &'static str, v: &'static str) -> Result<Option<Vec<Tag>>> {
    Ok(Some(m
         .subcommand_matches(s)
         .ok_or_else(|| format_err!("Expected subcommand '{}', but was not specified", s))?
         .values_of(v)
         .unwrap() // enforced by clap
         .map(String::from)
         .collect()))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::ffi::OsStr;

    use toml::value::Value;
    use toml_query::read::TomlValueReadExt;
    use failure::Fallible as Result;
    use failure::Error;

    use libimagrt::runtime::Runtime;
    use libimagstore::storeid::StoreId;
    use libimagstore::store::{FileLockEntry, Entry};

    use super::*;

    make_mock_app! {
        app "imag-tag";
        modulename mock;
        version env!("CARGO_PKG_VERSION");
        with help "imag-tag mocking app";
        with ui builder function crate::ui::build_ui;
    }
    use self::mock::generate_test_runtime;

    fn create_test_default_entry<'a, S: AsRef<OsStr>>(rt: &'a Runtime, name: S) -> Result<StoreId> {
        let mut path = PathBuf::new();
        path.set_file_name(name);

        let default_entry = Entry::new(StoreId::new(PathBuf::from("")).unwrap())
            .to_str()
            .unwrap();

        let id = StoreId::new(path)?;
        let mut entry = rt.store().create(id.clone())?;
        entry.get_content_mut().push_str(&default_entry);

        Ok(id)
    }

    fn get_entry_tags<'a>(entry: &'a FileLockEntry<'a>) -> Result<Option<&'a Value>> {
        entry.get_header().read(&"tag.values".to_owned()).map_err(Error::from)
    }

    fn tags_toml_value<I: IntoIterator<Item = &'static str>>(tags: I) -> Value {
        Value::Array(tags.into_iter().map(|s| Value::String(s.to_owned())).collect())
    }

    fn setup_logging() {
        let _ = ::env_logger::try_init();
    }

    #[test]
    fn test_tag_add_adds_tag() -> Result<()> {
        setup_logging();
        debug!("Generating runtime");
        let name = "test-tag-add-adds-tags";
        let rt = generate_test_runtime(vec![name, "add", "foo"]).unwrap();

        debug!("Creating default entry");
        create_test_default_entry(&rt, name).unwrap();
        let id = PathBuf::from(String::from(name));

        debug!("Getting 'add' tags");
        let add = get_add_tags(rt.cli())?;
        debug!("Add-tags: {:?}", add);

        debug!("Altering things");
        alter(&rt, StoreId::new(id.clone()).unwrap(), add, None)?;
        debug!("Altered");

        let test_entry = rt.store().get(id).unwrap().unwrap();

        let test_tags  = get_entry_tags(&test_entry);
        assert!(test_tags.is_ok(), "Should be Ok(_) = {:?}", test_tags);

        let test_tags  = test_tags.unwrap();
        assert!(test_tags.is_some(), "Should be Some(_) = {:?}", test_tags);
        let test_tags  = test_tags.unwrap();

        assert_ne!(*test_tags, tags_toml_value(vec![]));
        assert_eq!(*test_tags, tags_toml_value(vec!["foo"]));
        Ok(())
    }

    #[test]
    fn test_tag_remove_removes_tag() -> Result<()> {
        setup_logging();
        debug!("Generating runtime");
        let name = "test-tag-remove-removes-tag";
        let rt = generate_test_runtime(vec![name, "remove", "foo"]).unwrap();

        debug!("Creating default entry");
        create_test_default_entry(&rt, name).unwrap();
        let id = PathBuf::from(String::from(name));

        // Manually add tags
        let add = Some(vec![ "foo".to_owned() ]);

        debug!("Getting 'remove' tags");
        let rem = get_remove_tags(rt.cli())?;
        debug!("Rem-tags: {:?}", rem);

        debug!("Altering things");
        alter(&rt, StoreId::new(id.clone()).unwrap(), add, rem)?;
        debug!("Altered");

        let test_entry = rt.store().get(id).unwrap().unwrap();
        let test_tags  = get_entry_tags(&test_entry).unwrap().unwrap();

        assert_eq!(*test_tags, tags_toml_value(vec![]));
        Ok(())
    }

    #[test]
    fn test_tag_remove_removes_only_to_remove_tag() -> Result<()> {
        setup_logging();
        debug!("Generating runtime");
        let name = "test-tag-remove-removes-only-to-remove-tag-doesnt-crash-on-nonexistent-tag";
        let rt = generate_test_runtime(vec![name, "remove", "foo"]).unwrap();

        debug!("Creating default entry");
        create_test_default_entry(&rt, name).unwrap();
        let id = PathBuf::from(String::from(name));

        // Manually add tags
        let add = Some(vec![ "foo".to_owned(), "bar".to_owned() ]);

        debug!("Getting 'remove' tags");
        let rem = get_remove_tags(rt.cli())?;
        debug!("Rem-tags: {:?}", rem);

        debug!("Altering things");
        alter(&rt, StoreId::new(id.clone()).unwrap(), add, rem)?;
        debug!("Altered");

        let test_entry = rt.store().get(id).unwrap().unwrap();
        let test_tags  = get_entry_tags(&test_entry).unwrap().unwrap();

        assert_eq!(*test_tags, tags_toml_value(vec!["bar"]));
        Ok(())
    }

    #[test]
    fn test_tag_remove_removes_but_doesnt_crash_on_nonexistent_tag() -> Result<()> {
        setup_logging();
        debug!("Generating runtime");
        let name = "test-tag-remove-removes-but-doesnt-crash-on-nonexistent-tag";
        let rt = generate_test_runtime(vec![name, "remove", "foo", "bar"]).unwrap();

        debug!("Creating default entry");
        create_test_default_entry(&rt, name).unwrap();
        let id = PathBuf::from(String::from(name));

        // Manually add tags
        let add = Some(vec![ "foo".to_owned() ]);

        debug!("Getting 'remove' tags");
        let rem = get_remove_tags(rt.cli())?;
        debug!("Rem-tags: {:?}", rem);

        debug!("Altering things");
        alter(&rt, StoreId::new(id.clone()).unwrap(), add, rem)?;
        debug!("Altered");

        let test_entry = rt.store().get(id).unwrap().unwrap();
        let test_tags  = get_entry_tags(&test_entry).unwrap().unwrap();

        assert_eq!(*test_tags, tags_toml_value(vec![]));
        Ok(())
    }

}

