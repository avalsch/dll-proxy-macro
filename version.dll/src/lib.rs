#![no_std]
#![allow(non_snake_case)]
#![feature(macro_metavar_expr)]

//! A no_std proxy dll for version.dll
//!
//! For your own sake, take this as just an example and
//! pick a different DLL with less functions to proxy.
//!
//! You can use any method run arbitrary code on inject.
//!
//! HOWEVER, using DllMain with the typical rust signature
//! (below) may cause issues as it will cause changes to
//! the ordinals of the DLL exports. Exports are ordered
//! alphabetically by default, and adding DllMain causes
//! the ordering to change. You can avoid this with a module
//! definition file specifying the ordinal of each function manually.
//!
//! (https://learn.microsoft.com/en-us/cpp/build/reference/module-definition-dot-def-files?view=msvc-170)
//!
//! ```rust
//! # use core::ffi::c_void;
//! # use windows_sys::Win32::Foundation::{HMODULE, BOOL};
//!
//! #[unsafe(export_name = "DllMain")]
//! extern "system" fn dll_main(module: HMODULE, call_reason: u32, _: *mut c_void) -> BOOL {
//! # /*
//!     ...
//! # */
//! }
//! ```
//!
//! In this example we use [CRT Initialization]
//! (https://learn.microsoft.com/en-us/cpp/c-runtime-library/crt-initialization)
//! inspired by [ctor](https://docs.rs/ctor/latest/ctor/attr.ctor.html#details)
//! to run our code after the DLL has been loaded.

use core::ffi::c_void;

use dll_proxy_macro::proxy;
use windows_sys::Win32::Foundation::{BOOL, HANDLE};
use windows_sys::Win32::Storage::FileSystem::{
    BY_HANDLE_FILE_INFORMATION, GET_FILE_VERSION_INFO_FLAGS, VER_FIND_FILE_FLAGS,
    VER_FIND_FILE_STATUS, VER_INSTALL_FILE_FLAGS, VER_INSTALL_FILE_STATUS,
};
use windows_sys::core::{PCSTR, PCWSTR, PSTR, PWSTR};

// CRT Initialization in order to run code at load time
#[used]
#[unsafe(link_section = ".CRT$XCY")]
static INIT: extern "C" fn() = {
    extern "C" fn init() {
        // run arbitrary code here
    }

    init
};

// Proxying VERSION.dll
proxy! {
    "VERSION"

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-getfileversioninfoa
    fn GetFileVersionInfoA(
        lptstrFilename: PCSTR,
        dwHandle: u32,
        dwLen: u32,
        lpData: *mut c_void,
    ) -> BOOL;

    // https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfileinformationbyhandle
    fn GetFileVersionInfoByHandle(
        hFile: HANDLE,
        lpFileInformation: *mut BY_HANDLE_FILE_INFORMATION
    ) -> BOOL;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-getfileversioninfoexa
    fn GetFileVersionInfoExA(
        dwFlags: GET_FILE_VERSION_INFO_FLAGS,
        lpwstrFilename: PCSTR,
        dwHandle: u32,
        dwLen: u32,
        lpData: *mut c_void,
    ) -> BOOL;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-getfileversioninfoexw
    fn GetFileVersionInfoExW(
        dwFlags: GET_FILE_VERSION_INFO_FLAGS,
        lpwstrFilename: PCWSTR,
        dwHandle: u32,
        dwLen: u32,
        lpData: *mut c_void,
    ) -> BOOL;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-getfileversioninfosizea
    fn GetFileVersionInfoSizeA(
        lptstrFilename: PCSTR,
        lpdwHandle: *mut u32,
    ) -> u32;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-getfileversioninfosizeexa
    fn GetFileVersionInfoSizeExA(
        dwFlags: GET_FILE_VERSION_INFO_FLAGS,
        lpwstrFilename: PCSTR,
        lpdwHandle: *mut u32,
    ) -> u32;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-getfileversioninfosizeexw
    fn GetFileVersionInfoSizeExW(
        dwFlags: GET_FILE_VERSION_INFO_FLAGS,
        lpwstrFilename: PCWSTR,
        lpdwHandle: *mut u32,
    ) -> u32;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-getfileversioninfosizew
    fn GetFileVersionInfoSizeW(
        lptstrFilename: PCWSTR,
        lpdwHandle: *mut u32,
    ) -> u32;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-getfileversioninfow
    fn GetFileVersionInfoW(
        lptstrFilename: PCWSTR,
        dwHandle: u32,
        dwLen: u32,
        lpData: *mut c_void,
    ) -> BOOL;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-verfindfilea
    fn VerFindFileA(
        uFlags: VER_FIND_FILE_FLAGS,
        szFileName: PCSTR,
        szWinDir: PCSTR,
        szAppDir: PCSTR,
        szCurDir: PSTR,
        puCurDirLen: *mut u32,
        szDestDir: PSTR,
        puDestDirLen: *mut u32,
    ) -> VER_FIND_FILE_STATUS;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-verfindfilew
    fn VerFindFileW(
        uFlags: VER_FIND_FILE_FLAGS,
        szFileName: PCWSTR,
        szwInDir: PCWSTR,
        szAppDir: PCWSTR,
        szCurDir: PWSTR,
        puCurDirLen: *mut u32,
        szDestDir: PWSTR,
        puDestDirLen: *mut u32,
    ) -> VER_FIND_FILE_STATUS;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-verinstallfilea
    fn VerInstallFileA(
        ufLags: VER_INSTALL_FILE_FLAGS,
        szSrcFileName: PCSTR,
        szDestFileName: PCSTR,
        szSrcDir: PCSTR,
        szDestDir: PCSTR,
        szCurDir: PCSTR,
        szTmpFile: PSTR,
        puTmpFileLen: *mut u32,
    ) -> VER_INSTALL_FILE_STATUS;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-verinstallfilew
    fn VerInstallFileW(
        uFlags: VER_INSTALL_FILE_FLAGS,
        szSrcFileName: PCWSTR,
        szDestFileName: PCWSTR,
        szSrcDir: PCWSTR,
        szDestDir: PCWSTR,
        szCurDir: PCWSTR,
        szTmpFile: PWSTR,
        puTmpFileLen: *mut u32,
    ) -> VER_INSTALL_FILE_STATUS;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-verlanguagenamea
    fn VerLanguageNameA(
        wLang: u32,
        szlang: PSTR,
        cchLang: u32,
    ) -> u32;

    //https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-verlanguagenamew
    fn VerLanguageNameW(
        wLang: u32,
        szLang: PWSTR,
        cchLang: u32
    ) -> u32;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-verqueryvaluea
    fn VerQueryValueA(
        pBlock: *const c_void,
        lpSubBlock: PCSTR,
        lplpBuffer: *mut *mut c_void,
        puLen: *mut u32,
    ) -> BOOL;

    // https://learn.microsoft.com/en-us/windows/win32/api/winver/nf-winver-verqueryvaluew
    fn VerQueryValueW(
        pBlock: *const c_void,
        lpSubBlock: PCWSTR,
        lplpBuffer: *mut *mut c_void,
        puLen: *mut u32,
    ) -> BOOL;
}
