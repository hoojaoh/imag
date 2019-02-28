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

use libimagstore::store::Entry;

use crate::builtin::header::field_path::FieldPath;
use crate::builtin::header::field_predicate::FieldPredicate;
use crate::builtin::header::field_predicate::Predicate;
use filters::failable::filter::FailableFilter;

use failure::Fallible as Result;
use failure::Error;

use toml::Value;

struct EqPred {
    expected: Value
}

impl Predicate for EqPred {

    fn evaluate(&self, v: &Value) -> bool {
        self.expected == *v
    }

}

/// Check whether certain header field in a entry is equal to a value
pub struct FieldEq {
    filter: FieldPredicate<EqPred>,
}

impl FieldEq {

    pub fn new(path: FieldPath, expected_value: Value) -> FieldEq {
        FieldEq {
            filter: FieldPredicate::new(path, Box::new(EqPred { expected: expected_value })),
        }
    }

}

impl FailableFilter<Entry> for FieldEq {
    type Error = Error;

    fn filter(&self, e: &Entry) -> Result<bool> {
        self.filter.filter(e)
    }

}

