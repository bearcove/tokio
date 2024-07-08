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
        #[allow(non_snake_case)]
        #[allow(missing_docs)]
        pub mod $name {
            extern "C" {
                #[link_name = concat!(stringify!($name), "_THUNK")]
                pub static THUNK: fn(Option<&mut Option<u64>>) -> *const u8;
            }
        }

        pub static $name: ::std::thread::LocalKey<$ty> = {
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
                        type InnerFn = fn(Option<&mut Option<$ty>>) -> *const $ty;
                        let thunk = std::mem::transmute::<_, InnerFn>($name::THUNK);
                        // println!("calling thunk, which is {:p}", thunk);
                        let ret = thunk(n);
                        // println!("calling thunk, which is {:p}, it returned {:p}", thunk, ret);
                        ret
                    },
                })
            }
        };
    };
}
