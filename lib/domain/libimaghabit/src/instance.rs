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

use std::ops::Deref;

use chrono::NaiveDate;
use toml::Value;
use toml_query::set::TomlValueSetExt;
use failure::Fallible as Result;

use crate::util::*;
use crate::habit::HabitTemplate;

use libimagstore::store::Entry;
use libimagstore::store::Store;
use libimagentryutil::isa::Is;
use libimagentryutil::isa::IsKindHeaderPathProvider;
use libimagentrylink::linkable::Linkable;

/// An instance of a habit is created for each time a habit is done.
///
/// # Note
///
/// A habit is a daily thing, so we only provide "date" as granularity for its time data.
///
pub trait HabitInstance {
    /// Check whether the instance is a habit instance by checking its headers for the habit
    /// data
    fn is_habit_instance(&self) -> Result<bool>;

    fn get_date(&self) -> Result<NaiveDate>;
    fn set_date(&mut self, n: NaiveDate) -> Result<()>;
    fn get_comment(&self, store: &Store) -> Result<String>;
    fn get_template_name(&self) -> Result<String>;
}

provide_kindflag_path!(pub IsHabitInstance, "habit.instance.is_habit_instance");

impl HabitInstance for Entry {
    fn is_habit_instance(&self) -> Result<bool> {
        self.is::<IsHabitInstance>().map_err(From::from)
    }

    fn get_date(&self) -> Result<NaiveDate> {
        use libimagutil::date::date_from_string as dts;
        let date_from_string = |d| dts(d).map_err(From::from);
        get_string_header_from_entry(self, "habit.instance.date").and_then(date_from_string)
    }

    fn set_date(&mut self, n: NaiveDate) -> Result<()> {
        use libimagutil::date::date_to_string;
        // Using `set` here because when creating the entry, these headers should be made present.
        self.get_header_mut()
            .set("habit.instance.date", Value::String(date_to_string(n)))
            .map_err(From::from)
            .map(|_| ())
    }

    /// Iterates all internal links, finds the template for this instance and gets the comment from
    /// it
    ///
    ///
    /// # Warning
    ///
    /// Internally tries to `Store::get()` the template entry. If this entry is borrowed outside of
    /// this function, this fails.
    ///
    /// If multiple templates are linked to this entry, this returns the comment of the first
    ///
    ///
    /// # Return
    ///
    /// Returns the Comment string from the first template that is linked to this instance.
    /// If this is not an instance, this might misbehave.
    /// If there is no template linked, this returns an error.
    ///
    fn get_comment(&self, store: &Store) -> Result<String> {
        let templ_name = self.get_template_name()?;
        for link in self.links()? {
            let template = store.get(link.get_store_id().clone())?.ok_or_else(|| {
                format_err!("Entry {} is linked to {}, but that entry does not exist",
                            self.get_location(),
                            link.get_store_id())
            })?;
            if HabitTemplate::is_habit_template(template.deref())? && template.habit_name()? == templ_name {
                return template.habit_comment()
            }
        }
        Err(format_err!("Cannot find template entry for {}", self.get_location()))
    }

    fn get_template_name(&self) -> Result<String> {
        get_string_header_from_entry(self, "habit.instance.name")
    }

}
