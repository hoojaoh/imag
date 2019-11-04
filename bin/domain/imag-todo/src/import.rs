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

use failure::Fallible as Result;
use failure::err_msg;

use libimagrt::runtime::Runtime;

pub fn import(rt: &Runtime) -> Result<()> {
    let scmd      = rt.cli().subcommand().1.unwrap();

    match scmd.subcommand_name() {
        None                => Err(err_msg("No subcommand called")),
        Some("taskwarrior") => import_taskwarrior(rt),
        Some(other)         => {
            debug!("Unknown command");
            if rt.handle_unknown_subcommand("imag-todo-import", other, rt.cli())?.success() {
                Ok(())
            } else {
                Err(err_msg("Failed to handle unknown subcommand"))
            }
        },
    }
}

#[allow(unused_variables)]
fn import_taskwarrior(rt: &Runtime) -> Result<()> {
    #[cfg(not(feature = "import-taskwarrior"))]
    {
        Err(err_msg("Binary not compiled with taskwarrior import functionality"))
    }

    #[cfg(feature = "import-taskwarrior")]
    {
        use std::collections::HashMap;
        use std::ops::Deref;

        use uuid::Uuid;

        use libimagtodo::status::Status;
        use libimagtodo::priority::Priority;
        use libimagtodo::store::TodoStore;
        use libimagentrytag::tagable::Tagable;
        use libimagentrylink::linkable::Linkable;

        use task_hookrs::import::import as taskwarrior_import;
        use task_hookrs::priority::TaskPriority;
        use task_hookrs::status::TaskStatus;

        let store = rt.store();
        if !rt.input_is_pipe() {
            return Err(err_msg("Cannot get stdin for importing tasks"))
        }
        let stdin = ::std::io::stdin();

        let translate_status = |twstatus: &TaskStatus| -> Option<Status> {
            match *twstatus {
                TaskStatus::Pending => Some(Status::Pending),
                TaskStatus::Completed => Some(Status::Done),
                TaskStatus::Deleted => Some(Status::Deleted),
                _ => Some(Status::Deleted), // default to deleted if taskwarrior data does not have a status
            }
        };

        let translate_prio = |p: &TaskPriority| -> Priority {
            match p {
                TaskPriority::Low    => Priority::Low,
                TaskPriority::Medium => Priority::Medium,
                TaskPriority::High   => Priority::High,
            }
        };

        taskwarrior_import(stdin)?
            .into_iter()
            .map(|task| {
                let hash = task.uuid().clone();
                let mut todo = store
                    .todo_builder()
                    .with_check_sanity(false) // data should be imported, even if it is not sane
                    .with_status(translate_status(task.status()))
                    .with_uuid(Some(task.uuid().clone()))
                    .with_due(task.due().map(Deref::deref).cloned())
                    .with_scheduled(task.scheduled().map(Deref::deref).cloned())
                    .with_hidden(task.wait().map(Deref::deref).cloned())
                    .with_prio(task.priority().map(|p| translate_prio(p)))
                    .build(rt.store())?;

                todo.set_content(task.description().clone());

                if let Some(tags) = task.tags() {
                    tags.into_iter().map(|tag| {
                        let tag = tag.clone();
                        if libimagentrytag::tag::is_tag_str(&tag).is_err() {
                            warn!("Not a valid tag, ignoring: {}", tag);
                            Ok(())
                        } else {
                            todo.add_tag(tag)
                        }
                    }).collect::<Result<Vec<_>>>()?;
                }

                if let Some(annos) = task.annotations() {
                    // We do not import annotations as imag annotations, but add them as text to
                    // the entry, which is more sane IMO.
                    //
                    // this could be changed into a configurable thing later.
                    let anno = annos.iter()
                        .map(|anno| anno.description())
                        .map(String::clone)
                        .collect::<Vec<String>>()
                        .join("\n");
                    todo.get_content_mut().push('\n');
                    todo.get_content_mut().push_str(&anno);
                }

                let dependends = task.depends().cloned().unwrap_or_else(|| vec![]);
                Ok((hash, dependends))
            })
            .collect::<Result<HashMap<Uuid, Vec<Uuid>>>>()?

            //
            // We actually _have_ to collect here, because we must ensure that all imported Todo
            // entries are in the store before we can continue and link them together (which is
            // what happens next)
            //

            .iter()
            .filter(|(_, list)| !list.is_empty())
            .map(|(key, list)| {
                let mut entry = store.get_todo_by_uuid(key)?.ok_or_else(|| {
                    format_err!("Cannot find todo by UUID: {}", key)
                })?;

                list.iter()
                    .map(move |element| {
                        store.get_todo_by_uuid(element)?
                            .ok_or_else(|| {
                                format_err!("Cannot find todo by UUID: {}", key)
                            })
                            .and_then(|mut target| entry.add_link(&mut target))
                    })
                    .collect::<Result<_>>()
            })
            .collect::<Result<_>>()
    }
}
