#![no_std]
#![feature(macro_metavar_expr)]

#[doc(hidden)]
pub use windows_sys;

#[macro_export]
macro_rules! proxy {
(
    $dll:literal;
    $(
        fn $export:ident ($($param:ident: $ty:ty $(,)?),*) $(-> $ret:ty)?;
    )*
) =>  {
    #[doc(hidden)]
    mod __dll_proxy_impl {
        // import types from the outer context
        use super::*;

        use ::core::ffi::c_int;
        use ::core::ptr::null_mut;

        // import windows_sys from our reexport
        use $crate::windows_sys;
        use windows_sys::Win32::Foundation::HMODULE;

        use windows_sys::Win32::Foundation::{S_OK, FARPROC};
        use windows_sys::Win32::System::Com::CoTaskMemRealloc;
        use windows_sys::Win32::System::LibraryLoader::{LoadLibraryW, GetProcAddress};
        use windows_sys::Win32::UI::Shell::{FOLDERID_System, SHGetKnownFolderPath};
        use windows_sys::core::PCWSTR;
        use windows_sys::core::w;

        // not provided by windows_sys but we still need this
        unsafe extern "C" {
            fn lstrlenW(lpString: PCWSTR) -> c_int;
        }

        const DLL_NAME: PCWSTR = w!($dll);
        const PROXY_FN_COUNT: usize = ${count($export)};

        static mut DLL_HANDLE: HMODULE = null_mut();
        static mut FARPROC_HANDLES: [FARPROC; PROXY_FN_COUNT] = [None; PROXY_FN_COUNT];

        // we use CRT initializers in order to setup our state at load time
        // we use XCT here so that we load our HMODULE before we try to call
        // GetProcAddress below
        #[used]
        #[unsafe(link_section = ".CRT$XCT")]
        static INIT_HMODULE: unsafe extern "C" fn() = {
            unsafe extern "C" fn init_hmodule() {
                let mut system32_path = null_mut();

                let res = unsafe {
                    SHGetKnownFolderPath(&FOLDERID_System, 0, null_mut(), &mut system32_path)
                };

                // TODO: handle failure more robustly
                if res != S_OK {
                    return;
                }

                let system32_len = unsafe { lstrlenW(system32_path) } as usize;
                let dll_name_len = unsafe { lstrlenW(DLL_NAME) } as usize;

                // ../Windows/System32 + "/" + "proxy_name.dll" + "\0"
                let new_len = system32_len + 1 + dll_name_len + 1;

                let system32_path =
                    unsafe { CoTaskMemRealloc(system32_path as _, 2 * new_len) as *mut u16 };


                // TODO: handle failure more robustly
                if system32_path.is_null() {
                    return;
                }

                let path_end = unsafe { system32_path.add(system32_len) };
                unsafe { core::ptr::copy(w!(r"\"), path_end, 1) };

                let path_end = unsafe { path_end.add(1) };
                unsafe { core::ptr::copy(DLL_NAME, path_end, dll_name_len + 1) };

                let dll_path = system32_path;

                unsafe { DLL_HANDLE = LoadLibraryW(dll_path) };

                unsafe { CoTaskMemRealloc(system32_path as _, 0) };
            }

            init_hmodule
        };

        // initialize our handles to the proxied DLLs functions
        // we use XCV in order to ensure that we only call this after
        // DLL_HANDLE has been initialized by the above function
        #[used]
        #[unsafe(link_section = ".CRT$XCV")]
        static INIT_PROXY_FNS: unsafe extern "C" fn() = {
            unsafe extern "C" fn init_proxy_fns() {
                $({
                    const NAME: *const u8 = concat!(stringify!($export), '\0').as_ptr();
                    unsafe { FARPROC_HANDLES[${index()}] = GetProcAddress(DLL_HANDLE, NAME) };
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
}
}
