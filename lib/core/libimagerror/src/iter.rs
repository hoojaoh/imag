//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2019 the imag contributors
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

use failure::Error;

/// An iterator that unwraps the `Ok` items of `iter`, while passing the `Err` items to its
/// closure `f`.
///
/// This `struct` is created by the `unwrap_with()` method on `TraceIterator`. See its
/// documentation for more information.
#[must_use = "iterator adaptors are lazy and do nothing unless consumed"]
#[derive(Clone)]
pub struct UnwrapWith<I, F>{
    iter: I,
    f: F
}

impl<I, F, T> Iterator for UnwrapWith<I, F>
    where
        I: Iterator<Item = Result<T, Error>>,
        F: FnMut(Error)
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                Some(Err(e))   => { (self.f)(e); },
                Some(Ok(item)) => return Some(item),
                None           => return None,
            }
        }
    }
}



/// Iterator helper for Unwrap with exiting on error
pub struct UnwrapExit<I, T>(I)
    where I: Iterator<Item = Result<T, Error>>;

impl<I, T> Iterator for UnwrapExit<I, T>
    where I: Iterator<Item = Result<T, Error>>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        use crate::trace::MapErrTrace;
        self.0.next().map(|e| e.map_err_trace_exit_unwrap())
    }
}

impl<I, T> DoubleEndedIterator for UnwrapExit<I, T>
    where I: DoubleEndedIterator<Item = Result<T, Error>>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        use crate::trace::MapErrTrace;
        self.0.next_back().map(|e| e.map_err_trace_exit_unwrap())
    }
}

/// This trait provides methods that make it easier to work with iterators that yield a `Result`.
pub trait TraceIterator<T> : Iterator<Item = Result<T, Error>> + Sized {
    /// Creates an iterator that yields the item in each `Ok` item, while filtering out the
    /// `Err`
    /// items. Each filtered `Err` will be trace-logged with [`::trace::trace_error`].
    ///
    /// As with all iterators, the processing is lazy. If you do not use the result of this method,
    /// nothing will be passed to `::trace::trace_error`, no matter how many `Err` items might
    /// be present.
    #[inline]
    fn trace_unwrap(self) -> UnwrapWith<Self, fn(Error)> {
        #[inline]
        fn trace_error(err: Error) {
            err.iter_chain().for_each(|cause| {
                eprintln!("{}", cause);
            });
        }

        self.unwrap_with(trace_error)
    }

    /// Creates an iterator that yields the item in each `Ok` item.
    ///
    /// The first `Err(_)` element is traced using `::trace::trace_error_exit`.
    ///
    /// As with all iterators, the processing is lazy. If you do not use the result of this method,
    /// nothing will be passed to `::trace::trace_error_exit`, no matter how many `Err` items might
    /// be present.
    #[inline]
    fn trace_unwrap_exit(self) -> UnwrapExit<Self, T> {
        UnwrapExit(self)
    }


    /// Takes a closure and creates an iterator that will yield the items inside all `Ok` items
    /// yielded by the original iterator. All `Err` items will be filtered out, and the contents
    /// of each `Err` will be passed to the closure.
    ///
    /// As with all iterators, the processing is lazy. The result of this method must be evaluated
    /// for the closure to be called.
    #[inline]
    fn unwrap_with<F>(self, f: F) -> UnwrapWith<Self, F>
        where F: FnMut(Error)
    {
        UnwrapWith { iter: self, f }
    }
}

impl<I, T> TraceIterator<T> for I where
    I: Iterator<Item = Result<T, Error>>
{}

