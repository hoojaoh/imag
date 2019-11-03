//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2019 the imag contributors
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

extern crate assert_cmd;
extern crate assert_fs;
extern crate env_logger;
extern crate predicates;
extern crate semver;
#[macro_use] extern crate log;
#[macro_use] extern crate pretty_assertions;

#[cfg(test)] mod imag;
#[cfg(test)] mod imag_annotate;
#[cfg(test)] mod imag_category;
#[cfg(test)] mod imag_create;
#[cfg(test)] mod imag_diagnostics;
#[cfg(test)] mod imag_edit;
#[cfg(test)] mod imag_git;
#[cfg(test)] mod imag_gps;
#[cfg(test)] mod imag_grep;
#[cfg(test)] mod imag_header;
#[cfg(test)] mod imag_id_in_collection;
#[cfg(test)] mod imag_ids;
#[cfg(test)] mod imag_init;
#[cfg(test)] mod imag_link;
#[cfg(test)] mod imag_markdown;
#[cfg(test)] mod imag_mv;
#[cfg(test)] mod imag_ref;
#[cfg(test)] mod imag_store;
#[cfg(test)] mod imag_tag;
#[cfg(test)] mod imag_view;
#[cfg(test)] mod imag_bookmark;
#[cfg(test)] mod imag_calendar;
#[cfg(test)] mod imag_contact;
#[cfg(test)] mod imag_diary;
#[cfg(test)] mod imag_habit;
#[cfg(test)] mod imag_log;
#[cfg(test)] mod imag_mail;
#[cfg(test)] mod imag_notes;
#[cfg(test)] mod imag_timetrack;
#[cfg(test)] mod imag_todo;
#[cfg(test)] mod imag_wiki;

static LOG_SYNC: std::sync::Once = std::sync::Once::new();

pub fn setup_logging() {
    LOG_SYNC.call_once(|| { let _ = env_logger::try_init(); });
}

