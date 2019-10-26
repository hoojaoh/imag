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

#[macro_use] extern crate log;
extern crate clap;
extern crate url;
extern crate failure;
#[macro_use] extern crate prettytable;
#[cfg(test)] extern crate toml;
#[cfg(test)] extern crate toml_query;
#[cfg(test)] extern crate env_logger;

extern crate libimagentrylink;
extern crate libimagentryurl;
extern crate libimagrt;
extern crate libimagstore;
extern crate libimagerror;

#[cfg(test)]
#[macro_use]
extern crate libimagutil;

#[cfg(not(test))]
extern crate libimagutil;

use std::io::Write;
use std::path::PathBuf;


use failure::err_msg;

use libimagentryurl::linker::UrlLinker;
use libimagentrylink::linkable::Linkable;
use libimagentrylink::storecheck::StoreLinkConsistentExt;
use libimagerror::trace::{MapErrTrace, trace_error};
use libimagerror::exit::ExitUnwrap;
use libimagerror::io::ToExitCode;
use libimagrt::runtime::Runtime;
use libimagrt::application::ImagApplication;
use libimagstore::store::FileLockEntry;
use libimagstore::storeid::StoreId;
use libimagutil::warn_exit::warn_exit;
use libimagutil::warn_result::*;

use url::Url;
use failure::Fallible as Result;
use clap::App;

mod ui;

/// Marker enum for implementing ImagApplication on
///
/// This is used by binaries crates to execute business logic
/// or to build a CLI completion.
pub enum ImagLink {}
impl ImagApplication for ImagLink {
    fn run(rt: Runtime) -> Result<()> {
        if rt.cli().is_present("check-consistency") {
            let exit_code = match rt.store().check_link_consistency() {
                Ok(_) => {
                    info!("Store is consistent");
                    0
                }
                Err(e) => {
                    trace_error(&e);
                    1
                }
            };
            ::std::process::exit(exit_code);
        }

        let _ = rt.cli()
            .subcommand_name()
            .map(|name| {
                match name {
                    "remove" => remove_linking(&rt),
                    "unlink" => unlink(&rt),
                    "list"   => list_linkings(&rt),
                    other    => {
                        debug!("Unknown command");
                        let _ = rt.handle_unknown_subcommand("imag-link", other, rt.cli())
                            .map_err_trace_exit_unwrap()
                            .code()
                            .map(::std::process::exit);
                    },
                }
            })
            .or_else(|| {
                if let (Some(from), Some(to)) = (rt.cli().value_of("from"), rt.cli().values_of("to")) {
                    Some(link_from_to(&rt, from, to))
                } else {
                    warn_exit("No commandline call", 1)
                }
            })
            .ok_or_else(|| err_msg("No commandline call".to_owned()))
            .map_err_trace_exit_unwrap();

        Ok(())
    }

    fn build_cli<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
        ui::build_ui(app)
    }

    fn name() -> &'static str {
        env!("CARGO_PKG_NAME")
    }

    fn description() -> &'static str {
        "Link entries"
    }

    fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

fn get_entry_by_name<'a>(rt: &'a Runtime, name: &str) -> Result<Option<FileLockEntry<'a>>> {
    debug!("Getting: {:?}", name);
    let result = StoreId::new(PathBuf::from(name)).and_then(|id| rt.store().get(id));

    debug!(" => : {:?}", result);
    result
}

fn link_from_to<'a, I>(rt: &'a Runtime, from: &'a str, to: I)
    where I: Iterator<Item = &'a str>
{
    let mut from_entry = match get_entry_by_name(rt, from).map_err_trace_exit_unwrap() {
        Some(e) => e,
        None    => {
            debug!("No 'from' entry");
            warn_exit("No 'from' entry", 1)
        },
    };

    for entry in to {
        debug!("Handling 'to' entry: {:?}", entry);
        if rt.store().get(PathBuf::from(entry)).map_err_trace_exit_unwrap().is_none() {
            debug!("Linking externally: {:?} -> {:?}", from, entry);
            let url = Url::parse(entry).unwrap_or_else(|e| {
                error!("Error parsing URL: {:?}", e);
                ::std::process::exit(1);
            });

            let iter = from_entry
                .add_url(rt.store(), url)
                .map_err_trace_exit_unwrap()
                .into_iter();

            rt.report_all_touched(iter).unwrap_or_exit();
        } else {
            debug!("Linking internally: {:?} -> {:?}", from, entry);

            let from_id = StoreId::new(PathBuf::from(from)).map_err_trace_exit_unwrap();
            let entr_id = StoreId::new(PathBuf::from(entry)).map_err_trace_exit_unwrap();

            if from_id == entr_id {
                error!("Cannot link entry with itself. Exiting");
                ::std::process::exit(1)
            }

            let mut to_entry = match rt.store().get(entr_id).map_err_trace_exit_unwrap() {
                Some(e) => e,
                None    => {
                    warn!("No 'to' entry: {}", entry);
                    ::std::process::exit(1)
                },
            };
            from_entry
                .add_link(&mut to_entry)
                .map_err_trace_exit_unwrap();

            rt.report_touched(to_entry.get_location()).unwrap_or_exit();
        }


        info!("Ok: {} -> {}", from, entry);
    }

    rt.report_touched(from_entry.get_location()).unwrap_or_exit();
}

fn remove_linking(rt: &Runtime) {
    let mut from = rt.cli()
        .subcommand_matches("remove")
        .unwrap() // safe, we know there is an "remove" subcommand
        .value_of("from")
        .map(PathBuf::from)
        .map(|id| {
            rt.store()
                .get(id)
                .map_err_trace_exit_unwrap()
                .ok_or_else(|| warn_exit("No 'from' entry", 1))
                .unwrap() // safe by line above
        })
        .unwrap();

    rt
        .ids::<crate::ui::PathProvider>()
        .map_err_trace_exit_unwrap()
        .unwrap_or_else(|| {
            error!("No ids supplied");
            ::std::process::exit(1);
        })
        .into_iter()
        .for_each(|id| match rt.store().get(id.clone()) {
            Err(e) => trace_error(&e),
            Ok(Some(mut to_entry)) => {
                to_entry
                    .remove_link(&mut from)
                    .map_err_trace_exit_unwrap();

                rt.report_touched(to_entry.get_location()).unwrap_or_exit();
            },
            Ok(None) => {
                // looks like this is not an entry, but a filesystem URI and therefor an
                // external link...?
                if id.local().is_file() {
                    let pb = id.local().to_str().unwrap_or_else(|| {
                        warn!("Not StoreId and not a Path: {}", id);
                        ::std::process::exit(1);
                    });
                    let url = Url::parse(pb).unwrap_or_else(|e| {
                        error!("Error parsing URL: {:?}", e);
                        ::std::process::exit(1);
                    });
                    from.remove_url(rt.store(), url).map_err_trace_exit_unwrap();
                    info!("Ok: {}", id);
                } else {
                    warn!("Entry not found: {:?}", id);
                }
            }
        });

    rt.report_touched(from.get_location()).unwrap_or_exit();
}

fn unlink(rt: &Runtime) {
    rt
        .ids::<crate::ui::PathProvider>()
        .map_err_trace_exit_unwrap()
        .unwrap_or_else(|| {
            error!("No ids supplied");
            ::std::process::exit(1);
        })
        .into_iter()
        .for_each(|id| {
            rt.store()
                .get(id.clone())
                .map_err_trace_exit_unwrap()
                .unwrap_or_else(|| {
                    warn!("No entry for {}", id);
                    ::std::process::exit(1)
                })
                .unlink(rt.store())
                .map_err_trace_exit_unwrap();

            rt.report_touched(&id).unwrap_or_exit();
        });
}

fn list_linkings(rt: &Runtime) {
    let cmd = rt.cli()
        .subcommand_matches("list")
        .unwrap(); // safed by clap

    let list_externals  = cmd.is_present("list-externals-too");
    let list_plain      = cmd.is_present("list-plain");

    let mut tab = ::prettytable::Table::new();
    tab.set_titles(row!["#", "Link"]);

    rt
        .ids::<crate::ui::PathProvider>()
        .map_err_trace_exit_unwrap()
        .unwrap_or_else(|| {
            error!("No ids supplied");
            ::std::process::exit(1);
        })
        .into_iter()
        .for_each(|id| {
            match rt.store().get(id.clone()) {
                Ok(Some(entry)) => {
                    for (i, link) in entry.links().map_err_trace_exit_unwrap().enumerate() {
                        let link = link
                            .to_str()
                            .map_warn_err(|e| format!("Failed to convert StoreId to string: {:?}", e))
                            .ok();

                        if let Some(link) = link {
                            if list_plain {
                                writeln!(rt.stdout(), "{: <3}: {}", i, link)
                                    .to_exit_code()
                                    .unwrap_or_exit();
                            } else {
                                tab.add_row(row![i, link]);
                            }
                        }
                    }

                    if list_externals {
                        entry.get_urls(rt.store())
                            .map_err_trace_exit_unwrap()
                            .enumerate()
                            .for_each(|(i, link)| {
                                let link = link
                                    .map_err_trace_exit_unwrap()
                                    .into_string();

                                if list_plain {
                                    writeln!(rt.stdout(), "{: <3}: {}", i, link)
                                        .to_exit_code()
                                        .unwrap_or_exit();
                                } else {
                                    tab.add_row(row![i, link]);
                                }
                            })
                    }

                    rt.report_touched(entry.get_location()).unwrap_or_exit();

                },
                Ok(None)        => warn!("Not found: {}", id),
                Err(e)          => trace_error(&e),
            }

            rt.report_touched(&id).unwrap_or_exit();
        });

    if !list_plain {
        let out      = rt.stdout();
        let mut lock = out.lock();
        tab.print(&mut lock)
            .to_exit_code()
            .unwrap_or_exit();
    }
}

#[cfg(test)]
mod tests {
    use super::link_from_to;
    use super::remove_linking;

    use std::path::PathBuf;
    use std::ffi::OsStr;

    use toml::value::Value;
    use toml_query::read::TomlValueReadExt;
    use failure::Fallible as Result;
    use failure::Error;

    use libimagrt::runtime::Runtime;
    use libimagstore::storeid::StoreId;
    use libimagstore::store::{FileLockEntry, Entry};

    fn setup_logging() {
        let _ = ::env_logger::try_init();
    }

    make_mock_app! {
        app "imag-link";
        modulename mock;
        version env!("CARGO_PKG_VERSION");
        with help "imag-link mocking app";
    }
    use self::mock::generate_test_runtime;
    use self::mock::reset_test_runtime;

    fn create_test_default_entry<'a, S: AsRef<OsStr>>(rt: &'a Runtime, name: S) -> Result<StoreId> {
        let mut path = PathBuf::new();
        path.set_file_name(name);

        let default_entry = Entry::new(StoreId::new(PathBuf::from("")).unwrap())
            .to_str()
            .unwrap();

        debug!("Default entry constructed");

        let id = StoreId::new(path)?;
        debug!("StoreId constructed: {:?}", id);

        let mut entry = rt.store().create(id.clone())?;

        debug!("Entry constructed: {:?}", id);
        entry.get_content_mut().push_str(&default_entry);

        Ok(id)
    }

    fn get_entry_links<'a>(entry: &'a FileLockEntry<'a>) -> Result<&'a Value> {
        match entry.get_header().read(&"links.internal".to_owned()).map_err(Error::from)? {
            Some(v) => Ok(v),
            None    => panic!("Didn't find 'links' in {:?}", entry),
        }
    }

    fn links_toml_value<I: IntoIterator<Item = &'static str>>(links: I) -> Value {
        Value::Array(links
                         .into_iter()
                         .map(|s| Value::String(s.to_owned()))
                         .collect())
    }

    #[test]
    fn test_link_modificates() {
        setup_logging();
        let rt = generate_test_runtime(vec!["test1", "test2"])
            .unwrap();

        debug!("Runtime created");

        let test_id1 = create_test_default_entry(&rt, "test1").unwrap();
        let test_id2 = create_test_default_entry(&rt, "test2").unwrap();

        debug!("Entries created");

        link_from_to(&rt, "test1", vec!["test2"].into_iter());

        debug!("Linking done");

        let test_entry1 = rt.store().get(test_id1).unwrap().unwrap();
        let test_links1 = get_entry_links(&test_entry1).unwrap();

        let test_entry2 = rt.store().get(test_id2).unwrap().unwrap();
        let test_links2 = get_entry_links(&test_entry2).unwrap();

        debug!("Asserting");

        assert_ne!(*test_links1, links_toml_value(vec![]));
        assert_ne!(*test_links2, links_toml_value(vec![]));

        debug!("Test finished")
    }

    #[test]
    fn test_linking_links() {
        setup_logging();
        let rt = generate_test_runtime(vec!["test1", "test2"])
            .unwrap();

        debug!("Runtime created");

        let test_id1 = create_test_default_entry(&rt, "test1").unwrap();
        let test_id2 = create_test_default_entry(&rt, "test2").unwrap();

        debug!("Test entries created");

        link_from_to(&rt, "test1", vec!["test2"].into_iter());

        debug!("Linking done");

        let test_entry1 = rt.store().get(test_id1).unwrap().unwrap();
        let test_links1 = get_entry_links(&test_entry1).unwrap();

        let test_entry2 = rt.store().get(test_id2).unwrap().unwrap();
        let test_links2 = get_entry_links(&test_entry2).unwrap();

        debug!("Asserting");

        assert_eq!(*test_links1, links_toml_value(vec!["test2"]));
        assert_eq!(*test_links2, links_toml_value(vec!["test1"]));
    }

    #[test]
    fn test_multilinking() {
        setup_logging();
        let rt = generate_test_runtime(vec!["test1", "test2"])
            .unwrap();

        debug!("Runtime created");

        let test_id1 = create_test_default_entry(&rt, "test1").unwrap();
        let test_id2 = create_test_default_entry(&rt, "test2").unwrap();

        debug!("Test entries created");

        link_from_to(&rt, "test1", vec!["test2"].into_iter());
        link_from_to(&rt, "test1", vec!["test2"].into_iter());

        debug!("Linking done");

        let test_entry1 = rt.store().get(test_id1).unwrap().unwrap();
        let test_links1 = get_entry_links(&test_entry1).unwrap();

        let test_entry2 = rt.store().get(test_id2).unwrap().unwrap();
        let test_links2 = get_entry_links(&test_entry2).unwrap();

        debug!("Asserting");

        assert_eq!(*test_links1, links_toml_value(vec!["test2"]));
        assert_eq!(*test_links2, links_toml_value(vec!["test1"]));
    }

    #[test]
    fn test_linking_more_than_two() {
        setup_logging();
        let rt = generate_test_runtime(vec!["test1", "test2", "test3"])
            .unwrap();

        debug!("Runtime created");

        let test_id1 = create_test_default_entry(&rt, "test1").unwrap();
        let test_id2 = create_test_default_entry(&rt, "test2").unwrap();
        let test_id3 = create_test_default_entry(&rt, "test3").unwrap();

        debug!("Test entries created");

        link_from_to(&rt, "test1", vec!["test2", "test3"].into_iter());
        link_from_to(&rt, "test1", vec!["test2", "test3"].into_iter());

        debug!("Linking done");

        let test_entry1 = rt.store().get(test_id1).unwrap().unwrap();
        let test_links1 = get_entry_links(&test_entry1).unwrap();

        let test_entry2 = rt.store().get(test_id2).unwrap().unwrap();
        let test_links2 = get_entry_links(&test_entry2).unwrap();

        let test_entry3 = rt.store().get(test_id3).unwrap().unwrap();
        let test_links3 = get_entry_links(&test_entry3).unwrap();

        debug!("Asserting");

        assert_eq!(*test_links1, links_toml_value(vec!["test2", "test3"]));
        assert_eq!(*test_links2, links_toml_value(vec!["test1"]));
        assert_eq!(*test_links3, links_toml_value(vec!["test1"]));
    }

    // Remove tests

    #[test]
    fn test_linking_links_unlinking_removes_links() {
        setup_logging();
        let rt = generate_test_runtime(vec!["test1", "test2"])
            .unwrap();

        debug!("Runtime created");

        let test_id1 = create_test_default_entry(&rt, "test1").unwrap();
        let test_id2 = create_test_default_entry(&rt, "test2").unwrap();

        debug!("Test entries created");

        link_from_to(&rt, "test1", vec!["test2"].into_iter());

        debug!("Linking done");

        let rt = reset_test_runtime(vec!["remove", "test1", "test2"], rt)
            .unwrap();

        remove_linking(&rt);

        debug!("Linking removed");

        let test_entry1 = rt.store().get(test_id1).unwrap().unwrap();
        let test_links1 = get_entry_links(&test_entry1).unwrap();

        let test_entry2 = rt.store().get(test_id2).unwrap().unwrap();
        let test_links2 = get_entry_links(&test_entry2).unwrap();

        debug!("Asserting");

        assert_eq!(*test_links1, links_toml_value(vec![]));
        assert_eq!(*test_links2, links_toml_value(vec![]));
    }

    #[test]
    fn test_linking_and_unlinking_more_than_two() {
        setup_logging();
        let rt = generate_test_runtime(vec!["test1", "test2", "test3"])
            .unwrap();

        debug!("Runtime created");

        let test_id1 = create_test_default_entry(&rt, "test1").unwrap();
        let test_id2 = create_test_default_entry(&rt, "test2").unwrap();
        let test_id3 = create_test_default_entry(&rt, "test3").unwrap();

        debug!("Test entries created");

        link_from_to(&rt, "test1", vec!["test2", "test3"].into_iter());

        debug!("linking done");

        let rt = reset_test_runtime(vec!["remove", "test1", "test2", "test3"], rt)
            .unwrap();

        remove_linking(&rt);

        debug!("linking removed");

        let test_entry1 = rt.store().get(test_id1).unwrap().unwrap();
        let test_links1 = get_entry_links(&test_entry1).unwrap();

        let test_entry2 = rt.store().get(test_id2).unwrap().unwrap();
        let test_links2 = get_entry_links(&test_entry2).unwrap();

        let test_entry3 = rt.store().get(test_id3).unwrap().unwrap();
        let test_links3 = get_entry_links(&test_entry3).unwrap();

        debug!("Asserting");

        assert_eq!(*test_links1, links_toml_value(vec![]));
        assert_eq!(*test_links2, links_toml_value(vec![]));
        assert_eq!(*test_links3, links_toml_value(vec![]));
    }
}
