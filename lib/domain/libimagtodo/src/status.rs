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

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Status {
    #[serde(rename = "pending")]
    Pending,

    #[serde(rename = "done")]
    Done,

    #[serde(rename = "deleted")]
    Deleted,
}

impl Status {
    pub fn as_str(&self) -> &str {
        match self {
            Status::Pending => "pending",
            Status::Done    => "done",
            Status::Deleted => "deleted",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "pending" => Ok(Status::Pending),
            "done"    => Ok(Status::Done),
            "deleted" => Ok(Status::Deleted),
            other     => Err(format_err!("{} is not a valid status", other)),
        }
    }
}

#[test]
fn test_serializing() {
    assert_eq!(Status::Pending.as_str(), "pending");
    assert_eq!(Status::Done.as_str(), "done");
    assert_eq!(Status::Deleted.as_str(), "deleted");
}

#[test]
fn test_deserializing() {
    assert_eq!(Status::from_str("pending").unwrap(), Status::Pending);
    assert_eq!(Status::from_str("done").unwrap(), Status::Done);
    assert_eq!(Status::from_str("deleted").unwrap(), Status::Deleted);
}

#[test]
fn test_serializing_deserializing() {
    assert_eq!(Status::Pending.as_str(), Status::from_str("pending").unwrap().as_str());
    assert_eq!(Status::Done.as_str(), Status::from_str("done").unwrap().as_str());
    assert_eq!(Status::Deleted.as_str(), Status::from_str("deleted").unwrap().as_str());
}
