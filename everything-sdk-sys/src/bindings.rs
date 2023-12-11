//! Rust C-bindings for `Everything.h` in Everything-SDK, almost purely handwritten.

use windows::Win32::Foundation::{BOOL, FILETIME, HWND, LPARAM, WPARAM};

// type BOOL = i32;
pub type DWORD = u32;
pub type UINT = u32;
type LPCSTR = windows::core::PCSTR;
type LPCWSTR = windows::core::PCWSTR;
type LPSTR = windows::core::PSTR;
type LPWSTR = windows::core::PWSTR;
#[allow(non_camel_case_types)]
pub type LARGE_INTEGER = i64; // Ref: https://github.com/microsoft/windows-rs/issues/2370

// all #define EVERYTHING_* const value
pub const EVERYTHING_SDK_VERSION: u32 = 2; // if not defined, version is 1.

pub const EVERYTHING_OK: u32 = 0; // no error detected
pub const EVERYTHING_ERROR_MEMORY: u32 = 1; // out of memory.
pub const EVERYTHING_ERROR_IPC: u32 = 2; // Everything search client is not running
pub const EVERYTHING_ERROR_REGISTERCLASSEX: u32 = 3; // unable to register window class.
pub const EVERYTHING_ERROR_CREATEWINDOW: u32 = 4; // unable to create listening window
pub const EVERYTHING_ERROR_CREATETHREAD: u32 = 5; // unable to create listening thread
pub const EVERYTHING_ERROR_INVALIDINDEX: u32 = 6; // invalid index
pub const EVERYTHING_ERROR_INVALIDCALL: u32 = 7; // invalid call
pub const EVERYTHING_ERROR_INVALIDREQUEST: u32 = 8; // invalid request data, request data first.
pub const EVERYTHING_ERROR_INVALIDPARAMETER: u32 = 9; // bad parameter.

pub const EVERYTHING_SORT_NAME_ASCENDING: u32 = 1;
pub const EVERYTHING_SORT_NAME_DESCENDING: u32 = 2;
pub const EVERYTHING_SORT_PATH_ASCENDING: u32 = 3;
pub const EVERYTHING_SORT_PATH_DESCENDING: u32 = 4;
pub const EVERYTHING_SORT_SIZE_ASCENDING: u32 = 5;
pub const EVERYTHING_SORT_SIZE_DESCENDING: u32 = 6;
pub const EVERYTHING_SORT_EXTENSION_ASCENDING: u32 = 7;
pub const EVERYTHING_SORT_EXTENSION_DESCENDING: u32 = 8;
pub const EVERYTHING_SORT_TYPE_NAME_ASCENDING: u32 = 9;
pub const EVERYTHING_SORT_TYPE_NAME_DESCENDING: u32 = 10;
pub const EVERYTHING_SORT_DATE_CREATED_ASCENDING: u32 = 11;
pub const EVERYTHING_SORT_DATE_CREATED_DESCENDING: u32 = 12;
pub const EVERYTHING_SORT_DATE_MODIFIED_ASCENDING: u32 = 13;
pub const EVERYTHING_SORT_DATE_MODIFIED_DESCENDING: u32 = 14;
pub const EVERYTHING_SORT_ATTRIBUTES_ASCENDING: u32 = 15;
pub const EVERYTHING_SORT_ATTRIBUTES_DESCENDING: u32 = 16;
pub const EVERYTHING_SORT_FILE_LIST_FILENAME_ASCENDING: u32 = 17;
pub const EVERYTHING_SORT_FILE_LIST_FILENAME_DESCENDING: u32 = 18;
pub const EVERYTHING_SORT_RUN_COUNT_ASCENDING: u32 = 19;
pub const EVERYTHING_SORT_RUN_COUNT_DESCENDING: u32 = 20;
pub const EVERYTHING_SORT_DATE_RECENTLY_CHANGED_ASCENDING: u32 = 21;
pub const EVERYTHING_SORT_DATE_RECENTLY_CHANGED_DESCENDING: u32 = 22;
pub const EVERYTHING_SORT_DATE_ACCESSED_ASCENDING: u32 = 23;
pub const EVERYTHING_SORT_DATE_ACCESSED_DESCENDING: u32 = 24;
pub const EVERYTHING_SORT_DATE_RUN_ASCENDING: u32 = 25;
pub const EVERYTHING_SORT_DATE_RUN_DESCENDING: u32 = 26;

pub const EVERYTHING_REQUEST_FILE_NAME: u32 = 0x00000001;
pub const EVERYTHING_REQUEST_PATH: u32 = 0x00000002;
pub const EVERYTHING_REQUEST_FULL_PATH_AND_FILE_NAME: u32 = 0x00000004;
pub const EVERYTHING_REQUEST_EXTENSION: u32 = 0x00000008;
pub const EVERYTHING_REQUEST_SIZE: u32 = 0x00000010;
pub const EVERYTHING_REQUEST_DATE_CREATED: u32 = 0x00000020;
pub const EVERYTHING_REQUEST_DATE_MODIFIED: u32 = 0x00000040;
pub const EVERYTHING_REQUEST_DATE_ACCESSED: u32 = 0x00000080;
pub const EVERYTHING_REQUEST_ATTRIBUTES: u32 = 0x00000100;
pub const EVERYTHING_REQUEST_FILE_LIST_FILE_NAME: u32 = 0x00000200;
pub const EVERYTHING_REQUEST_RUN_COUNT: u32 = 0x00000400;
pub const EVERYTHING_REQUEST_DATE_RUN: u32 = 0x00000800;
pub const EVERYTHING_REQUEST_DATE_RECENTLY_CHANGED: u32 = 0x00001000;
pub const EVERYTHING_REQUEST_HIGHLIGHTED_FILE_NAME: u32 = 0x00002000;
pub const EVERYTHING_REQUEST_HIGHLIGHTED_PATH: u32 = 0x00004000;
pub const EVERYTHING_REQUEST_HIGHLIGHTED_FULL_PATH_AND_FILE_NAME: u32 = 0x00008000;

pub const EVERYTHING_TARGET_MACHINE_X86: u32 = 1;
pub const EVERYTHING_TARGET_MACHINE_X64: u32 = 2;
pub const EVERYTHING_TARGET_MACHINE_ARM: u32 = 3;

// export 88 functions listed in .def file provided by Everything-SDK, and 2 functions NOT listed in .def file
// the former require winapi-rs features "winuser" and "shellapi", the latter require "winsvc"
//
// it seems they are a little special for `Everything_MSIExitAndStopService` and `Everything_MSIStartService`
extern "C" {

    // write search state
    pub fn Everything_SetSearchW(lpString: LPCWSTR);
    pub fn Everything_SetSearchA(lpString: LPCSTR);
    pub fn Everything_SetMatchPath(bEnable: BOOL);
    pub fn Everything_SetMatchCase(bEnable: BOOL);
    pub fn Everything_SetMatchWholeWord(bEnable: BOOL);
    pub fn Everything_SetRegex(bEnable: BOOL);
    pub fn Everything_SetMax(dwMax: DWORD);
    pub fn Everything_SetOffset(dwOffset: DWORD);
    pub fn Everything_SetReplyWindow(hWnd: HWND);
    pub fn Everything_SetReplyID(dwId: DWORD);
    pub fn Everything_SetSort(dwSort: DWORD); // Everything 1.4.1
    pub fn Everything_SetRequestFlags(dwRequestFlags: DWORD); // Everything 1.4.1

    // read search state
    pub fn Everything_GetMatchPath() -> BOOL;
    pub fn Everything_GetMatchCase() -> BOOL;
    pub fn Everything_GetMatchWholeWord() -> BOOL;
    pub fn Everything_GetRegex() -> BOOL;
    pub fn Everything_GetMax() -> DWORD;
    pub fn Everything_GetOffset() -> DWORD;
    pub fn Everything_GetSearchA() -> LPCSTR;
    pub fn Everything_GetSearchW() -> LPCWSTR;
    pub fn Everything_GetLastError() -> DWORD;
    pub fn Everything_GetReplyWindow() -> HWND;
    pub fn Everything_GetReplyID() -> DWORD;
    pub fn Everything_GetSort() -> DWORD; // Everything 1.4.1
    pub fn Everything_GetRequestFlags() -> DWORD; // Everything 1.4.1

    // execute query
    pub fn Everything_QueryA(bWait: BOOL) -> BOOL;
    pub fn Everything_QueryW(bWait: BOOL) -> BOOL;

    // query reply
    pub fn Everything_IsQueryReply(
        message: UINT,
        wParam: WPARAM,
        lParam: LPARAM,
        dwId: DWORD,
    ) -> BOOL;

    // write result state
    pub fn Everything_SortResultsByPath();

    // read result state
    pub fn Everything_GetNumFileResults() -> DWORD;
    pub fn Everything_GetNumFolderResults() -> DWORD;
    pub fn Everything_GetNumResults() -> DWORD;
    pub fn Everything_GetTotFileResults() -> DWORD;
    pub fn Everything_GetTotFolderResults() -> DWORD;
    pub fn Everything_GetTotResults() -> DWORD;
    pub fn Everything_IsVolumeResult(dwIndex: DWORD) -> BOOL;
    pub fn Everything_IsFolderResult(dwIndex: DWORD) -> BOOL;
    pub fn Everything_IsFileResult(dwIndex: DWORD) -> BOOL;
    pub fn Everything_GetResultFileNameW(dwIndex: DWORD) -> LPCWSTR;
    pub fn Everything_GetResultFileNameA(dwIndex: DWORD) -> LPCSTR;
    pub fn Everything_GetResultPathW(dwIndex: DWORD) -> LPCWSTR;
    pub fn Everything_GetResultPathA(dwIndex: DWORD) -> LPCSTR;
    pub fn Everything_GetResultFullPathNameA(dwIndex: DWORD, buf: LPSTR, bufsize: DWORD) -> DWORD;
    pub fn Everything_GetResultFullPathNameW(
        dwIndex: DWORD,
        wbuf: LPWSTR,
        wbuf_size_in_wchars: DWORD,
    ) -> DWORD;
    pub fn Everything_GetResultListSort() -> DWORD; // Everything 1.4.1
    pub fn Everything_GetResultListRequestFlags() -> DWORD; // Everything 1.4.1
    pub fn Everything_GetResultExtensionW(dwIndex: DWORD) -> LPCWSTR; // Everything 1.4.1
    pub fn Everything_GetResultExtensionA(dwIndex: DWORD) -> LPCSTR; // Everything 1.4.1
    pub fn Everything_GetResultSize(dwIndex: DWORD, lpSize: *mut LARGE_INTEGER) -> BOOL; // Everything 1.4.1
    pub fn Everything_GetResultDateCreated(dwIndex: DWORD, lpDateCreated: *mut FILETIME) -> BOOL; // Everything 1.4.1
    pub fn Everything_GetResultDateModified(dwIndex: DWORD, lpDateModified: *mut FILETIME) -> BOOL; // Everything 1.4.1
    pub fn Everything_GetResultDateAccessed(dwIndex: DWORD, lpDateAccessed: *mut FILETIME) -> BOOL; // Everything 1.4.1
    pub fn Everything_GetResultAttributes(dwIndex: DWORD) -> DWORD; // Everything 1.4.1
    pub fn Everything_GetResultFileListFileNameW(dwIndex: DWORD) -> LPCWSTR; // Everything 1.4.1
    pub fn Everything_GetResultFileListFileNameA(dwIndex: DWORD) -> LPCSTR; // Everything 1.4.1
    pub fn Everything_GetResultRunCount(dwIndex: DWORD) -> DWORD; // Everything 1.4.1
    pub fn Everything_GetResultDateRun(dwIndex: DWORD, lpDateRun: *mut FILETIME) -> BOOL;
    pub fn Everything_GetResultDateRecentlyChanged(
        dwIndex: DWORD,
        lpDateRecentlyChanged: *mut FILETIME,
    ) -> BOOL;
    pub fn Everything_GetResultHighlightedFileNameW(dwIndex: DWORD) -> LPCWSTR; // Everything 1.4.1
    pub fn Everything_GetResultHighlightedFileNameA(dwIndex: DWORD) -> LPCSTR; // Everything 1.4.1
    pub fn Everything_GetResultHighlightedPathW(dwIndex: DWORD) -> LPCWSTR; // Everything 1.4.1
    pub fn Everything_GetResultHighlightedPathA(dwIndex: DWORD) -> LPCSTR; // Everything 1.4.1
    pub fn Everything_GetResultHighlightedFullPathAndFileNameW(dwIndex: DWORD) -> LPCWSTR; // Everything 1.4.1
    pub fn Everything_GetResultHighlightedFullPathAndFileNameA(dwIndex: DWORD) -> LPCSTR; // Everything 1.4.1

    // reset state and free any allocated memory
    pub fn Everything_Reset();
    pub fn Everything_CleanUp();

    pub fn Everything_GetMajorVersion() -> DWORD;
    pub fn Everything_GetMinorVersion() -> DWORD;
    pub fn Everything_GetRevision() -> DWORD;
    pub fn Everything_GetBuildNumber() -> DWORD;
    pub fn Everything_Exit() -> BOOL;
    // can be called as admin or standard user. will self elevate if needed. (NOT in .def)
    #[cfg(feature = "vendored")]
    pub fn Everything_MSIExitAndStopService(msihandle: *const std::ffi::c_void) -> UINT;
    // MUST be called as an admin (NOT in .def)
    #[cfg(feature = "vendored")]
    pub fn Everything_MSIStartService(msihandle: *const std::ffi::c_void) -> UINT;
    pub fn Everything_IsDBLoaded() -> BOOL; // Everything 1.4.1
    pub fn Everything_IsAdmin() -> BOOL; // Everything 1.4.1
    pub fn Everything_IsAppData() -> BOOL; // Everything 1.4.1
    pub fn Everything_RebuildDB() -> BOOL; // Everything 1.4.1
    pub fn Everything_UpdateAllFolderIndexes() -> BOOL; // Everything 1.4.1
    pub fn Everything_SaveDB() -> BOOL; // Everything 1.4.1
    pub fn Everything_SaveRunHistory() -> BOOL; // Everything 1.4.1
    pub fn Everything_DeleteRunHistory() -> BOOL; // Everything 1.4.1
    pub fn Everything_GetTargetMachine() -> DWORD; // Everything 1.4.1
    pub fn Everything_IsFastSort(sortType: DWORD) -> BOOL; // Everything 1.4.1.859
    pub fn Everything_IsFileInfoIndexed(fileInfoType: DWORD) -> BOOL; // Everything 1.4.1.859

    pub fn Everything_GetRunCountFromFileNameW(lpFileName: LPCWSTR) -> DWORD; // Everything 1.4.1
    pub fn Everything_GetRunCountFromFileNameA(lpFileName: LPCSTR) -> DWORD; // Everything 1.4.1
    pub fn Everything_SetRunCountFromFileNameW(lpFileName: LPCWSTR, dwRunCount: DWORD) -> BOOL; // Everything 1.4.1
    pub fn Everything_SetRunCountFromFileNameA(lpFileName: LPCSTR, dwRunCount: DWORD) -> BOOL; // Everything 1.4.1
    pub fn Everything_IncRunCountFromFileNameW(lpFileName: LPCWSTR) -> DWORD; // Everything 1.4.1
    pub fn Everything_IncRunCountFromFileNameA(lpFileName: LPCSTR) -> DWORD; // Everything 1.4.1
}
