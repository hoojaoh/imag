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

use std::error::Error;

use notify_rust::NotificationUrgency;

use notificator::Notificator;
use self::err::*;
use self::ok::*;

mod err {
    use std::ops::Deref;
    use std::ops::DerefMut;
    use std::error::Error;

    use notify_rust::Notification as RustNotification;
    use notify_rust::NotificationUrgency;

    use notificator::default::Urgency;
    use notificator::default::Notification;
    use notificator::Notificator;

    #[derive(Debug, Default, Clone)]
    pub struct ErrorNotification(Notification, usize);

    impl ErrorNotification {
        pub fn new(trace: usize, timeout: i32) -> ErrorNotification {
            let notif = Notification {
                timeout: timeout,
                message: String::new(), // Not used in this special case
                summary: "[Error]".to_owned(),
                urgency: Urgency::High,
            };

            ErrorNotification(notif, trace)
        }
    }

    impl<T: Error> Notificator<T> for ErrorNotification {

        /// A default implementation for all Types that implement Display
        fn notify(&self, item: &T) {
            fn trace_notify(urgency: NotificationUrgency, e: &Error, u: usize) {
                let mut n = RustNotification::new();
                n.appname("imag");
                n.summary("[Error]");
                n.urgency(urgency.clone());
                n.body(e.description());
                n.finalize().show().ok(); // Ignoring error here

                if u > 0 {
                    e.cause().map(|cause| trace_notify(urgency, cause, u - 1));
                }
            }

            trace_notify(self.0.urgency.clone().into(), item, self.1);
        }

    }

    impl Deref for ErrorNotification {
        type Target = Notification;

        fn deref(&self) -> &Notification {
            &self.0
        }

    }

    impl DerefMut for ErrorNotification {

        fn deref_mut(&mut self) -> &mut Notification {
            &mut self.0
        }

    }

}

mod ok {
    use std::ops::Deref;
    use std::ops::DerefMut;
    use std::error::Error;
    use std::fmt::Display;

    use notify_rust::Notification as RustNotification;
    use notify_rust::NotificationUrgency;

    use notificator::default::Urgency;
    use notificator::default::Notification;
    use notificator::Notificator;

    #[derive(Debug, Default, Clone)]
    pub struct OkNotification(Notification);

    impl From<Notification> for OkNotification {

        fn from(n: Notification) -> OkNotification {
            OkNotification(n)
        }

    }

    impl<T> Notificator<T> for OkNotification {

        /// A default implementation for all Types that implement Display
        fn notify(&self, item: &T) {
            let mut n = RustNotification::new();
            n.appname("imag");
            n.summary("[Ok]");
            n.urgency(self.0.urgency.clone().into());
            n.body(&"< >".to_owned());
            n.finalize().show().ok(); // Ignoring error here
        }

    }

    impl Deref for OkNotification {
        type Target = Notification;

        fn deref(&self) -> &Notification {
            &self.0
        }

    }

    impl DerefMut for OkNotification {

        fn deref_mut(&mut self) -> &mut Notification {
            &mut self.0
        }

    }

}

pub trait ResultNotification<T, E> {

    fn notify_with(self, n: &Notificator<T>) -> Self;
    fn notify(self) -> Self;

    fn notify_on_err_with(self, n: &Notificator<E>) -> Self;
    fn notify_on_err(self) -> Self;

}

impl<T, E: Error> ResultNotification<T, E> for Result<T, E> {

    fn notify_with(self, n: &Notificator<T>) -> Self {
        self.map(|item| { n.notify(&item); item })
    }

    fn notify(self) -> Self {
        self.notify_with(&OkNotification::default())
    }

    fn notify_on_err_with(self, n: &Notificator<E>) -> Self {
        self.map_err(|e| { n.notify(&e); e })
    }

    fn notify_on_err(self) -> Self {
        self.notify_on_err_with(&ErrorNotification::default())
    }

}

