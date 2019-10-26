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
extern crate filters;
#[macro_use] extern crate log;
extern crate toml;
extern crate toml_query;
extern crate failure;

#[cfg(test)]
extern crate env_logger;

extern crate libimagerror;
extern crate libimagstore;
#[macro_use] extern crate libimagrt;

use std::io::Write;

use filters::filter::Filter;
use failure::Fallible as Result;
use failure::Error;
use failure::err_msg;

use libimagstore::storeid::StoreId;
use libimagrt::setup::generate_runtime_setup;

mod ui;

pub struct IsInCollectionsFilter<'a, A>(Option<A>, ::std::marker::PhantomData<&'a str>)
    where A: AsRef<[&'a str]>;

impl<'a, A> IsInCollectionsFilter<'a, A>
    where A: AsRef<[&'a str]>
{
    pub fn new(collections: Option<A>) -> Self {
        IsInCollectionsFilter(collections, ::std::marker::PhantomData)
    }
}

impl<'a, A> Filter<StoreId> for IsInCollectionsFilter<'a, A>
    where A: AsRef<[&'a str]> + 'a
{
    fn filter(&self, sid: &StoreId) -> bool {
        match self.0 {
            Some(ref colls) => sid.is_in_collection(colls),
            None => true,
        }
    }
}


fn main() -> Result<()> {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-id-in-collection",
                                    &version,
                                    "filter ids by collection",
                                    crate::ui::build_ui);
    let values = rt
        .cli()
        .values_of("in-collection-filter")
        .map(|v| v.collect::<Vec<&str>>());

    let collection_filter = IsInCollectionsFilter::new(values);

    let mut stdout = rt.stdout();
    trace!("Got output: {:?}", stdout);

    rt.ids::<crate::ui::PathProvider>()?
        .ok_or_else(|| err_msg("No ids supplied"))?
        .iter()
        .filter(|id| collection_filter.filter(id))
        .map(|id| {
            if !rt.output_is_pipe() {
                let id = id.to_str()?;
                trace!("Writing to {:?}", stdout);
                writeln!(stdout, "{}", id)?;
            }

            rt.report_touched(&id).map_err(Error::from)
        })
        .collect()
}

