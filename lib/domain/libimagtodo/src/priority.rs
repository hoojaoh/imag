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

use std::cmp::PartialOrd;
use std::cmp::Ordering;

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Priority {
    #[serde(rename = "h")]
    High,

    #[serde(rename = "m")]
    Medium,

    #[serde(rename = "l")]
    Low,
}

impl Priority {
    pub fn as_str(&self) -> &str {
        match self {
            Priority::High => "h",
            Priority::Medium => "m",
            Priority::Low => "l",
        }
    }
}

impl PartialOrd for Priority {
    fn partial_cmp(&self, other: &Priority) -> Option<Ordering> {
        Some(match (self, other) {
            (Priority::Low,    Priority::Low)    => Ordering::Equal,
            (Priority::Low,    Priority::Medium) => Ordering::Less,
            (Priority::Low,    Priority::High)   => Ordering::Less,

            (Priority::Medium, Priority::Low)    => Ordering::Greater,
            (Priority::Medium, Priority::Medium) => Ordering::Equal,
            (Priority::Medium, Priority::High)   => Ordering::Less,

            (Priority::High,   Priority::Low)    => Ordering::Greater,
            (Priority::High,   Priority::Medium) => Ordering::Greater,
            (Priority::High,   Priority::High)   => Ordering::Equal,
        })
    }
}

impl Ord for Priority {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap() // save by impl above
    }
}

