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
    dead_code,
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

//! External linking is a complex implementation to be able to serve a clean and easy-to-use
//! interface.
//!
//! Internally, there are no such things as "external links" (plural). Each Entry in the store can
//! only have _one_ external link.
//!
//! This library does the following therefor: It allows you to have several external links with one
//! entry, which are internally one file in the store for each link, linked with "internal
//! linking".
//!
//! This helps us greatly with deduplication of URLs.
//!

extern crate itertools;
#[macro_use] extern crate log;
extern crate toml;
extern crate toml_query;
extern crate url;
extern crate sha1;
extern crate hex;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate failure;

#[cfg(test)]
extern crate env_logger;

#[macro_use] extern crate libimagstore;
extern crate libimagerror;
extern crate libimagutil;
extern crate libimagentrylink;

module_entry_path_mod!("url");

pub mod iter;
pub mod link;
pub mod linker;
pub mod util;

