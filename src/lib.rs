#![no_std]
#![feature(macro_metavar_expr)]

#[cfg(not(windows))]
compiler_error!("dll-proxy-macro is only supported on windows");

/// Reexports for types used by [`proxy`]
#[doc(hidden)]
pub mod x {
    pub use core::ffi::c_int;
    pub use core::ptr::{copy, null_mut};

    pub use windows_sys::Win32::Foundation::{FARPROC, HMODULE, S_OK};
    pub use windows_sys::Win32::System::Com::CoTaskMemRealloc;
    pub use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    pub use windows_sys::Win32::UI::Shell::{FOLDERID_System, SHGetKnownFolderPath};
    pub use windows_sys::core::{PCWSTR, w};

    #[cfg(feature = "panic-handler")]
    pub use windows_sys::Win32::System::Threading::ExitProcess;
}

#[macro_export]
/// # Examples
///
/// ```rust
/// #![feature(macro_metavar_expr)]
///
/// dll_proxy_macro::proxy! {
///     "my_dll"
///
///     fn DllExportFn1();
///     fn DllExportFn2(param: usize) -> u32;
///     fn MyDllFunction(
///         param1: *mut u32,
///         param2: *mut *mut usize,
///         param3: *const fn(u32, u32) -> u32,
///     ) -> u32;
/// # /*
///     ...
/// # */
/// }
/// ```
macro_rules! proxy {
(
    $dll:literal
    $(
        fn $export:ident ($($param:ident: $ty:ty $(,)?),*) $(-> $ret:ty)?;
    )*
) =>  {
    #[doc(hidden)]
    mod __dll_proxy_impl {
        // import types from the outer context
        // so that they exist when they are
        // used in the export declarations
        use super::*;

        // not provided by windows_sys but we still need this
        unsafe extern "C" {
            fn lstrlenW(lpString: $crate::x::PCWSTR) -> $crate::x::c_int;
        }

        const DLL_NAME: $crate::x::PCWSTR = $crate::x::w!($dll);
        const PROXY_FN_COUNT: usize = ${count($export)};

        static mut DLL_HANDLE: $crate::x::HMODULE = $crate::x::null_mut();
        static mut FARPROC_HANDLES: [$crate::x::FARPROC; PROXY_FN_COUNT] = [None; PROXY_FN_COUNT];

        // we use CRT initializers in order to setup our state at
        // load time we use XCB here so that we load our HMODULE
        // before we try to call GetProcAddress below
        #[used]
        #[unsafe(link_section = ".CRT$XCB")]
        static INIT_HMODULE: unsafe extern "C" fn() = {
            unsafe extern "C" fn init_hmodule() {
                let mut system32_path = $crate::x::null_mut();

                let res = unsafe {
                    $crate::x::SHGetKnownFolderPath(&$crate::x::FOLDERID_System, 0, $crate::x::null_mut(), &mut system32_path)
                };

                // TODO: handle failure more robustly
                if res != $crate::x::S_OK {
                    return;
                }

                let system32_len = unsafe { lstrlenW(system32_path) } as usize;
                let dll_name_len = unsafe { lstrlenW(DLL_NAME) } as usize;

                // ../Windows/System32 + "/" + "proxy_name.dll" + "\0"
                let new_len = system32_len + 1 + dll_name_len + 1;

                let system32_path =
                    unsafe { $crate::x::CoTaskMemRealloc(system32_path as _, 2 * new_len) as *mut u16 };


                // TODO: handle failure more robustly
                if system32_path.is_null() {
                    return;
                }

                let path_end = unsafe { system32_path.add(system32_len) };
                unsafe { $crate::x::copy($crate::x::w!(r"\"), path_end, 1) };

                let path_end = unsafe { path_end.add(1) };
                unsafe { $crate::x::copy(DLL_NAME, path_end, dll_name_len + 1) };

                let dll_path = system32_path;

                unsafe { DLL_HANDLE = $crate::x::LoadLibraryW(dll_path) };

                unsafe { $crate::x::CoTaskMemRealloc(system32_path as _, 0) };
            }

            init_hmodule
        };

        // initialize our handles to the proxied DLLs functions
        // we use XCC in order to ensure that we only call this after
        // DLL_HANDLE has been initialized by the above function
        #[used]
        #[unsafe(link_section = ".CRT$XCC")]
        static INIT_PROXY_FNS: unsafe extern "C" fn() = {
            unsafe extern "C" fn init_proxy_fns() {
                $({
                    const NAME: *const u8 = concat!(stringify!($export), '\0').as_ptr();
                    unsafe { FARPROC_HANDLES[${index()}] = $crate::x::GetProcAddress(DLL_HANDLE, NAME) };
                })*
            }

            init_proxy_fns
        };

        // declare our exported functions
        $(
            #[unsafe(no_mangle)]
            #[allow(non_snake_case)]
            unsafe extern "stdcall" fn $export($($param: $ty),*) $(-> $ret)? {
                type F = unsafe extern "stdcall" fn($($ty),*)$(-> $ret)?;

                let f = unsafe { FARPROC_HANDLES[${index()}] };
                let f = unsafe { core::mem::transmute::<_, F>(f) };

                unsafe { f($($param),*) }
            }
        )*
    }

    $crate::__panic_handler!();
}
}

#[doc(hidden)]
#[macro_export]
#[cfg(feature = "panic-handler")]
macro_rules! __panic_handler {
    () => {
        #[cfg(not(test))] // so that rust-analyzer doesn't complain
        #[panic_handler]
        fn panic(info: &core::panic::PanicInfo) -> ! {
            unsafe { $crate::x::ExitProcess(u32::MAX) }
        }
    };
}

#[doc(hidden)]
#[macro_export]
#[cfg(not(feature = "panic-handler"))]
macro_rules! __panic_handler {
    () => {};
}
