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

#[cfg(all(
    not(all(loom, test)),
    not(any(feature = "import-globals", feature = "export-globals"))
))]
macro_rules! tokio_thread_local {
    ($($tts:tt)+) => {
        ::std::thread_local!{ $($tts)+ }
    }
}

#[cfg(all(not(all(loom, test)), feature = "export-globals"))]
macro_rules! tokio_thread_local {
    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty = const { $expr:expr } $(;)?) => {
        tokio_thread_local! {
            $(#[$attrs])*
            $vis static $name: $ty = $expr;
        }
    };

    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty = $expr:expr $(;)?) => {
        ::std::thread_local! {
            $(#[$attrs])*
            $vis static $name: $ty = $expr;
        }

        #[allow(non_snake_case)]
        mod $name {
            struct MyLocalKey<T: 'static> {
                inner: unsafe fn(Option<&mut Option<T>>) -> Option<&'static T>,
            }

            #[no_mangle]
            static $name: MyLocalKey<()> = MyLocalKey {
                inner: |v| {
                    unsafe {
                        let lk = std::mem::transmute::<_, MyLocalKey<()>>(super::$name);
                        (lk.inner)(v)
                    }
                }
            };
        }
    };
}

#[cfg(all(not(all(loom, test)), feature = "import-globals"))]
macro_rules! tokio_thread_local {
    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty = const { $expr:expr } $(;)?) => {
        tokio_thread_local! {
            $(#[$attrs])*
            $vis static $name: $ty = $expr;
        }
    };

    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty = $expr:expr $(;)?) => {
        #[allow(non_snake_case)]
        mod $name {
            extern "C" {
                #[link_name = stringify!($name)]
                #[allow(improper_ctypes)]
                pub(super) static LK: ::std::thread::LocalKey<()>;
            }
        }

        static $name: &'static ::std::thread::LocalKey<$ty> = unsafe { std::mem::transmute(&$name::LK) };
    };
}
