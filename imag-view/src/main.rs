extern crate clap;
extern crate glob;
#[macro_use] extern crate log;
extern crate semver;
extern crate toml;
#[macro_use] extern crate version;

extern crate libimagrt;
extern crate libimagstore;
extern crate libimagutil;

use std::process::exit;

use clap::ArgMatches;

use libimagrt::runtime::Runtime;
use libimagstore::store::Entry;
use libimagstore::store::FileLockEntry;
use libimagstore::store::Result as StoreResult;
use libimagstore::storeid::StoreId;
use libimagutil::trace::trace_error;

mod ui;
mod viewer;

use ui::build_ui;
use viewer::Viewer;
use viewer::ViewInformation;
use viewer::stdout::StdoutViewer;

fn main() {
    let name = "imag-view";
    let version = &version!()[..];
    let about = "View entries (readonly)";
    let ui = build_ui(Runtime::get_default_cli_builder(name, version, about));
    let rt = {
        let rt = Runtime::new(ui);
        if rt.is_ok() {
            rt.unwrap()
        } else {
            println!("Could not set up Runtime");
            println!("{:?}", rt.err().unwrap());
            exit(1);
        }
    };

    rt.init_logger();

    debug!("Hello. Logging was just enabled");
    debug!("I already set up the Runtime object and build the commandline interface parser.");
    debug!("Lets get rollin' ...");

    info!("No implementation yet");

    let entry_id = rt.cli().value_of("id").unwrap(); // enforced by clap

    if rt.cli().is_present("versions") {
        view_versions_of(entry_id, &rt);
    } else {
        let entry_version   = rt.cli().value_of("version");
        let view_header     = rt.cli().is_present("view-header");
        let view_content    = rt.cli().is_present("view-content");
        let view_copy       = rt.cli().is_present("view-copy");
        let keep_copy       = rt.cli().is_present("keep-copy");

        let scmd = rt.cli().subcommand_matches("view-in");
        if scmd.is_none() {
            debug!("No commandline call");
            exit(1);
        }
        let scmd = scmd.unwrap();

        let viewer = build_viewer(scmd);
        let entry = load_entry(entry_id, entry_version, &rt);
        if entry.is_err() {
            trace_error(&entry.err().unwrap());
            exit(1);
        }
        let entry = entry.unwrap();

        let view_info = ViewInformation {
            entry:          entry,
            view_header:    view_header,
            view_content:   view_content,
            view_copy:      view_copy,
            keep_copy:      keep_copy,
        };

        let viewer = StdoutViewer::new();
        viewer.view(view_info);
    }
}

// TODO: This is a shameless adaption of imag-store/src/util.rs
fn load_entry<'a>(id: &str,
                  version: Option<&str>,
                  rt: &'a Runtime)
    -> StoreResult<FileLockEntry<'a>>
{
    use std::ops::Deref;

    debug!("Checking path element for version");

    let version = {
        version.unwrap_or_else(|| {
            id.split("~").last().unwrap_or_else(|| {
                warn!("No version");
                exit(1);
            })
        })
    };

    debug!("Building path from {:?} and {:?}", id, version);
    let mut path = rt.store().path().clone();

    if id.chars().next() == Some('/') {
        path.push(format!("{}~{}", &id[1..id.len()], version));
    } else {
        path.push(format!("{}~{}", id, version));
    }

    // the above is the adaption...

    rt.store().retrieve(path)
}

fn view_versions_of(id: &str, rt: &Runtime) {
    use glob::glob;

    let mut path = rt.store().path().clone();

    if id.chars().next() == Some('/') {
        path.push(format!("{}~*", &id[1..id.len()]));
    } else {
        path.push(format!("{}~*", id));
    }

    if let Some(path) = path.to_str() {
        match glob(path) {
            Ok(paths) =>
                for entry in paths {
                    match entry {
                        Ok(path) => println!("{}", path.file_name().and_then(|s| s.to_str()).unwrap()),
                        Err(e)   => trace_error(e.error()),
                    }
                },
            Err(e) => {
                warn!("{}", e); // trace_error(&e); // error seems not to be implemented
                debug!("Error in pattern");
                exit(1);
            },
        }
    } else {
        warn!("Could not build glob() argument!");
    }
}

