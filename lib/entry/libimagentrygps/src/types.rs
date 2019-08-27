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

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use toml::Value;
use toml::map::Map;
use failure::Fallible as Result;
use failure::err_msg;
use failure::Error;

use libimagerror::errors::ErrorMsg as EM;

pub trait FromValue : Sized {
    fn from_value(v: &Value) -> Result<Self>;
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct GPSValue {
    pub degree:  i64,
    pub minutes: i64,
    pub seconds: i64,
}

impl GPSValue {

    pub fn new(d: i64, m: i64, s: i64) -> GPSValue {
        GPSValue {
            degree:  d,
            minutes: m,
            seconds: s
        }
    }

    pub fn degree(&self) -> i64 {
        self.degree
    }

    pub fn minutes(&self) -> i64 {
        self.minutes
    }

    pub fn seconds(&self) -> i64 {
        self.seconds
    }

}


impl Into<Value> for GPSValue {

    fn into(self) -> Value {
        let mut map = Map::new();
        let _ = map.insert("degree".to_owned(),  Value::Integer(self.degree));
        let _ = map.insert("minutes".to_owned(), Value::Integer(self.minutes));
        let _ = map.insert("seconds".to_owned(), Value::Integer(self.seconds));
        Value::Table(map)
    }

}

impl FromValue for GPSValue {
    fn from_value(v: &Value) -> Result<Self> {
        let int_to_appropriate_width = |v: &Value| {
            v.as_integer()
             .ok_or_else(|| Error::from(EM::EntryHeaderTypeError))
        };

        match *v {
            Value::Table(ref map) => {
                Ok(GPSValue::new(
                    map.get("degree")
                        .ok_or_else(|| err_msg("Degree missing"))
                        .and_then(&int_to_appropriate_width)?,

                    map
                        .get("minutes")
                        .ok_or_else(|| err_msg("Minutes missing"))
                        .and_then(&int_to_appropriate_width)?,

                    map
                        .get("seconds")
                        .ok_or_else(|| err_msg("Seconds missing"))
                        .and_then(&int_to_appropriate_width)?
                ))
            }
            _ => Err(Error::from(EM::EntryHeaderTypeError))
        }
    }

}

impl Display for GPSValue {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}° {}\" {}'", self.degree, self.minutes, self.seconds)
    }
}

/// Data-transfer type for transfering longitude-latitude-pairs
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Coordinates {
    pub longitude: GPSValue,
    pub latitude:  GPSValue,
}

impl Coordinates {
    pub fn new(long: GPSValue, lat: GPSValue) -> Coordinates {
        Coordinates {
            longitude: long,
            latitude:  lat,
        }
    }

    pub fn longitude(&self) -> &GPSValue {
        &self.longitude
    }

    pub fn latitude(&self) -> &GPSValue {
        &self.latitude
    }
}

impl Into<Value> for Coordinates {

    fn into(self) -> Value {
        let mut map = Map::new();
        let _ = map.insert("longitude".to_owned(), self.longitude.into());
        let _ = map.insert("latitude".to_owned(), self.latitude.into());
        Value::Table(map)
    }

}

impl FromValue for Coordinates {
    fn from_value(v: &Value) -> Result<Self> {
        v.as_table()
            .ok_or_else(|| Error::from(EM::EntryHeaderTypeError))
            .and_then(|t| {
                let get = |m: &Map<_, _>, what: &'static str, ek| -> Result<GPSValue> {
                    m.get(what)
                        .ok_or_else(|| err_msg(ek))
                        .and_then(GPSValue::from_value)
                };

                Ok(Coordinates::new(
                    get(t, "longitude", "Longitude missing")?,
                    get(t, "latitude", "Latitude missing")?
                ))
            })
    }

}

impl Display for Coordinates {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "longitude = {}\nlatitude = {}", self.longitude, self.latitude)
    }
}


