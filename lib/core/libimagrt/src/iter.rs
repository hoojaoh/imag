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

/// Iterator extension helpers for `Runtime::report_touched`
mod reporting {
    use std::ops::Deref;

    use libimagstore::store::Entry;
    use libimagstore::storeid::StoreId;
    use failure::Fallible as Result;
    use failure::Error;

    use runtime::Runtime;


    pub trait ReportTouchedEntry<'a, I, D>
        where I: Iterator<Item = D>,
              D: Deref<Target = Entry>,
    {
        fn map_report_touched(self, rt: &'a Runtime) -> ReportTouchedEntryImpl<'a, I, D>;
    }

    impl<'a, I, D> ReportTouchedEntry<'a, I, D> for I
        where I: Iterator<Item = D>,
              D: Deref<Target = Entry>,
    {
        fn map_report_touched(self, rt: &'a Runtime) -> ReportTouchedEntryImpl<'a, I, D> {
            ReportTouchedEntryImpl(self, rt)
        }
    }

    pub struct ReportTouchedEntryImpl<'a, I, D>(I, &'a Runtime<'a>)
        where I: Iterator<Item = D>,
              D: Deref<Target = Entry>;

    impl<'a, I, D> Iterator for ReportTouchedEntryImpl<'a, I, D>
        where I: Iterator<Item = D>,
              D: Deref<Target = Entry>,
    {
        type Item = Result<D>;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.next().map(|e| self.1.report_touched(e.get_location()).map_err(Error::from).map(|_| e))
        }
    }




    pub trait ReportTouchedStoreId<'a, I>
        where I: Iterator<Item = StoreId>
    {
        fn map_report_touched(self, rt: &'a Runtime) -> ReportTouchedStoreIdImpl<'a, I>;
    }

    impl<'a, I> ReportTouchedStoreId<'a, I> for I
        where I: Iterator<Item = StoreId>,
    {
        fn map_report_touched(self, rt: &'a Runtime) -> ReportTouchedStoreIdImpl<'a, I> {
            ReportTouchedStoreIdImpl(self, rt)
        }
    }

    pub struct ReportTouchedStoreIdImpl<'a, I>(I, &'a Runtime<'a>)
        where I: Iterator<Item = StoreId>;

    impl<'a, I> Iterator for ReportTouchedStoreIdImpl<'a, I>
        where I: Iterator<Item = StoreId>,
    {
        type Item = Result<StoreId>;

        fn next(&mut self) -> Option<Self::Item> {
            self.0
                .next()
                .map(|id| {
                    self.1
                        .report_touched(&id)
                        .map_err(Error::from)
                        .map(|_| id)
                })
        }
    }



    pub trait ReportTouchedResultStoreId<'a, I>
        where I: Iterator<Item = Result<StoreId>>
    {
        fn map_report_touched(self, rt: &'a Runtime) -> ReportTouchedResultStoreIdImpl<'a, I>;
    }

    impl<'a, I> ReportTouchedResultStoreId<'a, I> for I
        where I: Iterator<Item = Result<StoreId>>,
    {
        fn map_report_touched(self, rt: &'a Runtime) -> ReportTouchedResultStoreIdImpl<'a, I> {
            ReportTouchedResultStoreIdImpl(self, rt)
        }
    }

    pub struct ReportTouchedResultStoreIdImpl<'a, I>(I, &'a Runtime<'a>)
        where I: Iterator<Item = Result<StoreId>>;

    impl<'a, I> Iterator for ReportTouchedResultStoreIdImpl<'a, I>
        where I: Iterator<Item = Result<StoreId>>,
    {
        type Item = Result<StoreId>;

        fn next(&mut self) -> Option<Self::Item> {
            self.0
                .next()
                .map(|rid| {
                    rid.and_then(|id| {
                        self.1
                            .report_touched(&id)
                            .map_err(Error::from)
                            .map(|_| id)
                    })
                })
        }
    }
}

pub use self::reporting::*;
