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
#[macro_use] extern crate log;
#[macro_use] extern crate failure;
extern crate toml_query;
#[macro_use] extern crate indoc;

#[macro_use] extern crate libimagrt;
extern crate libimagmail;
extern crate libimagerror;
extern crate libimagstore;
extern crate libimagutil;
extern crate libimagentryref;

use std::io::Write;
use std::path::PathBuf;

use failure::Fallible as Result;
use toml_query::read::TomlValueReadTypeExt;

use libimagerror::trace::{MapErrTrace, trace_error};
use libimagerror::iter::TraceIterator;
use libimagerror::exit::ExitUnwrap;
use libimagerror::io::ToExitCode;
use libimagmail::mail::Mail;
use libimagmail::store::MailStore;
use libimagmail::util;
use libimagentryref::reference::{Ref, RefFassade};
use libimagentryref::util::get_ref_config;
use libimagrt::runtime::Runtime;
use libimagrt::setup::generate_runtime_setup;
use libimagutil::info_result::*;
use libimagstore::store::FileLockEntry;
use libimagstore::storeid::StoreIdIterator;
use libimagstore::iter::get::StoreIdGetIteratorExtension;

mod ui;

use crate::ui::build_ui;

fn main() {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-mail",
                                    &version,
                                    "Mail collection tool",
                                    build_ui);

    rt.cli()
        .subcommand_name()
        .map(|name| {
            debug!("Call {}", name);
            match name {
                "import-mail" => import_mail(&rt),
                "list"        => list(&rt),
                "mail-store"  => mail_store(&rt),
                other         => {
                    debug!("Unknown command");
                    let _ = rt.handle_unknown_subcommand("imag-mail", other, rt.cli())
                        .map_err_trace_exit_unwrap()
                        .code()
                        .map(::std::process::exit);
                }
            }
        });
}

fn import_mail(rt: &Runtime) {
    let collection_name = get_ref_collection_name(rt).map_err_trace_exit_unwrap();
    let refconfig       = get_ref_config(rt, "imag-mail").map_err_trace_exit_unwrap();
    let scmd            = rt.cli().subcommand_matches("import-mail").unwrap();
    let store           = rt.store();

    debug!(r#"Importing mail with
    collection_name = {}
    refconfig = {:?}
    "#, collection_name, refconfig);

    scmd.values_of("path")
        .unwrap() // enforced by clap
        .map(PathBuf::from)
        .map(|path| {
            if scmd.is_present("ignore_existing_ids") {
                store.retrieve_mail_from_path(path, &collection_name, &refconfig)
            } else {
                store.create_mail_from_path(path, &collection_name, &refconfig)
            }
            .map_info_str("Ok")
            .map_err_trace_exit_unwrap()
        })
        .for_each(|entry| rt.report_touched(entry.get_location()).unwrap_or_exit());
}

fn list(rt: &Runtime) {
    let refconfig       = get_ref_config(rt, "imag-mail").map_err_trace_exit_unwrap();
    let scmd            = rt.cli().subcommand_matches("list").unwrap(); // safe via clap
    let print_content   = scmd.is_present("list-read");

    if print_content {
        // TODO: Check whether workaround with "{}" is still necessary when updating "indoc"
        warn!("{}", indoc!(r#"You requested to print the content of the mail as well.
        We use the 'mailparse' crate underneath, but its implementation is nonoptimal.
        Thus, the content might be printed as empty (no text in the email)
        This is not reliable and might be wrong."#));

        // TODO: Fix above.
    }

    // TODO: Implement lister type in libimagmail for this
    //
    // Optimization: Pass refconfig here instead of call get_ref_config() in lister function. This
    // way we do not call get_ref_config() multiple times.
    fn list_mail<'a>(rt: &Runtime,
                     refconfig: &::libimagentryref::reference::Config,
                     m: &FileLockEntry<'a>,
                     print_content: bool) {

        let id = match m.get_message_id(&refconfig) {
            Ok(Some(f)) => f,
            Ok(None) => "<no id>".to_owned(),
            Err(e) => {
                trace_error(&e);
                "<error>".to_owned()
            },
        };

        let from = match m.get_from(&refconfig) {
            Ok(Some(f)) => f,
            Ok(None) => "<no from>".to_owned(),
            Err(e) => {
                trace_error(&e);
                "<error>".to_owned()
            },
        };

        let to = match m.get_to(&refconfig) {
            Ok(Some(f)) => f,
            Ok(None) => "<no to>".to_owned(),
            Err(e) => {
                trace_error(&e);
                "<error>".to_owned()
            },
        };

        let subject = match m.get_subject(&refconfig) {
            Ok(Some(f)) => f,
            Ok(None) => "<no subject>".to_owned(),
            Err(e) => {
                trace_error(&e);
                "<error>".to_owned()
            },
        };

        if print_content {
            use libimagmail::hasher::MailHasher;

            let content = m.as_ref_with_hasher::<MailHasher>()
                .get_path(&refconfig)
                .and_then(util::get_mail_text_content)
                .map_err_trace_exit_unwrap();

            writeln!(rt.stdout(),
                     "Mail: {id}\nFrom: {from}\nTo: {to}\n{subj}\n---\n{content}\n---\n",
                     from    = from,
                     id      = id,
                     subj    = subject,
                     to      = to,
                     content = content
            ).to_exit_code().unwrap_or_exit();
        } else {
            writeln!(rt.stdout(),
                     "Mail: {id}\nFrom: {from}\nTo: {to}\n{subj}\n",
                     from = from,
                     id   = id,
                     subj = subject,
                     to   = to
            ).to_exit_code().unwrap_or_exit();
        }

        rt.report_touched(m.get_location()).unwrap_or_exit();
    }

    if rt.ids_from_stdin() {
        let iter = rt
            .ids::<crate::ui::PathProvider>()
            .map_err_trace_exit_unwrap()
            .unwrap_or_else(|| {
                error!("No ids supplied");
                ::std::process::exit(1);
            })
            .into_iter()
            .map(Ok);

        StoreIdIterator::new(Box::new(iter))
    } else {
        rt.store()
            .all_mails()
            .map_err_trace_exit_unwrap()
            .into_storeid_iter()
    }
    .map(|id| { debug!("Found: {:?}", id); id })
    .into_get_iter(rt.store())
    .trace_unwrap_exit()
    .filter_map(|e| e)
    .for_each(|m| list_mail(&rt, &refconfig, &m, print_content));
}

fn mail_store(rt: &Runtime) {
    let _ = rt.cli().subcommand_matches("mail-store").unwrap();
    error!("This feature is currently not implemented.");
    unimplemented!()
}

fn get_ref_collection_name(rt: &Runtime) -> Result<String> {
    let setting_name = "mail.ref_collection_name";

    debug!("Getting configuration: {}", setting_name);

    rt.config()
        .ok_or_else(|| format_err!("No configuration, cannot find collection name for mail collection"))?
        .read_string(setting_name)?
        .ok_or_else(|| format_err!("Setting missing: {}", setting_name))
}

