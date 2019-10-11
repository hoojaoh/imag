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

use std::collections::BTreeMap;

use clap::ArgMatches;
use vobject::icalendar::ICalendar;
use vobject::icalendar::Event;
use handlebars::Handlebars;
use failure::Fallible as Result;
use failure::Error;
use failure::err_msg;
use toml_query::read::TomlValueReadTypeExt;
use chrono::NaiveDateTime;

use libimagrt::runtime::Runtime;
use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;
use libimagentryref::reference::fassade::RefFassade;
use libimagentryref::reference::Ref;
use libimagentryref::reference::Config;
use libimagentryref::hasher::default::DefaultHasher;
use libimagerror::trace::MapErrTrace;
use crate::libimagcalendar::store::EventStore;

#[derive(Debug)]
pub struct ParsedEventFLE<'a> {
    inner: FileLockEntry<'a>,
    data: ICalendar,
}

impl<'a> ParsedEventFLE<'a> {

    /// Because libimagcalendar only links to the actual calendar data, we need to read the data and
    /// parse it.
    /// With this function, a FileLockEntry can be parsed to a ParsedEventFileLockEntry
    /// (ParsedEventFLE).
    pub fn parse(fle: FileLockEntry<'a>, refconfig: &Config) -> Result<Self> {
        fle.as_ref_with_hasher::<DefaultHasher>()
            .get_path(refconfig)
            .and_then(|p| ::std::fs::read_to_string(p).map_err(Error::from))
            .and_then(|s| ICalendar::build(&s).map_err(Error::from))
            .map(|cal| ParsedEventFLE {
                inner: fle,
                data: cal,
            })
    }

    pub fn get_entry(&self) -> &FileLockEntry<'a> {
        &self.inner
    }

    pub fn get_data(&self) -> &ICalendar {
        &self.data
    }
}

pub fn get_event_print_format(config_value_path: &'static str, rt: &Runtime, scmd: &ArgMatches)
    -> Result<Handlebars>
{
    scmd.value_of("format")
        .map(String::from)
        .map(Ok)
        .unwrap_or_else(|| {
            rt.config()
                .ok_or_else(|| err_msg("No configuration file"))?
                .read_string(config_value_path)?
                .ok_or_else(|| err_msg("Configuration 'contact.list_format' does not exist"))
        })
        .and_then(|fmt| {
            let mut hb = Handlebars::new();
            hb.register_template_string("format", fmt)?;

            hb.register_escape_fn(::handlebars::no_escape);
            ::libimaginteraction::format::register_all_color_helpers(&mut hb);
            ::libimaginteraction::format::register_all_format_helpers(&mut hb);

            Ok(hb)
        })
}

pub fn build_data_object_for_handlebars<'a>(i: usize, event: &Event<'a>)
    -> BTreeMap<&'static str, String>
{
    macro_rules! process_opt {
        ($t:expr, $text:expr) => {
            ($t).map(|obj| obj.into_raw()).unwrap_or_else(|| String::from($text))
        }
    }

    let mut data = BTreeMap::new();

    data.insert("i"           , format!("{}", i));
    data.insert("dtend"       , process_opt!(event.dtend()       , "<no dtend>"));
    data.insert("dtstart"     , process_opt!(event.dtstart()     , "<no dtstart>"));
    data.insert("dtstamp"     , process_opt!(event.dtstamp()     , "<no dtstamp>"));
    data.insert("uid"         , process_opt!(event.uid()         , "<no uid>"));
    data.insert("description" , process_opt!(event.description() , "<no description>"));
    data.insert("summary"     , process_opt!(event.summary()     , "<no summary>"));
    data.insert("url"         , process_opt!(event.url()         , "<no url>"));
    data.insert("location"    , process_opt!(event.location()    , "<no location>"));
    data.insert("class"       , process_opt!(event.class()       , "<no class>"));
    data.insert("categories"  , process_opt!(event.categories()  , "<no categories>"));
    data.insert("transp"      , process_opt!(event.transp()      , "<no transp>"));
    data.insert("rrule"       , process_opt!(event.rrule()       , "<no rrule>"));

    data
}

pub fn kairos_parse(spec: &str) -> Result<NaiveDateTime> {
    match ::kairos::parser::parse(spec).map_err_trace_exit_unwrap() {
        ::kairos::parser::Parsed::Iterator(_) => {
            trace!("before-filter spec resulted in iterator");
            Err(format_err!("Not a moment in time: {}", spec))
        }

        ::kairos::parser::Parsed::TimeType(tt) => {
            trace!("before-filter spec resulted in timetype");
            let tt = tt.calculate()
                .map_err_trace_exit_unwrap()
                .get_moment().unwrap_or_else(|| {
                    error!("Not a moment in time: {}", spec);
                    ::std::process::exit(1);
                })
                .clone();

            trace!("Before filter spec {} => {}", spec, tt);
            Ok(tt)
        }
    }
}

pub fn find_event_by_id<'a>(store: &'a Store, id: &str, refconfig: &Config) -> Result<Option<ParsedEventFLE<'a>>> {
    if let Some(entry) = store.get_event_by_uid(id)? {
        debug!("Found directly: {} -> {}", id, entry.get_location());
        return ParsedEventFLE::parse(entry, refconfig).map(Some)
    }

    for sid in store.all_events()? {
        let sid = sid?;

        let event = store.get(sid.clone())?.ok_or_else(|| {
            format_err!("Cannot get {}, which should be there.", sid)
        })?;

        trace!("Checking whether {} is represented by {}", id, event.get_location());
        let parsed = ParsedEventFLE::parse(event, refconfig)?;

        if parsed
            .get_data()
            .events()
            .filter_map(|event| if event
                .as_ref()
                .map(|e| {
                    trace!("Checking whether {:?} starts with {}", e.uid(), id);
                    e.uid().map(|uid| uid.raw().starts_with(id)).unwrap_or(false)
                })
                .unwrap_or(false)
            {
                trace!("Seems to be relevant");
                Some(event)
            } else {
                None
            })
            .next()
            .is_some()
        {
            return Ok(Some(parsed))
        }
    }

    Ok(None)
}

