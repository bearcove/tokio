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

#[cfg(all(not(all(loom, test)), feature = "import-globals"))]
macro_rules! tokio_thread_local {
    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty = const { $expr:expr } $(;)?) => {
        tokio_thread_local! {
            $(#[$attrs])*
            $vis static $name: $ty = $expr;
        }
    };

    ($(#[$attrs:meta])* $vis:vis static $name:ident: $ty:ty = $expr:expr $(;)?) => {
        extern "C" {
            #[link_name = concat!(stringify!($name), "_THUNK")]
            #[allow(improper_ctypes)]
            $vis static mut $name: ::std::thread::LocalKey<$ty>;
        }
    };
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
            static $name: $ty = $expr;
        }

        #[no_mangle]
        #[allow(improper_ctypes)]
        $vis static $name: ::std::thread::LocalKey<$ty> = {
            ::std::thread::LocalKey::new(|| $name.with(|v| unsafe { ::std::ptr::read(v) }))
        };
    };
}
