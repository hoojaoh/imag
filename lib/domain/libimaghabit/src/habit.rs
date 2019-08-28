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

use toml::Value;
use toml_query::read::TomlValueReadTypeExt;
use toml_query::insert::TomlValueInsertExt;
use chrono::NaiveDateTime;
use chrono::Local;
use chrono::NaiveDate;
use failure::Error;
use failure::Fallible as Result;
use failure::ResultExt;
use failure::err_msg;

use crate::iter::HabitInstanceStoreIdIterator;
use crate::util::IsHabitCheck;
use crate::util::get_string_header_from_entry;
use crate::instance::IsHabitInstance;

use libimagentrylink::linkable::Linkable;
use libimagstore::store::Store;
use libimagstore::store::FileLockEntry;
use libimagstore::store::Entry;
use libimagstore::storeid::StoreId;
use libimagstore::storeid::StoreIdIterator;
use libimagentryutil::isa::Is;
use libimagentryutil::isa::IsKindHeaderPathProvider;
use libimagutil::date::date_to_string;

/// A HabitTemplate is a "template" of a habit. A user may define a habit "Eat vegetable".
/// If the user ate a vegetable, she should create a HabitInstance from the Habit with the
/// appropriate date (and optionally a comment) set.
pub trait HabitTemplate : Sized {

    /// Create an instance from this habit template
    ///
    /// By default creates an instance with the name of the template, the current time and the
    /// current date and copies the comment from the template to the instance.
    ///
    /// It uses `Store::retrieve()` underneath. So if there is already an instance for the day
    /// passed, this will simply return the instance.
    fn create_instance_with_date<'a>(&mut self, store: &'a Store, date: NaiveDate)
        -> Result<FileLockEntry<'a>>;

    /// Shortcut for calling `Self::create_instance_with_date()` with an instance of
    /// `::chrono::Local::today().naive_local()`.
    fn create_instance_today<'a>(&mut self, store: &'a Store) -> Result<FileLockEntry<'a>>;

    /// Same as `HabitTemplate::create_instance_with_date()` but uses `Store::retrieve` internally.
    fn retrieve_instance_with_date<'a>(&mut self, store: &'a Store, date: NaiveDate)
        -> Result<FileLockEntry<'a>>;

    /// Same as `HabitTemplate::create_instance_today()` but uses `Store::retrieve` internally.
    fn retrieve_instance_today<'a>(&mut self, store: &'a Store) -> Result<FileLockEntry<'a>>;

    /// Get instances for this template
    fn linked_instances(&self) -> Result<HabitInstanceStoreIdIterator>;

    /// Get the date of the next date when the habit should be done
    fn next_instance_date_after(&self, base: &NaiveDateTime) -> Result<Option<NaiveDate>>;

    /// Get the date of the next date when the habit should be done
    fn next_instance_date(&self) -> Result<Option<NaiveDate>>;

    /// Check whether the instance is a habit by checking its headers for the habit data
    fn is_habit_template(&self) -> Result<bool>;

    fn habit_name(&self) -> Result<String>;
    fn habit_basedate(&self) -> Result<String>;
    fn habit_recur_spec(&self) -> Result<String>;
    fn habit_comment(&self) -> Result<String>;
    fn habit_until_date(&self) -> Result<Option<String>>;

    fn instance_exists_for_date(&self, date: NaiveDate) -> Result<bool>;

    /// Create a StoreId for a habit name and a date the habit should be instantiated for
    fn instance_id_for(habit_name: &str, habit_date: NaiveDate) -> Result<StoreId>;
}

provide_kindflag_path!(pub IsHabitTemplate, "habit.template.is_habit_template");

impl HabitTemplate for Entry {

    fn create_instance_with_date<'a>(&mut self, store: &'a Store, date: NaiveDate) -> Result<FileLockEntry<'a>> {
        let name    = self.habit_name()?;
        let date    = date_to_string(date);
        let id      = instance_id_for_name_and_datestr(&name, &date)?;

        store.create(id)
            .map_err(From::from)
            .and_then(|entry| postprocess_instance(entry, name, date, self))
    }

    fn create_instance_today<'a>(&mut self, store: &'a Store) -> Result<FileLockEntry<'a>> {
        self.create_instance_with_date(store, Local::today().naive_local())
    }

    fn retrieve_instance_with_date<'a>(&mut self, store: &'a Store, date: NaiveDate) -> Result<FileLockEntry<'a>> {
        let name    = self.habit_name()?;
        let date    = date_to_string(date);
        let id      = instance_id_for_name_and_datestr(&name, &date)?;

        store.retrieve(id)
            .map_err(From::from)
            .and_then(|entry| postprocess_instance(entry, name, date, self))
    }

    fn retrieve_instance_today<'a>(&mut self, store: &'a Store) -> Result<FileLockEntry<'a>> {
        self.retrieve_instance_with_date(store, Local::today().naive_local())
    }

    fn linked_instances(&self) -> Result<HabitInstanceStoreIdIterator> {
        let iter = self
            .links()?
            .map(|link| link.get_store_id().clone())
            .filter(IsHabitCheck::is_habit_instance)
            .map(Ok);

        let sidi = StoreIdIterator::new(Box::new(iter));
        Ok(HabitInstanceStoreIdIterator::new(sidi))
    }

    fn next_instance_date_after(&self, base: &NaiveDateTime) -> Result<Option<NaiveDate>> {
        use kairos::timetype::TimeType;
        use kairos::parser::parse;
        use kairos::parser::Parsed;
        use kairos::iter::extensions::Every;

        let date_from_s = |r: String| -> Result<TimeType> {
            match parse(&r)? {
                Parsed::TimeType(tt) => Ok(tt),
                Parsed::Iterator(_) => {
                    Err(format_err!("'{}' yields an iterator. Cannot use.", r))
                },
            }
        };

        debug!("Base is {:?}", base);

        let basedate  = date_from_s(self.habit_basedate()?)?;
        debug!("Basedate is {:?}", basedate);

        let increment = date_from_s(self.habit_recur_spec()?)?;
        debug!("Increment is {:?}", increment);

        let until = self.habit_until_date()?.map(|s| -> Result<_> {
            date_from_s(s)?
                .calculate()?
                .get_moment()
                .map(Clone::clone)
                .ok_or_else(|| err_msg("until-date seems to have non-date value"))
        });

        debug!("Until-Date is {:?}", basedate);

        for element in basedate.every(increment)? {
            debug!("Calculating: {:?}", element);
            let element = element?.calculate()?;
            debug!(" = {:?}", element);
            if let Some(ndt) = element.get_moment() {
                if ndt >= base {
                    debug!("-> {:?} >= {:?}", ndt, base);
                    if let Some(u) = until {
                        if *ndt > u? {
                            return Ok(None);
                        } else {
                            return Ok(Some(ndt.date()));
                        }
                    } else {
                        return Ok(Some(ndt.date()));
                    }
                }
            } else {
                return Err(err_msg("Iterator seems to return bogus values."));
            }
        }

        unreachable!() // until we have habit-end-date support
    }

    /// Get the date of the next date when the habit should be done
    fn next_instance_date(&self) -> Result<Option<NaiveDate>> {
        use kairos::timetype::TimeType;

        let today = TimeType::today();
        let today = today.get_moment().unwrap(); // we know this is safe.
        debug!("Today is {:?}", today);

        self.next_instance_date_after(&today.date().and_hms(0, 0, 0))
    }

    /// Check whether the instance is a habit by checking its headers for the habit data
    fn is_habit_template(&self) -> Result<bool> {
        self.is::<IsHabitTemplate>().map_err(From::from)
    }

    fn habit_name(&self) -> Result<String> {
        get_string_header_from_entry(self, "habit.template.name")
    }

    fn habit_basedate(&self) -> Result<String> {
        get_string_header_from_entry(self, "habit.template.basedate")
    }

    fn habit_recur_spec(&self) -> Result<String> {
        get_string_header_from_entry(self, "habit.template.recurspec")
    }

    fn habit_comment(&self) -> Result<String> {
        get_string_header_from_entry(self, "habit.template.comment")
    }

    fn habit_until_date(&self) -> Result<Option<String>> {
        self.get_header()
            .read_string("habit.template.until")
            .map_err(From::from)
            .map(|os| os.map(String::from))
    }

    fn instance_exists_for_date(&self, date: NaiveDate) -> Result<bool> {
        let name = self.habit_name()?;
        let date = date_to_string(date);

        for link in self.links()? {
            let sid         = link.get_store_id();
            let instance_id = instance_id_for_name_and_datestr(&name, &date)?;

            if sid.is_habit_instance() && instance_id == *sid {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn instance_id_for(habit_name: &str, habit_date: NaiveDate) -> Result<StoreId> {
        instance_id_for_name_and_datestr(habit_name, &date_to_string(habit_date))
    }

}

fn instance_id_for_name_and_datestr(habit_name: &str, habit_date: &str) -> Result<StoreId> {
    crate::module_path::new_id(format!("instance/{}-{}", habit_name, habit_date))
        .context(format_err!("Failed building ID for instance: habit name = {}, habit date = {}", habit_name, habit_date))
        .map_err(Error::from)
}

pub mod builder {
    use toml::Value;
    use toml_query::insert::TomlValueInsertExt;
    use chrono::NaiveDate;

    use libimagstore::store::Store;
    use libimagstore::storeid::StoreId;
    use libimagstore::store::FileLockEntry;
    use libimagentryutil::isa::Is;
    use libimagutil::debug_result::DebugResult;

    use failure::Error;
    use failure::Fallible as Result;
    use failure::err_msg;

    use libimagutil::date::date_to_string;
    use crate::habit::IsHabitTemplate;

    #[derive(Debug)]
    pub struct HabitBuilder {
        name: Option<String>,
        comment: Option<String>,
        basedate: Option<NaiveDate>,
        recurspec: Option<String>,
        untildate: Option<NaiveDate>,
    }

    impl HabitBuilder {

        pub fn with_name(mut self, name: String) -> Self {
            self.name = Some(name);
            self
        }

        pub fn with_comment(mut self, comment: String) -> Self {
            self.comment = Some(comment);
            self
        }

        pub fn with_basedate(mut self, date: NaiveDate) -> Self {
            self.basedate = Some(date);
            self
        }

        pub fn with_recurspec(mut self, spec: String) -> Self {
            self.recurspec = Some(spec);
            self
        }

        pub fn with_until(mut self, date: NaiveDate) -> Self {
            self.untildate = Some(date);
            self
        }

        pub fn build<'a>(self, store: &'a Store) -> Result<FileLockEntry<'a>> {
            #[inline]
            fn mkerr(s: &'static str) -> Error {
                format_err!("Habit builder missing: {}", s)
            }

            let name = self.name
                .ok_or_else(|| mkerr("name"))
                .map_dbg_str("Success: Name present")?;

            let dateobj = self.basedate
                .ok_or_else(|| mkerr("date"))
                .map_dbg_str("Success: Date present")?;

            let recur : String = self.recurspec
                .ok_or_else(|| mkerr("recurspec"))
                .map_dbg_str("Success: Recurr spec present")?;

            if let Some(until) = self.untildate {
                debug!("Success: Until-Date present");
                if dateobj > until {
                    let e = err_msg("Habit builder logic error: until-date before start date");
                    return Err(e);
                }
            }

            if let Err(e) = ::kairos::parser::parse(&recur).map_err(Error::from) {
                debug!("Kairos failed: {:?}", e);
                return Err(e)
            }
            let date      = date_to_string(dateobj);
            debug!("Success: Date valid");

            let comment   = self.comment.unwrap_or_else(String::new);
            let sid       = build_habit_template_sid(&name)?;

            debug!("Creating entry in store for: {:?}", sid);
            let mut entry = store.create(sid)?;

            entry.set_isflag::<IsHabitTemplate>()?;
            {
                let h = entry.get_header_mut();
                let _ = h.insert("habit.template.name", Value::String(name))?;
                let _ = h.insert("habit.template.basedate", Value::String(date))?;
                let _ = h.insert("habit.template.recurspec", Value::String(recur))?;
                let _ = h.insert("habit.template.comment", Value::String(comment))?;
            }

            if let Some(until) = self.untildate {
                let until = date_to_string(until);
                entry.get_header_mut().insert("habit.template.until", Value::String(until))?;
            }

            debug!("Success: Created entry in store and set headers");
            Ok(entry)
        }

    }

    impl Default for HabitBuilder {
        fn default() -> Self {
            HabitBuilder {
                name: None,
                comment: None,
                basedate: None,
                recurspec: None,
                untildate: None,
            }
        }
    }

    /// Buld a StoreId for a Habit from a date object and a name of a habit
    fn build_habit_template_sid(name: &str) -> Result<StoreId> {
        crate::module_path::new_id(format!("template/{}", name)).map_err(From::from)
    }

}

fn postprocess_instance<'a>(mut entry: FileLockEntry<'a>,
                            name: String,
                            date: String,
                            template: &mut Entry)
    -> Result<FileLockEntry<'a>>
{
    {
        entry.set_isflag::<IsHabitInstance>()?;
        let hdr = entry.get_header_mut();
        let _   = hdr.insert("habit.instance.name",    Value::String(name))?;
        let _   = hdr.insert("habit.instance.date",    Value::String(date))?;
    }

    entry.add_link(template)?;

    Ok(entry)
}

