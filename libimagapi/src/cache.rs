//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015, 2016 Matthias Beyer <mail@beyermatthias.de> and contributors
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

use std::sync::Arc;
use std::sync::Mutex;
use std::collections::BTreeMap;
use std::ops::Deref;

use handle::Handle;

pub struct Cache<H: Handle, O>(Arc<Mutex<BTreeMap<H, O>>>);

impl<H: Handle, O> Cache<H, O> {

    pub fn new() -> Cache<H, O> {
        Cache(Arc::new(Mutex::new(BTreeMap::new())))
    }

}

impl<H: Handle, O> Deref for Cache<H, O> {
    type Target = Arc<Mutex<BTreeMap<H, O>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

