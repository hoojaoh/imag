extern crate clap;
extern crate glob;
#[macro_use] extern crate log;
extern crate serde_json;
extern crate semver;
extern crate toml;
#[macro_use] extern crate version;

extern crate task_hookrs;

extern crate libimagrt;
extern crate libimagstore;
extern crate libimagutil;
extern crate libimagtodo;

use std::process::exit;
use std::process::{Command, Stdio};
use std::io::stdin;
use std::io::BufRead;

use task_hookrs::import::{import, import_task, import_tasks};

use libimagrt::runtime::Runtime;
use libimagtodo::task::IntoTask;
use libimagutil::trace::trace_error;

mod ui;

use ui::build_ui;
fn main() {

    let name = "imag-todo";
    let version = &version!()[..];
    let about = "Interface with taskwarrior";
    let ui = build_ui(Runtime::get_default_cli_builder(name, version, about));

    let rt = {
        let rt = Runtime::new(ui);
        if rt.is_ok() {
            rt.unwrap()
        } else {
            println!("Could not set up Runtime");
            println!("{:?}", rt.unwrap_err());
            exit(1);
        }
    };



    let scmd = rt.cli().subcommand_name();
    match scmd {
        Some("tw-hook") => {
            let subcmd = rt.cli().subcommand_matches("tw-hook").unwrap();
            if subcmd.is_present("add") {
                let stdin = stdin();
                let mut stdin = stdin.lock();
                let mut line = String::new();
                match stdin.read_line(&mut line) {
                    Ok(_) => { }
                    Err(e) => {
                        error!("{}", e);
                        return;
                    }
                };
                if let Ok(ttask) = import_task(&line.as_str()) {
                    let uuid = *ttask.uuid();
                    println!("{}", match serde_json::ser::to_string(&ttask) {
                        Ok(val) => val,
                        Err(e) => {
                            error!("{}", e);
                            return;
                        }
                    });
                    match ttask.into_filelockentry(rt.store()) {
                        Ok(val) => {
                            println!("Task {} stored in imag", uuid);
                            val
                        },
                        Err(e) => {
                            trace_error(&e);
                            error!("{}", e);
                            return;
                        }
                    };
                }
                else {
                    error!("No usable input");
                    return;
                }
            }
            else if subcmd.is_present("delete") {
                // The used hook is "on-modify". This hook gives two json-objects
                // per usage und wants one (the second one) back.
                let mut counter = 0;
                let stdin = stdin();
                let mut stdin = stdin.lock();
                if let Ok(ttasks) = import_tasks(stdin) {
                    for ttask in ttasks {
                        if counter % 2 == 1 {
                            // Only every second task is needed, the first one is the
                            // task before the change, and the second one after
                            // the change. The (maybe modified) second one is
                            // expected by taskwarrior.
                            println!("{}", match serde_json::ser::to_string(&ttask) {
                                Ok(val) => val,
                                Err(e) => {
                                    error!("{}", e);
                                    return;
                                }
                            });
                            match ttask.status() {
                                &task_hookrs::status::TaskStatus::Deleted => {
                                    match libimagtodo::delete::delete(rt.store(), *ttask.uuid()) {
                                        Ok(_) => {
                                            println!("Deleted task {}", *ttask.uuid());
                                        }
                                        Err(e) => {
                                            trace_error(&e);
                                            error!("{}", e);
                                            return;
                                        }
                                    }
                                }
                                _ => {
                                }
                            } // end match ttask.status()
                        } // end if c % 2
                        counter += 1;
                    } // end for
                } // end if let
                else {
                    error!("No usable input");
                }
            }
            else {
                // Should not be possible, as one argument is required via
                // ArgGroup
                unreachable!();
            }
        },
        Some("exec") => {
            let subcmd = rt.cli().subcommand_matches("exec").unwrap();
            let mut args = Vec::new();
            if let Some(exec_string) = subcmd.values_of("command") {
                for e in exec_string {
                    args.push(e);
                }
                let tw_process = Command::new("task").stdin(Stdio::null()).args(&args).spawn().unwrap_or_else(|e| {
                    panic!("failed to execute taskwarrior: {}", e);
                });

                let output = tw_process.wait_with_output().unwrap_or_else(|e| {
                    panic!("failed to unwrap output: {}", e);
                });
                let outstring = String::from_utf8(output.stdout).unwrap_or_else(|e| {
                    panic!("failed to ececute: {}", e);
                });
                println!("{}", outstring);
            } else {
                panic!("faild to execute: You need to exec --command");
            }
        }
        Some("list") => {
            let subcmd = rt.cli().subcommand_matches("list").unwrap();
            let mut args = Vec::new();
            let verbose = subcmd.is_present("verbose");
            let iter = match libimagtodo::read::get_todo_iterator(rt.store()) {
                //let iter = match rt.store().retrieve_for_module("todo/taskwarrior") {
                Err(e) => {
                    error!("{}", e);
                    return;
                }
                Ok(val) => val,
            };
            for task in iter {
                match task {
                    Ok(val) => {
                        //let val = libimagtodo::task::Task::new(fle);
                        //println!("{:#?}", val.flentry);
                        let uuid = match val.flentry.get_header().read("todo.uuid") {
                            Ok(Some(u)) => u,
                            Ok(None) => continue,
                            Err(e) => {
                                error!("{}", e);
                                continue;
                            }
                        };
                        if verbose {
                            args.clear();
                            args.push(format!("uuid:{}", uuid));
                            args.push(format!("{}", "information"));
                            let tw_process = Command::new("task").stdin(Stdio::null()).args(&args).spawn()
                                .unwrap_or_else(|e| {
                                    error!("{}", e);
                                    panic!("failed");
                                });
                            let output = tw_process.wait_with_output().unwrap_or_else(|e| {
                                panic!("failed to unwrap output: {}", e);
                            });
                            let outstring = String::from_utf8(output.stdout).unwrap_or_else(|e| {
                                panic!("failed to ececute: {}", e);
                            });
                            println!("{}", outstring);
                        }
                        else {
                            println!("{}", match uuid {
                                toml::Value::String(s) => s,
                                _ => {
                                    error!("Unexpected type for todo.uuid: {}", uuid);
                                    continue;
                                },
                            });
                        }
                    }
                    Err(e) => {
                        error!("{}", e);
                        continue;
                    }
                } // end match task
            } // end for
        }
        _ => unimplemented!(),
    } // end match scmd
} // end main

