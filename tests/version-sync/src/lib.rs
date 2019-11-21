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

#[cfg(test)] extern crate version_sync;
#[cfg(test)] extern crate walkdir;
#[cfg(test)] extern crate env_logger;
#[cfg(test)] #[macro_use] extern crate log;
#[cfg(test)] extern crate toml_query;
#[cfg(test)] extern crate toml;

#[cfg(test)] use std::path::PathBuf;

#[cfg(test)]
fn setup_logging() {
    let _ = env_logger::try_init();
}

#[test]
fn test_readme() {
    version_sync::assert_markdown_deps_updated!("../../README.md");
}

#[test]
fn test_doc() {
    version_sync::assert_contains_regex!("../../doc/src/00000.md", "^version: {version}$");
    version_sync::assert_contains_regex!("../../doc/src/02000-store.md", "^version = \"{version}\"$");
    version_sync::assert_contains_regex!("../../doc/src/03020-writing-modules.md", "version = \"{version}\"");

    version_sync::assert_contains_regex!("../../doc/user/src/approach.md", "^version = \"{version}\"$");
}

#[test]
fn test_all_cargotoml_files() {
    use toml::Value;

    setup_logging();

    let current_version = env!("CARGO_PKG_VERSION");
    let imag_root = PathBuf::from(format!("{}/../../", env!("CARGO_MANIFEST_DIR")));
    println!("imag_root = {}", imag_root.display());

    walkdir::WalkDir::new(&imag_root)
        .follow_links(false)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed collecting files")
        .into_iter()
        .filter(|e| !e.path().to_str().unwrap().contains("target"))
        .filter_map(|element| if element.file_type().is_file() && element.path().ends_with("Cargo.toml") {
            debug!("Using = {:?}", element);
            Some(element.into_path())
        } else {
            debug!("Ignoring = {:?}", element);
            None
        })
        .for_each(|cargotoml| {
            let filecontent = std::fs::read_to_string(&cargotoml).expect(&format!("Failed to read {}", cargotoml.display()));
            let toml = filecontent.parse::<Value>().expect(&format!("Failed to parse toml: {}", cargotoml.display()));

            match toml.get("dependencies") {
                Some(Value::Table(ref tab)) => {
                    for (k, v) in tab.iter() {
                        if k.contains("libimag") {
                            match v {
                                Value::String(s) => assert!(s.contains(current_version)),
                                Value::Table(ref dep) => {
                                    let version = dep.get("version").expect(&format!("No 'version' key for dependencies at {}", cargotoml.display()));
                                    let version_str = version.as_str().unwrap();
                                    assert!(version_str.contains(current_version), "failed for: {} ('{}')", cargotoml.display(), version_str)
                                },
                                _ => unimplemented!(),
                            }
                        }
                    }
                },
                Some(_) => panic!("Dependencies is not a table?"),
                None => /* ignore if there is no "dependencies" */ {},
            }

            match toml.get("dev-dependencies") {
                Some(Value::Table(ref tab)) => {
                    for (k, v) in tab.iter() {
                        if k.contains("libimag") {
                            match v {
                                Value::String(s) => assert!(s.contains(current_version)),
                                Value::Table(ref dep) => {
                                    let version = dep.get("version").expect(&format!("No 'version' key for dev-dependencies at {}", cargotoml.display()));
                                    let version_str = version.as_str().unwrap();
                                    assert!(version_str.contains(current_version), "failed for: {} ('{}')", cargotoml.display(), version_str)
                                },
                                _ => unimplemented!(),
                            }
                        }
                    }
                },
                Some(_) => panic!("dev-dependencies is not a table?"),
                None => /* ignore if there is no "dependencies" */ {},
            }
        });
}
