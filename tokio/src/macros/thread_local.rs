#[cfg(all(loom, test))]
macro_rules! tokio_thread_local {
    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty = const { $expr:expr } $(;)?) => {
        loom::thread_local! {
            $(#[$attrs])*
            $vis static $name: $ty = $expr;
        }
    };

    ($($tts:tt)+) => { loom::thread_local!{ $($tts)+ } }
}

#[cfg(all(not(all(loom, test)), not(feature = "external-tls")))]
macro_rules! tokio_thread_local {
    ($($tts:tt)+) => {
        ::std::thread_local!{ $($tts)+ }
    }
}

#[cfg(all(not(all(loom, test)), feature = "external-tls"))]
macro_rules! tokio_thread_local {
    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty = const { $expr:expr } $(;)?) => {
        tokio_thread_local! {
            $(#[$attrs])*
            $vis static $name: $ty = $expr;
        }
    };

    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty = $expr:expr $(;)?) => {
        pub static $name: ::std::thread::LocalKey<$ty> = {
            extern "C" {
                #[link_name = concat!(stringify!($name), "_THUNK")]
                #[allow(improper_ctypes)]
                static mut THUNK: fn(Option<&mut Option<$ty>>) -> *const $ty;
            }

            unsafe {
                /// same layout as `LocalKey` in std, this means we don't need to
                /// opt into the `thread_local_internals` feature
                struct MyLocalKey<T: 'static> {
                    #[allow(unused)]
                    inner: fn(Option<&mut Option<T>>) -> *const T,
                }

                std::mem::transmute(MyLocalKey {
                    // the closure is NOT redundant, apparently $name_INNER_ADDR gets devirtualized
                    // if it's not there.
                    #[allow(clippy::redundant_closure)]
                    inner: |n| {
                        THUNK(n)
                    },
                })
            }
        };
    };
}
