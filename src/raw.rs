//! Raw function wrapper in Rust type for pure C-bindings with Windows related types
//!
//! This code is written based on the official SDK web documents in October 2023.
//! The [Everything SDK Documents](https://www.voidtools.com/support/everything/sdk/)
//! web content may be changed and updated in the future, therefore, the code logic
//! is based on the comment documents (rustdoc) in this code instead of the web documents.
//!
//! Rust does a good job of supporting Unicode, so it seems that no need to support
//! the ANSI version functions such as `Everything_SetSearchA` or `Everything_QueryA`.
//! Therefore, this crate uses Unicode version functions ending in `W` by default.
//! (Ref: <https://stackoverflow.com/questions/33714546/winapi-unicode-and-ansi-functions>)
//!
//! For input value of type `LPCWSTR` when we calling `Everything_SetSearchW` or else, we
//! do not need to keep the memory of search text available after the return of this
//! function, because the C code in Everything-SDK will allocate the memory to store the
//! search text. After calling these functions, we can deallocate the memory which the
//! input pointer points to.

#![allow(non_snake_case)]

use std::{
    ffi::{OsStr, OsString},
    fmt::Display,
    num::NonZeroU32,
};

use bitflags::bitflags;
use enum_primitive_derive::Primitive;
use sdk_sys::{LARGE_INTEGER, UINT};
use widestring::{U16CStr, U16CString};

use everything_sdk_sys as sdk_sys;
// use winapi::um::winnt::ULARGE_INTEGER;
use windows::{
    core::{PCWSTR, PWSTR},
    Win32::{
        Foundation::{BOOL, FALSE, FILETIME, HWND, LPARAM, TRUE, WPARAM},
        Storage::FileSystem::INVALID_FILE_ATTRIBUTES,
    },
};

// pub type LARGE_INTEGER = i64;
// pub type ULARGE_INTEGER = u64;
// pub type UINT = u32;

// use windows::Win32::Foundation::{TRUE, FALSE, HWND};

/// convert the Win32 [`BOOL`] to normal `bool`
fn lower_bool(b: BOOL) -> bool {
    match b {
        TRUE => true,
        FALSE => false,
        _ => unreachable!(),
    }
}

/// convert the Win32 [`BOOL`] to normal `bool`. Check LastError when FALSE.
fn lower_bool_or_ipc_error(b: BOOL) -> Option<bool> {
    match b {
        TRUE => Some(true),
        FALSE => match Everything_GetLastError() {
            LastError::EVERYTHING_OK => Some(false),
            LastError::EVERYTHING_ERROR_IPC => None,
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
}

/// Check if IPC Error occurred when u32 number is 0.
fn zero_or_ipc_error(n: u32) -> Option<u32> {
    if n == 0 {
        match Everything_GetLastError() {
            LastError::EVERYTHING_OK => Some(0),
            LastError::EVERYTHING_ERROR_IPC => None,
            _ => unreachable!(),
        }
    } else {
        Some(n)
    }
}

// --- write search state ---

/// The `Everything_SetSearch` function sets the search string for the IPC Query.
///
/// # Arguments
/// * `text` - An os string to be used as the new search text.
///
/// # Remarks
/// - Optionally call this function prior to a call to `Everything_Query`
/// - `Everything_Query` executes the IPC Query using this search string.
/// - If you want to do one less memory copy (from OsStr to "valid" UTF-16 u16 array), you
///   should use [`everything_sdk_sys::Everything_SetSearchW`] directly.
pub fn Everything_SetSearch(text: impl AsRef<OsStr>) {
    // string slice to `\0` end C string
    let search_text = U16CString::from_os_str(text).expect("the nul value only in the end");
    unsafe { sdk_sys::Everything_SetSearchW(PCWSTR(search_text.as_ptr())) };
}

/// The `Everything_SetMatchPath` function enables or disables full path matching for
/// the next call to `Everything_Query`.
///
/// # Arguments
/// * `enabled` - If `true`, full path matching is enabled, `false` is disabled.
///
/// # Remarks
/// - If match full path is being enabled, the next call to `Everything_Query` will search
///   the full path and file name of each file and folder.
/// - If match full path is being disabled, the next call to `Everything_Query` will search
///   the file name only of each file and folder.
/// - Match path is disabled by default.
/// - Enabling match path will add a significant performance hit.
pub fn Everything_SetMatchPath(enabled: bool) {
    let enabled: BOOL = if enabled { TRUE } else { FALSE };
    unsafe { sdk_sys::Everything_SetMatchPath(enabled) }
}

/// The `Everything_SetMatchCase` function enables or disables full path matching for
/// the next call to `Everything_Query`.
///
/// # Arguments
/// * `sensitive` - If `true`, the search is case sensitive, `false` is case insensitive.
///
/// # Remarks
/// - Match case is disabled by default.
pub fn Everything_SetMatchCase(sensitive: bool) {
    let enabled: BOOL = if sensitive { TRUE } else { FALSE };
    unsafe { sdk_sys::Everything_SetMatchCase(enabled) }
}

/// The `Everything_SetMatchWholeWord` function enables or disables matching whole words
/// for the next call to `Everything_Query`.
///
/// # Arguments
/// * `enabled` - If `true`, the search matches whole words only, `false` is the search
///   can occur anywhere.
///
/// # Remarks
/// - Match whole word is disabled by default.
pub fn Everything_SetMatchWholeWord(enabled: bool) {
    let enabled: BOOL = if enabled { TRUE } else { FALSE };
    unsafe { sdk_sys::Everything_SetMatchWholeWord(enabled) }
}

/// The `Everything_SetRegex` function enables or disables Regular Expression searching.
///
/// # Arguments
/// * `enabled` - Set to `true` to enable regex, set to `false` to disable regex.
///
/// # Remarks
/// - Regex is disabled by default.
pub fn Everything_SetRegex(enabled: bool) {
    let enabled: BOOL = if enabled { TRUE } else { FALSE };
    unsafe { sdk_sys::Everything_SetRegex(enabled) }
}

/// The `Everything_SetMax` function set the maximum number of results to return
/// from `Everything_Query`.
///
/// # Arguments
/// * `max_results` - Specifies the maximum number of results to return.
///   Setting this to `u32::MAX` (0xffffffff) will return all results.
///
/// # Remarks
/// - The default maximum number of results is 0xffffffff (all results).
/// - If you are displaying the results in a window, set the maximum number of results
///   to the number of visible items in the window.
pub fn Everything_SetMax(max_results: u32) {
    unsafe { sdk_sys::Everything_SetMax(max_results) }
}

/// The `Everything_SetOffset` function set the first result offset to return from
/// a call to `Everything_Query`.
///
/// # Arguments
/// * `offset` - Specifies the first result from the available results. Set this to 0 to
///   return the first available result.
///
/// # Remarks
/// - The default offset is 0 (the first available result).
/// - If you are displaying the results in a window with a custom scroll bar, set the
///   offset to the vertical scroll bar position.
/// - Using a search window can reduce the amount of data sent over the IPC and significantly
///   increase search performance.
pub fn Everything_SetOffset(offset: u32) {
    unsafe { sdk_sys::Everything_SetOffset(offset) }
}

/// The `Everything_SetReplyWindow` function sets the window that will receive the the IPC
/// Query results.
///
/// # Arguments
/// * `h_wnd` - The handle to the window that will receive the IPC Query reply.
///
/// # Remarks
/// - This function MUST be called before calling `Everything_Query` with `wait` set to `false`.
/// - Check for results with the specified window using `Everything_IsQueryReply`.
/// - Call `Everything_SetReplyID` with a unique identifier to specify multiple searches.
///
/// TODO: These functions coupled with the IPC mechanism that is `WM_COPYDATA` in Win32 API.
/// ...
#[cfg_attr(not(feature = "raw"), allow(dead_code))]
pub fn Everything_SetReplyWindow(h_wnd: HWND) {
    unsafe { sdk_sys::Everything_SetReplyWindow(h_wnd) }
}

/// The `Everything_SetReplyID` function sets the unique number to identify the next query.
///
/// # Arguments
/// * `n_id` - The unique number to identify the next query.
///
/// # Remarks
/// - The default identifier is 0.
/// - Set a unique identifier for the IPC Query.
/// - If you want to post multiple search queries with the same window handle, you MUST call
///   the `Everything_SetReplyID` function to assign each query a unique identifier.
/// - The nID value `n_id` is the `dwData` member in the `COPYDATASTRUCT` used in
///   the `WM_COPYDATA` reply message.
/// - This function is not required if you call `Everything_Query` with `wait` set to `true`.
///
/// # References
/// - [Using Data Copy](https://learn.microsoft.com/en-us/windows/win32/dataxchg/using-data-copy)
///
/// TODO: These functions coupled with the IPC mechanism that is `WM_COPYDATA` in Win32 API.
/// ...
#[cfg_attr(not(feature = "raw"), allow(dead_code))]
pub fn Everything_SetReplyID(n_id: u32) {
    unsafe { sdk_sys::Everything_SetReplyID(n_id) }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Primitive)]
#[allow(non_camel_case_types)]
pub enum SortType {
    EVERYTHING_SORT_NAME_ASCENDING = sdk_sys::EVERYTHING_SORT_NAME_ASCENDING,
    EVERYTHING_SORT_NAME_DESCENDING = sdk_sys::EVERYTHING_SORT_NAME_DESCENDING,
    EVERYTHING_SORT_PATH_ASCENDING = sdk_sys::EVERYTHING_SORT_PATH_ASCENDING,
    EVERYTHING_SORT_PATH_DESCENDING = sdk_sys::EVERYTHING_SORT_PATH_DESCENDING,
    EVERYTHING_SORT_SIZE_ASCENDING = sdk_sys::EVERYTHING_SORT_SIZE_ASCENDING,
    EVERYTHING_SORT_SIZE_DESCENDING = sdk_sys::EVERYTHING_SORT_SIZE_DESCENDING,
    EVERYTHING_SORT_EXTENSION_ASCENDING = sdk_sys::EVERYTHING_SORT_EXTENSION_ASCENDING,
    EVERYTHING_SORT_EXTENSION_DESCENDING = sdk_sys::EVERYTHING_SORT_EXTENSION_DESCENDING,
    EVERYTHING_SORT_TYPE_NAME_ASCENDING = sdk_sys::EVERYTHING_SORT_TYPE_NAME_ASCENDING,
    EVERYTHING_SORT_TYPE_NAME_DESCENDING = sdk_sys::EVERYTHING_SORT_TYPE_NAME_DESCENDING,
    EVERYTHING_SORT_DATE_CREATED_ASCENDING = sdk_sys::EVERYTHING_SORT_DATE_CREATED_ASCENDING,
    EVERYTHING_SORT_DATE_CREATED_DESCENDING = sdk_sys::EVERYTHING_SORT_DATE_CREATED_DESCENDING,
    EVERYTHING_SORT_DATE_MODIFIED_ASCENDING = sdk_sys::EVERYTHING_SORT_DATE_MODIFIED_ASCENDING,
    EVERYTHING_SORT_DATE_MODIFIED_DESCENDING = sdk_sys::EVERYTHING_SORT_DATE_MODIFIED_DESCENDING,
    EVERYTHING_SORT_ATTRIBUTES_ASCENDING = sdk_sys::EVERYTHING_SORT_ATTRIBUTES_ASCENDING,
    EVERYTHING_SORT_ATTRIBUTES_DESCENDING = sdk_sys::EVERYTHING_SORT_ATTRIBUTES_DESCENDING,
    EVERYTHING_SORT_FILE_LIST_FILENAME_ASCENDING =
        sdk_sys::EVERYTHING_SORT_FILE_LIST_FILENAME_ASCENDING,
    EVERYTHING_SORT_FILE_LIST_FILENAME_DESCENDING =
        sdk_sys::EVERYTHING_SORT_FILE_LIST_FILENAME_DESCENDING,
    EVERYTHING_SORT_RUN_COUNT_ASCENDING = sdk_sys::EVERYTHING_SORT_RUN_COUNT_ASCENDING,
    EVERYTHING_SORT_RUN_COUNT_DESCENDING = sdk_sys::EVERYTHING_SORT_RUN_COUNT_DESCENDING,
    EVERYTHING_SORT_DATE_RECENTLY_CHANGED_ASCENDING =
        sdk_sys::EVERYTHING_SORT_DATE_RECENTLY_CHANGED_ASCENDING,
    EVERYTHING_SORT_DATE_RECENTLY_CHANGED_DESCENDING =
        sdk_sys::EVERYTHING_SORT_DATE_RECENTLY_CHANGED_DESCENDING,
    EVERYTHING_SORT_DATE_ACCESSED_ASCENDING = sdk_sys::EVERYTHING_SORT_DATE_ACCESSED_ASCENDING,
    EVERYTHING_SORT_DATE_ACCESSED_DESCENDING = sdk_sys::EVERYTHING_SORT_DATE_ACCESSED_DESCENDING,
    EVERYTHING_SORT_DATE_RUN_ASCENDING = sdk_sys::EVERYTHING_SORT_DATE_RUN_ASCENDING,
    EVERYTHING_SORT_DATE_RUN_DESCENDING = sdk_sys::EVERYTHING_SORT_DATE_RUN_DESCENDING,
}

impl Default for SortType {
    fn default() -> Self {
        Self::EVERYTHING_SORT_NAME_ASCENDING
    }
}

/// The Everything_SetSort function sets how the results should be ordered.
///
/// # Arguments
/// * `sort_type` - The sort type, should be one of the values named `EVERYTHING_SORT_*`.
///
/// # Remarks
/// - The default sort is `EVERYTHING_SORT_NAME_ASCENDING` (1). This sort is free.
/// - Using fast sorts is recommended, fast sorting is instant.
/// - It is possible the sort is not supported, in which case after you have received your results you should
///   call `Everything_GetResultListSort` to determine the sorting actually used.
/// - This function MUST be called before `Everything_Query`.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_SetSort(sort_type: SortType) {
    unsafe { sdk_sys::Everything_SetSort(sort_type as u32) }
}

bitflags! {
    #[repr(transparent)] // TODO: should i?
    #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
    pub struct RequestFlags: u32 {
        const EVERYTHING_REQUEST_FILE_NAME = sdk_sys::EVERYTHING_REQUEST_FILE_NAME;
        const EVERYTHING_REQUEST_PATH = sdk_sys::EVERYTHING_REQUEST_PATH;
        const EVERYTHING_REQUEST_FULL_PATH_AND_FILE_NAME = sdk_sys::EVERYTHING_REQUEST_FULL_PATH_AND_FILE_NAME;
        const EVERYTHING_REQUEST_EXTENSION = sdk_sys::EVERYTHING_REQUEST_EXTENSION;
        const EVERYTHING_REQUEST_SIZE = sdk_sys::EVERYTHING_REQUEST_SIZE;
        const EVERYTHING_REQUEST_DATE_CREATED = sdk_sys::EVERYTHING_REQUEST_DATE_CREATED;
        const EVERYTHING_REQUEST_DATE_MODIFIED = sdk_sys::EVERYTHING_REQUEST_DATE_MODIFIED;
        const EVERYTHING_REQUEST_DATE_ACCESSED = sdk_sys::EVERYTHING_REQUEST_DATE_ACCESSED;
        const EVERYTHING_REQUEST_ATTRIBUTES = sdk_sys::EVERYTHING_REQUEST_ATTRIBUTES;
        const EVERYTHING_REQUEST_FILE_LIST_FILE_NAME = sdk_sys::EVERYTHING_REQUEST_FILE_LIST_FILE_NAME;
        const EVERYTHING_REQUEST_RUN_COUNT = sdk_sys::EVERYTHING_REQUEST_RUN_COUNT;
        const EVERYTHING_REQUEST_DATE_RUN = sdk_sys::EVERYTHING_REQUEST_DATE_RUN;
        const EVERYTHING_REQUEST_DATE_RECENTLY_CHANGED = sdk_sys::EVERYTHING_REQUEST_DATE_RECENTLY_CHANGED;
        const EVERYTHING_REQUEST_HIGHLIGHTED_FILE_NAME = sdk_sys::EVERYTHING_REQUEST_HIGHLIGHTED_FILE_NAME;
        const EVERYTHING_REQUEST_HIGHLIGHTED_PATH = sdk_sys::EVERYTHING_REQUEST_HIGHLIGHTED_PATH;
        const EVERYTHING_REQUEST_HIGHLIGHTED_FULL_PATH_AND_FILE_NAME = sdk_sys::EVERYTHING_REQUEST_HIGHLIGHTED_FULL_PATH_AND_FILE_NAME;
    }
}

impl Default for RequestFlags {
    fn default() -> Self {
        Self::EVERYTHING_REQUEST_FILE_NAME | Self::EVERYTHING_REQUEST_PATH
    }
}

/// The `Everything_SetRequestFlags` function sets the desired result data.
///
/// # Arguments
/// * `request_flags` - The request flags, can be zero or more of the flags named `EVERYTHING_SORT_*`.
///
/// # Remarks
/// - Make sure you include `EVERYTHING_REQUEST_FILE_NAME` and `EVERYTHING_REQUEST_PATH` if you want
///   the result file name information returned.
/// - The default request flags are `EVERYTHING_REQUEST_FILE_NAME` | `EVERYTHING_REQUEST_PATH` (0x00000003).
/// - When the default flags (`EVERYTHING_REQUEST_FILE_NAME` | `EVERYTHING_REQUEST_PATH`) are used
///   the SDK will use the old version 1 query.
/// - When any other flags are used the new version 2 query will be tried first, and then fall back
///   to version 1 query.
/// - It is possible the requested data is not available, in which case after you have received
///   your results you should call `Everything_GetResultListRequestFlags` to determine the available
///   result data.
/// - This function MUST be called before `Everything_Query`.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_SetRequestFlags(request_flags: RequestFlags) {
    unsafe { sdk_sys::Everything_SetRequestFlags(request_flags.bits()) }
}

// --- read search state ---

/// The `Everything_GetMatchPath` function returns the state of the match full path switch.
///
/// # Return
/// Returns `true` if match full path is enabled, else `false`.
///
/// # Remarks
/// - Get the internal state of the match full path switch.
/// - The default state is `false`, or disabled.
pub fn Everything_GetMatchPath() -> bool {
    let enabled = unsafe { sdk_sys::Everything_GetMatchPath() };
    lower_bool(enabled)
}

/// The `Everything_GetMatchCase` function returns the match case state.
///
/// # Return
/// Returns the match case state, `true` if the match case is enabled, `false` if disabled.
///
/// # Remarks
/// - Get the internal state of the match case switch.
/// - The default state is `false`, or disabled.
pub fn Everything_GetMatchCase() -> bool {
    let enabled = unsafe { sdk_sys::Everything_GetMatchCase() };
    lower_bool(enabled)
}

/// The `Everything_GetMatchWholeWord` function returns the match whole word state.
///
/// # Return
/// Returns the match whole word state, `true` if the the match whole word is
/// enabled, `false` if disabled.
///
/// # Remarks
/// - The default state is `false`, or disabled.
pub fn Everything_GetMatchWholeWord() -> bool {
    let enabled = unsafe { sdk_sys::Everything_GetMatchWholeWord() };
    lower_bool(enabled)
}

/// The `Everything_GetRegex` function returns the regex state.
///
/// # Return
/// Returns the regex state, `true` if the the regex is enabled, `false` if disabled.
///
/// # Remarks
/// - The default state is `false`, or disabled.
pub fn Everything_GetRegex() -> bool {
    let enabled = unsafe { sdk_sys::Everything_GetRegex() };
    lower_bool(enabled)
}

/// The `Everything_GetMax` function returns the maximum number of results state.
///
/// # Return
/// The return value is the maximum number of results state. The function returns
/// u32::MAX (0xFFFFFFFF) if all results should be returned.
///
/// # Remarks
/// - The default state is u32::MAX (0xFFFFFFFF), or all results.
pub fn Everything_GetMax() -> u32 {
    unsafe { sdk_sys::Everything_GetMax() }
}

/// The `Everything_GetOffset` function returns the first item offset of the available results.
///
/// # Return
/// Returns the first item offset.
///
/// # Remarks
/// - The default offset is 0.
pub fn Everything_GetOffset() -> u32 {
    unsafe { sdk_sys::Everything_GetOffset() }
}

/// The `Everything_GetSearch` function retrieves the search text to use for the next call
/// to `Everything_Query`.
///
/// # Return
/// If the function should not fail.
///
/// # Remarks
/// - Get the internal state of the search text.
/// - The default string is an empty string.
pub fn Everything_GetSearch() -> OsString {
    let ptr = unsafe { sdk_sys::Everything_GetSearchW() };
    assert!(!ptr.is_null());
    // SAFETY: now ptr is non-null, and it is null terminated string return
    // from `Everything_GetSearchW`
    unsafe { U16CStr::from_ptr_str(ptr.as_ptr()) }.to_os_string()
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Primitive)]
#[allow(non_camel_case_types)]
pub enum LastError {
    EVERYTHING_OK = sdk_sys::EVERYTHING_OK, // no error detected
    EVERYTHING_ERROR_MEMORY = sdk_sys::EVERYTHING_ERROR_MEMORY, // out of memory.
    EVERYTHING_ERROR_IPC = sdk_sys::EVERYTHING_ERROR_IPC, // Everything search client is not running
    EVERYTHING_ERROR_REGISTERCLASSEX = sdk_sys::EVERYTHING_ERROR_REGISTERCLASSEX, // unable to register window class.
    EVERYTHING_ERROR_CREATEWINDOW = sdk_sys::EVERYTHING_ERROR_CREATEWINDOW, // unable to create listening window
    EVERYTHING_ERROR_CREATETHREAD = sdk_sys::EVERYTHING_ERROR_CREATETHREAD, // unable to create listening thread
    EVERYTHING_ERROR_INVALIDINDEX = sdk_sys::EVERYTHING_ERROR_INVALIDINDEX, // invalid index
    EVERYTHING_ERROR_INVALIDCALL = sdk_sys::EVERYTHING_ERROR_INVALIDCALL,   // invalid call
    EVERYTHING_ERROR_INVALIDREQUEST = sdk_sys::EVERYTHING_ERROR_INVALIDREQUEST, // invalid request data, request data first.
    EVERYTHING_ERROR_INVALIDPARAMETER = sdk_sys::EVERYTHING_ERROR_INVALIDPARAMETER, // bad parameter.
}

/// The `Everything_GetLastError` function retrieves the last-error code value.
///
/// It will **keep** the _LAST_ error (maybe OK), unless the [`Everything_Reset`] or else is called.
///
/// Note that if some function was called and done successfully, the _last error_ may OR may not
/// be set or updated to [`LastError::EVERYTHING_OK`]. The SDK makes no guarantees about this
/// behavior.
///
/// So call this when the document of the api function explicitly mentions "To get extended error
/// information, call `Everything_GetLastError`."
pub fn Everything_GetLastError() -> LastError {
    let last_error = unsafe { sdk_sys::Everything_GetLastError() };
    LastError::try_from(last_error).expect("it should be a valid LastError number")
}

/// The `Everything_GetReplyWindow` function returns the current reply window for the IPC query reply.
///
/// # Return
/// Returns the current reply window.
///
/// # Remarks
/// - The default reply window is 0, or no reply window.
///
/// TODO: These functions coupled with the IPC mechanism that is `WM_COPYDATA` in Win32 API.
/// ...
#[cfg_attr(not(feature = "raw"), allow(dead_code))]
pub fn Everything_GetReplyWindow() -> HWND {
    unsafe { sdk_sys::Everything_GetReplyWindow() }
}

/// The `Everything_GetReplyID` function returns the current reply identifier for the IPC query reply.
///
/// # Return
/// The return value is the current reply identifier.
///
/// # Remarks
/// - The default reply identifier is 0.
///
/// TODO: These functions coupled with the IPC mechanism that is `WM_COPYDATA` in Win32 API.
/// ...
#[cfg_attr(not(feature = "raw"), allow(dead_code))]
pub fn Everything_GetReplyID() -> u32 {
    unsafe { sdk_sys::Everything_GetReplyID() }
}

/// The `Everything_GetSort` function returns the desired sort order for the results.
///
/// # Return
/// Returns one of the sort types.
///
/// # Remarks
/// - The default sort is `EVERYTHING_SORT_NAME_ASCENDING` (1)
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetSort() -> SortType {
    let sort_type = unsafe { sdk_sys::Everything_GetSort() };
    SortType::try_from(sort_type).expect("it should be a valid SortType number")
}

/// The `Everything_GetRequestFlags` function returns the desired result data flags.
///
/// # Return
/// Returns zero or more of the request flags.
///
/// # Remarks
/// - The default request flags are `EVERYTHING_REQUEST_FILE_NAME` | `EVERYTHING_REQUEST_PATH`,
///   that is (0x00000003).
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetRequestFlags() -> RequestFlags {
    let request_flags = unsafe { sdk_sys::Everything_GetRequestFlags() };
    RequestFlags::from_bits(request_flags).expect("unknown bits should not be set")
}

// --- execute query ---

/// The `Everything_Query` function executes an Everything IPC query with the current search state.
///
/// # Arguments
/// * `wait` - should the function wait for the results or return immediately.
///   Set this to `true` to send the IPC Query and wait for the results. (Normally)
///   Set this to `false` to post the IPC Query and return immediately.
///
/// # Return
/// If succeeds, return `true`, else `false`.
/// To get extended error information, call `Everything_GetLastError`
///
/// # Remarks
/// - If wait is `false` you MUST call `Everything_SetReplyWindow` before calling `Everything_Query`.
///   Use the `Everything_IsQueryReply` function to check for query replies.
/// - Optionally call the `Everything_Set*` functions to set the search state before
///   calling `Everything_Query`.
/// - The search state is not modified from a call to `Everything_Query`.
/// - If you want to know the default search state, see `Everything_Reset` for it.
pub fn Everything_Query(wait: bool) -> bool {
    let wait = if wait { TRUE } else { FALSE };
    let success = unsafe { sdk_sys::Everything_QueryW(wait) };
    lower_bool(success)
}

// --- query reply ---

/// The `Everything_IsQueryReply` function checks if the specified window message is a query reply.
///
/// # Arguments
/// * `u_msg` - Specifies the message identifier. (uMsg as `UINT` in winapi)
/// * `w_param` - Specifies additional information about the message. (wParam as `WPARAM` in winapi)
/// * `l_param` - Specifies additional information about the message. (lParam as `LPARAM` in winapi)
/// * `n_id` - The unique identifier specified with `Everything_SetReplyID`, or 0 for the default ID.
///   This is the underlying value used to compare with the `dwData` member of the `COPYDATASTRUCT`
///   if the message is `WM_COPYDATA`.
///
/// # Return
/// if the message is a query reply, return `true`.
/// if the function fails, return `false`. To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - This function checks if the message is a `WM_COPYDATA` message. If the message is a `WM_COPYDATA`
///   message the function checks if the ReplyID matches the `dwData` member of the `COPYDATASTRUCT`.
///   If they match the function makes a copy of the query results.
/// - You MUST call `Everything_IsQueryReply` in the windows message handler to check for an IPC query
///   reply if you call `Everything_Query` with `wait` set to `false`.
/// - For your `WindowProc` function, if this function returns `true`, you should return `true`.
///   Ref: [WindowProc](https://en.wikipedia.org/wiki/WindowProc)
/// - If this function `true` you can call the other functions (like `Everything_GetResultPath`) to
///   read the results.
///
/// TODO: These functions coupled with the IPC mechanism that is `WM_COPYDATA` in Win32 API.
/// - `Everything_IsQueryReply`, `Everything_SetReplyWindow`, `Everything_GetReplyWindow`,
///   `Everything_SetReplyID`, `Everything_GetReplyID`,
/// - [Using Data Copy](https://learn.microsoft.com/en-us/windows/win32/dataxchg/using-data-copy)
/// - [Windows and Messages](https://learn.microsoft.com/en-us/windows/win32/api/_winmsg/)
/// - [Window Procedures](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-procedures)
#[cfg_attr(not(feature = "raw"), allow(dead_code))]
pub fn Everything_IsQueryReply(u_msg: UINT, w_param: WPARAM, l_param: LPARAM, n_id: u32) -> bool {
    let is_reply = unsafe { sdk_sys::Everything_IsQueryReply(u_msg, w_param, l_param, n_id) };
    lower_bool(is_reply)
}

// --- write result state ---

/// The `Everything_SortResultsByPath` function sorts the current results by path, then file name.
/// SortResultsByPath is **CPU Intensive**. Sorting by path can take several seconds.
///
/// # Remarks
/// - The default result list contains no results.
/// - Call `Everything_Query` to retrieve the result list prior to a call to `Everything_SortResultsByPath`.
/// - For improved performance, use `Everything_SetSort`.
pub fn Everything_SortResultsByPath() {
    unsafe { sdk_sys::Everything_SortResultsByPath() }
}

// --- read result state ---

/// The `Everything_GetNumFileResults` function returns the number of visible file results.
///
/// # Return
/// Returns the number of visible file results.
/// If the function fails the return value is 0. (weird)
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You must call `Everything_Query` before calling `Everything_GetNumFileResults`.
/// - Use [`Everything_GetTotFileResults`] to retrieve the total number of file results.
/// - If the result offset state is 0, and the max result is u32::MAX (0xFFFFFFFF),
///   `Everything_GetNumFileResults` will return the total number of file results
///   and all file results will be visible.
/// - `Everything_GetNumFileResults` is not supported when using [`Everything_SetRequestFlags`]
pub fn Everything_GetNumFileResults() -> u32 {
    unsafe { sdk_sys::Everything_GetNumFileResults() }
}

/// The `Everything_GetNumFolderResults` function returns the number of visible
/// folder results.
///
/// # Return
/// Returns the number of visible file results.
/// If the function fails the return value is 0. (weird)
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You must call `Everything_Query` before calling `Everything_GetNumFolderResults`.
/// - Use [`Everything_GetTotFolderResults`] to retrieve the total number of folder results.
/// - If the result offset state is 0, and the max result is u32::MAX (0xFFFFFFFF),
///   `Everything_GetNumFolderResults` will return the total number of folder results and all
///   folder results will be visible.
/// - `Everything_GetNumFolderResults` is not supported when using [`Everything_SetRequestFlags`].
pub fn Everything_GetNumFolderResults() -> u32 {
    unsafe { sdk_sys::Everything_GetNumFolderResults() }
}

/// The `Everything_GetNumResults` function returns the number of visible file and
/// folder results.
///
/// # Return
/// Returns the number of visible file and folder results.
/// If the function fails the return value is 0. (weird)
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You must call `Everything_Query` before calling `Everything_GetNumResults`.
/// - Use `Everything_GetTotResults` to retrieve the total number of file and folder results.
/// - If the result offset state is 0, and the max result is u32::MAX (0xFFFFFFFF),
///   `Everything_GetNumResults` will return the total number of file and folder results
///   and all file and folder results will be visible.
pub fn Everything_GetNumResults() -> u32 {
    unsafe { sdk_sys::Everything_GetNumResults() }
}

/// The `Everything_GetTotFileResults` function returns the total number of file results.
///
/// # Return
/// Returns the total number of file results.
/// If the function fails the return value is 0. (weird)
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You must call `Everything_Query` before calling `Everything_GetTotFileResults`.
/// - Use `Everything_GetNumFileResults` to retrieve the number of visible file results.
/// - Use the result offset and max result values to limit the number of visible results.
/// - `Everything_GetTotFileResults` is not supported when using [`Everything_SetRequestFlags`].
pub fn Everything_GetTotFileResults() -> u32 {
    unsafe { sdk_sys::Everything_GetTotFileResults() }
}

/// The `Everything_GetTotFolderResults` function returns the total number of folder results.
///
/// # Return
/// Returns the total number of folder results.
/// If the function fails the return value is 0. (weird)
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You must call `Everything_Query` before calling `Everything_GetTotFolderResults`.
/// - Use `Everything_GetNumFolderResults` to retrieve the number of visible folder results.
/// - Use the result offset and max result values to limit the number of visible results.
/// - `Everything_GetTotFolderResults` is not supported when using [`Everything_SetRequestFlags`].
pub fn Everything_GetTotFolderResults() -> u32 {
    unsafe { sdk_sys::Everything_GetTotFolderResults() }
}

/// The `Everything_GetTotResults` function returns the total number of file and folder results.
///
/// # Return
/// Returns the total number of file and folder results.
/// If the function fails the return value is 0. (weird)
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You must call `Everything_Query` before calling `Everything_GetTotResults`.
/// - Use `Everything_GetNumResults` to retrieve the number of visible file and folder results.
/// - Use the result offset and max result values to limit the number of visible results.
pub fn Everything_GetTotResults() -> u32 {
    unsafe { sdk_sys::Everything_GetTotResults() }
}

/// The `Everything_IsVolumeResult` function determines if the visible result is the root
/// folder of a volume.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// - The function returns `true`, if the visible result is a volume. (For example: C:)
/// - The function returns `false`, if the visible result is a folder or file.
///   (For example: C:\ABC.123)
/// - If the function fails the return value is `false`. (weird)
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You can only call this function for a visible result. To determine if a result is
///   visible use the `Everything_GetNumFileResults` function.
pub fn Everything_IsVolumeResult(index: u32) -> bool {
    let result = unsafe { sdk_sys::Everything_IsVolumeResult(index) };
    lower_bool(result)
}

/// The `Everything_IsFolderResult` function determines if the visible result is a folder.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// - The function returns `true`, if the visible result is a folder or volume.
///   (For example: C: or c:\WINDOWS)
/// - The function returns `false`, if the visible result is a file.
///   (For example: C:\ABC.123)
/// - If the function fails the return value is `false`. (weird)
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You can only call this function for a visible result. To determine if a result is
///   visible use the `Everything_GetNumFileResults` function.
pub fn Everything_IsFolderResult(index: u32) -> bool {
    let result = unsafe { sdk_sys::Everything_IsFolderResult(index) };
    lower_bool(result)
}

/// The `Everything_IsFileResult` function determines if the visible result is file.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// - The function returns `true`, if the visible result is a file.
///   (For example: C:\ABC.123)
/// - The function returns `false`, if the visible result is a folder or volume.
///   (For example: C: or c:\WINDOWS)
/// - If the function fails the return value is `false`. (weird)
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You can only call this function for a visible result. To determine if a result is
///   visible use the `Everything_GetNumFileResults` function.
pub fn Everything_IsFileResult(index: u32) -> bool {
    let result = unsafe { sdk_sys::Everything_IsFileResult(index) };
    lower_bool(result)
}

/// The `Everything_GetResultFileName` function retrieves the file name part only of the
/// visible result.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// If the function fails the return value is None.
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - The function is NOT faster than `Everything_GetResultFullPathName` now, as this function
///   DO the memory copying. If you want no memory copying, you should use the no-copy ffi
///   function [`everything_sdk_sys::Everything_GetResultFileNameW`] directly.
/// - The function returns a pointer to an internal structure that is only valid until the next
///   call to `Everything_Query` or `Everything_Reset`.
/// - You can only call this function for a visible result. To determine if a result is visible
///   use the `Everything_GetNumFileResults` function.
pub fn Everything_GetResultFileName(index: u32) -> Option<OsString> {
    let ptr = unsafe { sdk_sys::Everything_GetResultFileNameW(index) };
    if ptr.is_null() {
        None
    } else {
        // SAFETY: now ptr is non-null, and it is null terminated string of TCHARs return
        // from `Everything_GetResultFileNameW`
        Some(unsafe { U16CStr::from_ptr_str(ptr.as_ptr()) }.to_os_string())
    }
}

/// The `Everything_GetResultPath` function retrieves the path part of the visible result.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// If the function fails the return value is None.
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - The function is NOT faster than `Everything_GetResultFullPathName` now, as this function
///   DO the memory copying. If you want no memory copying, you should use the no-copy ffi
///   function [`everything_sdk_sys::Everything_GetResultPathW`] directly.
/// - The function returns a pointer to an internal structure that is only valid until the next
///   call to `Everything_Query` or `Everything_Reset`.
/// - You can only call this function for a visible result. To determine if a result is visible
///   use the `Everything_GetNumFileResults` function.
pub fn Everything_GetResultPath(index: u32) -> Option<OsString> {
    let ptr = unsafe { sdk_sys::Everything_GetResultPathW(index) };
    if ptr.is_null() {
        None
    } else {
        // SAFETY: now ptr is non-null, and it is null terminated string of TCHARs return
        // from `Everything_GetResultPathW`
        Some(unsafe { U16CStr::from_ptr_str(ptr.as_ptr()) }.to_os_string())
    }
}

/// The `Everything_GetResultFullPathName` function retrieves the full path and file name
/// of the visible result.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
/// * `out_buf` - Buffer that will receive the text. If the string is as long or longer than
///   the buffer, the string is truncated and terminated with a NULL character.
///
/// # Return
/// Returns the number of wchar_t excluding the null terminator copied into lpString.
/// If the function fails, return None. To get extended error information, call
/// `Everything_GetLastError`.
///
/// # Remarks
/// - You can only call this function for a visible result. To determine if a result is visible
///   use the Everything_GetNumFileResults function.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultFullPathName(index: u32, out_buf: &mut [u16]) -> Option<NonZeroU32> {
    let buf_ptr = out_buf.as_mut_ptr();
    let buf_size = u32::try_from(out_buf.len()).expect("buf size should not be greater than u32");
    // If lpString is not NULL, the return value is the number of wchar_t excluding
    // the null terminator copied into lpString.
    let number_of_wchar_without_null_terminator =
        unsafe { sdk_sys::Everything_GetResultFullPathNameW(index, PWSTR(buf_ptr), buf_size) };
    NonZeroU32::new(number_of_wchar_without_null_terminator)
}

/// The `Everything_GetResultFullPathNameSizeHint` function get the buffer size hint
/// for calling `Everything_GetResultFullPathName` including the null terminator.
/// You can new a Vec<MaybeUninit<u16>> or array in stack with this size hint as the
/// initial capacity of the buffer `out_buf` in `Everything_GetResultFullPathName`.
///
/// This function is not native in Everything-SDK, but part of the functionality
/// stripped out of the ffi function `Everything_GetResultFullPathNameW` when its
/// out buffer pointer is set to null.
///
/// # Examples
/// ```no_run
/// use everything_sdk::raw::*;
/// let result_index = 0;
/// let size_hint = u32::from(Everything_GetResultFullPathNameSizeHint(result_index).unwrap());
/// let mut buf = vec![0; size_hint as usize];
/// let n_wchar = u32::from(Everything_GetResultFullPathName(result_index, &mut buf).unwrap());
/// assert_eq!(size_hint, n_wchar + 1);
/// ```
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// - Returns the size of the wchar_t buffer (including null terminator) needed to
///   store the full path and file name of the visible result.
/// - If the function fails, return None.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You can only call this function for a visible result. To determine if a result is visible
///   use the Everything_GetNumFileResults function.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultFullPathNameSizeHint(index: u32) -> Option<NonZeroU32> {
    // If lpString is NULL, the return value is the number of wchar_t excluding the
    // null terminator needed to store the full path and file name of the visible result.
    let wchars_without_null_terminator_size_hint =
        unsafe { sdk_sys::Everything_GetResultFullPathNameW(index, PWSTR::null(), 0) };
    if wchars_without_null_terminator_size_hint == 0 {
        None
    } else {
        let size_hint = wchars_without_null_terminator_size_hint + 1;
        NonZeroU32::new(size_hint)
    }
}

/// The `Everything_GetResultListSort` function returns the actual sort order for the results.
///
/// # Return
/// Returns one of the sort types.
///
/// # Remarks
/// - `Everything_GetResultListSort` must be called after calling `Everything_Query`.
/// - If no desired sort order was specified the result list is sorted by
///   [`SortType::EVERYTHING_SORT_NAME_ASCENDING`].
/// - The result list sort may differ to the desired sort specified in `Everything_SetSort`.
///
/// # Requirements
/// Maybe require Everything 1.4.1 or later indicated in source code.
pub fn Everything_GetResultListSort() -> SortType {
    let sort_type = unsafe { sdk_sys::Everything_GetResultListSort() };
    SortType::try_from(sort_type).expect("it should be a valid SortType number")
}

/// The `Everything_GetResultListRequestFlags` function returns the flags of available result data.
///
/// # Return
/// Returns zero or more of the request flags.
///
/// # Remarks
/// - The requested result data may differ to the desired result data specified in
///   [`Everything_SetRequestFlags`].
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultListRequestFlags() -> RequestFlags {
    let request_flags = unsafe { sdk_sys::Everything_GetResultListRequestFlags() };
    RequestFlags::from_bits(request_flags).expect("unknown bits should not be set")
}

/// The `Everything_GetResultExtension` function retrieves the extension part of a visible
/// result.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// If the function fails the return value is None.
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You can only call this function for a visible result. To determine if a result is
///   visible use the `Everything_GetNumResults` function.
///
/// # Requirements
/// Maybe require Everything 1.4.1 or later indicated in source code.
pub fn Everything_GetResultExtension(index: u32) -> Option<OsString> {
    // The function returns a pointer to an internal structure that is only valid until
    // the next call to `Everything_Query`, `Everything_Reset` or `Everything_CleanUp`.
    let ptr = unsafe { sdk_sys::Everything_GetResultExtensionW(index) };
    if ptr.is_null() {
        None
    } else {
        // SAFETY: now ptr is non-null, and it is null terminated string of TCHARs return
        // from `Everything_GetResultExtensionW`
        Some(unsafe { U16CStr::from_ptr_str(ptr.as_ptr()) }.to_os_string())
    }
}

/// The `Everything_GetResultSize` function retrieves the size of a visible result.
///
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// The function returns result size if successful.
/// The function returns `None` if size information is unavailable.
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - `index` must be a valid visible result index. To determine if a result index is
///   visible use the `Everything_GetNumResults` function.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultSize(index: u32) -> Option<i64> {
    // Ref: https://github.com/retep998/winapi-rs/blob/0.3/README.md#how-do-i-create-an-instance-of-a-union
    let mut size: LARGE_INTEGER = 0;
    // lpSize is the pointer to a LARGE_INTEGER to hold the size of the result.
    let success = unsafe { sdk_sys::Everything_GetResultSize(index, &mut size) };
    match success {
        // SAFETY: If your compiler has built-in support for 64-bit integers, use the QuadPart member
        // to store the 64-bit integer.
        // Ref: https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-large_integer-r1#remarks
        // NOTE:
        // - If request flag `EVERYTHING_REQUEST_ATTRIBUTES` is set, GetResultSize for a folder will return 0.
        // - If not, GetResultSize for a folder will return -1. (wired)
        TRUE => Some(size),
        FALSE => None,
        _ => unreachable!(),
    }
}

/// Ref: <https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-filetime>
/// Ref: <https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-ularge_integer-r1>
fn convert_filetime_to_u64(filetime: FILETIME) -> u64 {
    // let mut time: ULARGE_INTEGER = 0;
    // // Irrelevant Tips: field `u` in ULARGE_INTEGER is type error(bug), it should be ULARGE_INTEGER_u
    // // instead of ULARGE_INTEGER_s.
    // let s_mut = unsafe { time.s_mut() };
    // s_mut.LowPart = filetime.dwLowDateTime;
    // s_mut.HighPart = filetime.dwHighDateTime;
    // unsafe { *time.QuadPart() }
    unsafe { std::mem::transmute(filetime) }
}

/// The `Everything_GetResultDateCreated` function retrieves the created date of a visible result.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// The function returns result created date (ftCreationTime) if successful.
/// (Similar to the u64 return value of [`std::os::windows::fs::MetadataExt::creation_time`], see
/// its docs for details.)
/// The function returns `None` if the date created information is unavailable.
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - `index` must be a valid visible result index. To determine if a result index is visible use
///   the `Everything_GetNumResults` function.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultDateCreated(index: u32) -> Option<u64> {
    let mut file_time = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    // lpDateCreated is the pointer to a FILETIME to hold the created date of the result.
    let success = unsafe { sdk_sys::Everything_GetResultDateCreated(index, &mut file_time) };
    match success {
        TRUE => Some(convert_filetime_to_u64(file_time)),
        FALSE => None,
        _ => unreachable!(),
    }
}

/// The `Everything_GetResultDateModified` function retrieves the modified date of a visible result.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// The function returns result modified date (ftLastWriteTime) if successful.
/// (Similar to the u64 return value of [`std::os::windows::fs::MetadataExt::last_write_time`], see
/// its docs for details.)
/// The function returns `None` if the modified date information is unavailable.
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - `index` must be a valid visible result index. To determine if a result index is visible use
///   the `Everything_GetNumResults` function.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultDateModified(index: u32) -> Option<u64> {
    let mut file_time = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    // lpDateModified is the pointer to a FILETIME to hold the modified date of the result.
    let success = unsafe { sdk_sys::Everything_GetResultDateModified(index, &mut file_time) };
    match success {
        TRUE => Some(convert_filetime_to_u64(file_time)),
        FALSE => None,
        _ => unreachable!(),
    }
}

/// The `Everything_GetResultDateAccessed` function retrieves the accessed date of a visible result.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// The function returns result accessed date (ftLastAccessTime) if successful.
/// (Similar to the u64 return value of [`std::os::windows::fs::MetadataExt::last_access_time`], see
/// its docs for details.)
/// The function returns `None` if the accessed date information is unavailable.
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - `index` must be a valid visible result index. To determine if a result index is visible use
///   the `Everything_GetNumResults` function.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultDateAccessed(index: u32) -> Option<u64> {
    let mut file_time = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    // lpDateAccessed is the pointer to a FILETIME to hold the accessed date of the result.
    let success = unsafe { sdk_sys::Everything_GetResultDateAccessed(index, &mut file_time) };
    match success {
        TRUE => Some(convert_filetime_to_u64(file_time)),
        FALSE => None,
        _ => unreachable!(),
    }
}

/// The `Everything_GetResultAttributes` function retrieves the attributes of a visible result.
///
/// Ref: <https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfileattributesw>
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// The function returns the u32 value for zero or more of FILE_ATTRIBUTE_* flags.
/// (You can use the constants like [`winapi::um::winnt::FILE_ATTRIBUTE_DIRECTORY`] to match it.
/// Ref: <https://learn.microsoft.com/en-us/windows/win32/fileio/file-attribute-constants>)
/// (Similar to the u32 return value of [`std::os::windows::fs::MetadataExt::file_attributes`], see
/// its docs for details.)
/// The function returns `None` if attribute information is unavailable.
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - `index` must be a valid visible result index. To determine if a result index is visible use
///   the `Everything_GetNumResults` function.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultAttributes(index: u32) -> Option<u32> {
    let attr = unsafe { sdk_sys::Everything_GetResultAttributes(index) };
    // The function returns `INVALID_FILE_ATTRIBUTES` if attribute information is unavailable.
    if attr == INVALID_FILE_ATTRIBUTES {
        None
    } else {
        Some(attr)
    }
}

/// The `Everything_GetResultFileListFileName` function retrieves the file list full path
/// and filename of the visible result.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// - If the function fails the return value is None.
///   To get extended error information, call `Everything_GetLastError`.
/// - If the result specified by `index` is not in a file list, then the filename returned
///   is an empty string.
///
/// # Remarks
/// - `index` must be a valid visible result index. To determine if a result is visible use
///   the `Everything_GetNumFileResults` function.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultFileListFileName(index: u32) -> Option<OsString> {
    // The function returns a pointer to an internal structure that is only valid until
    // the next call to `Everything_Query` or `Everything_Reset`.
    let ptr = unsafe { sdk_sys::Everything_GetResultFileListFileNameW(index) };
    if ptr.is_null() {
        None
    } else {
        // SAFETY: now ptr is non-null, and it is null terminated string of TCHARs return
        // from `Everything_GetResultFileListFileNameW`
        Some(unsafe { U16CStr::from_ptr_str(ptr.as_ptr()) }.to_os_string())
    }
}

/// The `Everything_GetResultRunCount` function retrieves the number of times a visible
/// result has been run from Everything.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// - The function returns the number of times the result has been run from Everything.
///   (maybe zero?)
/// - The function returns 0 if the run count information is unavailable.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - `index` must be a valid visible result index. To determine if a result index is visible
///   use the `Everything_GetNumResults` function.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultRunCount(index: u32) -> u32 {
    unsafe { sdk_sys::Everything_GetResultRunCount(index) }
}

/// The `Everything_GetResultDateRun` function retrieves the run date of a visible result.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// - The function returns result run date if successful.
/// - The function returns `None` if the run date information is unavailable.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - `index` must be a valid visible result index. To determine if a result index is visible
///   use the `Everything_GetNumResults` function.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultDateRun(index: u32) -> Option<u64> {
    let mut file_time = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    // lpDateRun is the pointer to a FILETIME to hold the run date of the result.
    let success = unsafe { sdk_sys::Everything_GetResultDateRun(index, &mut file_time) };
    match success {
        TRUE => Some(convert_filetime_to_u64(file_time)),
        FALSE => None,
        _ => unreachable!(),
    }
}

/// The `Everything_GetResultDateRecentlyChanged` function retrieves the recently changed
/// date of a visible result.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// - The function returns result run date if successful.
/// - The function returns `None` if the recently changed date information is unavailable.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - `index` must be a valid visible result index. To determine if a result index is visible
///   use the `Everything_GetNumResults` function.
/// - The date recently changed is the date and time of when the result was changed in
///   the Everything index, this could be from a file creation, rename, attribute or content
///   change.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultDateRecentlyChanged(index: u32) -> Option<u64> {
    let mut file_time = FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    };
    // lpDateRecentlyChanged is the pointer to a FILETIME to hold the recently changed date of the result.
    let success =
        unsafe { sdk_sys::Everything_GetResultDateRecentlyChanged(index, &mut file_time) };
    match success {
        TRUE => Some(convert_filetime_to_u64(file_time)),
        FALSE => None,
        _ => unreachable!(),
    }
}

/// The `Everything_GetResultHighlightedFileName` function retrieves the highlighted file
/// name part of the visible result.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// If the function fails the return value is None.
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You can only call this function for a visible result. To determine if a result is visible
///   use the `Everything_GetNumFileResults` function.
/// - Text inside a \* quote is highlighted, two consecutive \*'s is a single literal \*.
/// - For example, in the highlighted text: abc \*123\* the 123 part is highlighted.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultHighlightedFileName(index: u32) -> Option<OsString> {
    // The function returns a pointer to an internal structure that is only valid until
    // the next call to `Everything_Query` or `Everything_Reset`.
    let ptr = unsafe { sdk_sys::Everything_GetResultHighlightedFileNameW(index) };
    if ptr.is_null() {
        None
    } else {
        // SAFETY: now ptr is non-null, and it is null terminated string of TCHARs return
        // from `Everything_GetResultHighlightedFileNameW`
        Some(unsafe { U16CStr::from_ptr_str(ptr.as_ptr()) }.to_os_string())
    }
}

/// The `Everything_GetResultHighlightedPath` function retrieves the highlighted path part
/// of the visible result.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// If the function fails the return value is None.
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You can only call this function for a visible result. To determine if a result is visible
///   use the `Everything_GetNumFileResults` function.
/// - Text inside a \* quote is highlighted, two consecutive \*'s is a single literal \*.
/// - For example, in the highlighted text: abc \*123\* the 123 part is highlighted.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultHighlightedPath(index: u32) -> Option<OsString> {
    // The function returns a pointer to an internal structure that is only valid until
    // the next call to `Everything_Query` or `Everything_Reset`.
    let ptr = unsafe { sdk_sys::Everything_GetResultHighlightedPathW(index) };
    if ptr.is_null() {
        None
    } else {
        // SAFETY: now ptr is non-null, and it is null terminated string of TCHARs return
        // from `Everything_GetResultHighlightedPathW`
        Some(unsafe { U16CStr::from_ptr_str(ptr.as_ptr()) }.to_os_string())
    }
}

/// The `Everything_GetResultHighlightedFullPathAndFileName` function retrieves the highlighted
/// full path and file name of the visible result.
///
/// # Arguments
/// * `index` - Zero based index of the visible result.
///
/// # Return
/// If the function fails the return value is None.
/// To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - You can only call this function for a visible result. To determine if a result is visible
///   use the `Everything_GetNumFileResults` function.
/// - Text inside a \* quote is highlighted, two consecutive \*'s is a single literal \*.
/// - For example, in the highlighted text: abc \*123\* the 123 part is highlighted.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetResultHighlightedFullPathAndFileName(index: u32) -> Option<OsString> {
    // The function returns a pointer to an internal structure that is only valid until
    // the next call to `Everything_Query` or `Everything_Reset`.
    let ptr = unsafe { sdk_sys::Everything_GetResultHighlightedFullPathAndFileNameW(index) };
    if ptr.is_null() {
        None
    } else {
        // SAFETY: now ptr is non-null, and it is null terminated string of TCHARs return
        // from `Everything_GetResultHighlightedFullPathAndFileNameW`
        Some(unsafe { U16CStr::from_ptr_str(ptr.as_ptr()) }.to_os_string())
    }
}

// --- reset state and free any allocated memory ---

/// The `Everything_Reset` function resets the result list and search state to the default
/// state, freeing any allocated memory by the library.
///
/// # Remarks
/// - Calling `Everything_SetSearch` frees the old search and allocates the new search string.
/// - Calling `Everything_Query` frees the old result list and allocates the new result list.
/// - Calling `Everything_Reset` frees the current search and current result list.
/// - The default state:
///    + Everything_SetSearch("");
///    + Everything_SetMatchPath(false);
///    + Everything_SetMatchCase(false);
///    + Everything_SetMatchWholeWord(false);
///    + Everything_SetRegex(false);
///    + Everything_SetMax(u32::MAX);
///    + Everything_SetOffset(0);
///    + Everything_SetReplyWindow(std::ptr::null_mut());
///    + Everything_SetReplyID(0);
pub fn Everything_Reset() {
    unsafe { sdk_sys::Everything_Reset() }
}

/// The `Everything_CleanUp` function frees any allocated memory by the library.
///
/// # Remarks
/// - You should call `Everything_CleanUp` to free any memory allocated by the Everything SDK.
/// - `Everything_CleanUp` should be the last call to the Everything SDK.
/// - Call `Everything_Reset` to free any allocated memory for the current search and results.
/// - `Everything_Reset` will also reset the search and result state to their defaults.
/// - Calling `Everything_SetSearch` frees the old search and allocates the new search string.
/// - Calling `Everything_Query` frees the old result list and allocates the new result list.
pub fn Everything_CleanUp() {
    unsafe { sdk_sys::Everything_CleanUp() }
}

/// The `Everything_GetMajorVersion` function retrieves the major version number of Everything.
///
/// # Return
/// - The function returns the major version number.
/// - The function returns 0 if major version information is unavailable.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - Everything uses the version format: `<major>.<minor>.<revision>.<build>`
/// - The build part is incremental and unique for all Everything versions.
///
/// # Requirements
/// Requires Everything 1.0.0.0 or later.
pub fn Everything_GetMajorVersion() -> Option<u32> {
    zero_or_ipc_error(unsafe { sdk_sys::Everything_GetMajorVersion() })
}

/// The `Everything_GetMinorVersion` function retrieves the minor version number of Everything.
///
/// # Return
/// - The function returns the minor version number.
/// - The function returns 0 if minor version information is unavailable.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - Everything uses the version format: `<major>.<minor>.<revision>.<build>`
/// - The build part is incremental and unique for all Everything versions.
///
/// # Requirements
/// Requires Everything 1.0.0.0 or later.
pub fn Everything_GetMinorVersion() -> Option<u32> {
    zero_or_ipc_error(unsafe { sdk_sys::Everything_GetMinorVersion() })
}

/// The `Everything_GetRevision` function retrieves the revision number of Everything.
///
/// # Return
/// - The function returns the revision number.
/// - The function returns 0 if revision information is unavailable.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - Everything uses the version format: `<major>.<minor>.<revision>.<build>`
/// - The build part is incremental and unique for all Everything versions.
///
/// # Requirements
/// Requires Everything 1.0.0.0 or later.
pub fn Everything_GetRevision() -> Option<u32> {
    zero_or_ipc_error(unsafe { sdk_sys::Everything_GetRevision() })
}

/// The `Everything_GetBuildNumber` function retrieves the build number of Everything.
///
/// # Return
/// - The function returns the build number.
/// - The function returns 0 if build information is unavailable.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - Everything uses the version format: `<major>.<minor>.<revision>.<build>`
/// - The build part is incremental and unique for all Everything versions.
///
/// # Requirements
/// Requires Everything 1.0.0.0 or later.
pub fn Everything_GetBuildNumber() -> Option<u32> {
    zero_or_ipc_error(unsafe { sdk_sys::Everything_GetBuildNumber() })
}

/// The `Everything_Exit` function requests Everything to exit.
///
/// # Return
/// - The function returns `true` if the exit request was successful.
/// - The function returns `false` if the request failed.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - Request Everything to save settings and data to disk and exit.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_Exit() -> Option<bool> {
    let exit_success = unsafe { sdk_sys::Everything_Exit() };
    lower_bool_or_ipc_error(exit_success)
}

/// Try closing `Everything` client and stoping `Everything` Windows service. (Unstable)
///
/// can be called as admin or standard user.
/// will self elevate if needed.
/// (this API NOT in .def)
///
/// # Return
/// If it does not make an attempt (that is it does nothing), return `false`.
/// If it makes an attempt (that is it calls the ffi function), whether it fails or not, return `true`.
#[cfg_attr(not(feature = "raw"), allow(dead_code))]
pub fn Everything_MSIExitAndStopService() -> bool {
    let result = unsafe { sdk_sys::Everything_MSIExitAndStopService(std::ptr::null_mut()) };
    match result {
        0 => true,
        1 => false,
        _ => unreachable!(),
    }
}

/// Try starting `Everything` Windows service. (Unstable)
///
/// MUST be called as an admin.
/// (this API NOT in .def)
///
/// # Return
/// If it does not make an attempt (that is it does nothing), return `false`.
/// If it makes an attempt(that is it calls the ffi function), whether it fails or not, return `true`.
#[cfg_attr(not(feature = "raw"), allow(dead_code))]
pub fn Everything_MSIStartService() -> bool {
    let result = unsafe { sdk_sys::Everything_MSIStartService(std::ptr::null_mut()) };
    match result {
        0 => true,
        1 => false,
        _ => unreachable!(),
    }
}

/// The `Everything_IsDBLoaded` function checks if the database has been fully loaded.
///
/// # Return
/// - The function returns `true` if the Everything database is fully loaded.
/// - The function returns `false` if the database has not fully loaded or if an error occurred.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - When Everything is loading, any queries will appear to return no results.
/// - Use `Everything_IsDBLoaded` to determine if the database has been loaded before
///   performing a query.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_IsDBLoaded() -> Option<bool> {
    let is_db_loaded = unsafe { sdk_sys::Everything_IsDBLoaded() };
    lower_bool_or_ipc_error(is_db_loaded)
}

/// The `Everything_IsAdmin` function checks if Everything is running as administrator
/// or as a standard user.
///
/// # Return
/// - The function returns `true` if the Everything is running as an administrator.
/// - The function returns `false` if Everything is running as a standard user OR an error
///   occurred. (weird)
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_IsAdmin() -> Option<bool> {
    let is_admin = unsafe { sdk_sys::Everything_IsAdmin() };
    lower_bool_or_ipc_error(is_admin)
}

/// The `Everything_IsAppData` function checks if Everything is saving settings and
/// data to `%APPDATA%\Everything` or to the same location as the `Everything.exe`.
///
/// # Return
/// - The function returns `true` if settings and data are saved in `%APPDATA%\Everything`.
/// - The function returns `false` if settings and data are saved to the same location
///   as the `Everything.exe` or if an error occurred. (weird)
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_IsAppData() -> Option<bool> {
    let is_app_data = unsafe { sdk_sys::Everything_IsAppData() };
    lower_bool_or_ipc_error(is_app_data)
}

/// The `Everything_RebuildDB` function requests Everything to forcefully rebuild
/// the Everything index.
///
/// # Return
/// - The function returns `true` if the request to forcefully rebuild the Everything
///   index was successful.
/// - The function returns `false` if an error occurred.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - Requesting a rebuild will mark all indexes as dirty and start the rebuild process.
/// - Use `Everything_IsDBLoaded` to determine if the database has been rebuilt before
///   performing a query.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_RebuildDB() -> Option<bool> {
    let success = unsafe { sdk_sys::Everything_RebuildDB() };
    lower_bool_or_ipc_error(success)
}

/// The `Everything_UpdateAllFolderIndexes` function requests Everything to rescan all
/// folder indexes.
///
/// # Return
/// - The function returns `true` if the request to rescan all folder indexes was successful.
/// - The function returns `false` if an error occurred.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - Everything will begin updating all folder indexes in the background.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_UpdateAllFolderIndexes() -> Option<bool> {
    let success = unsafe { sdk_sys::Everything_UpdateAllFolderIndexes() };
    lower_bool_or_ipc_error(success)
}

/// The `Everything_SaveDB` function requests Everything to save the index to disk.
///
/// # Return
/// - The function returns `true` if the request to save the Everything index to disk
///   was successful.
/// - The function returns `false` if an error occurred.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - The index is only saved to disk when you exit Everything.
/// - Call `Everything_SaveDB` to write the index to the file `Everything.db`
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_SaveDB() -> Option<bool> {
    // flush index to disk
    let success = unsafe { sdk_sys::Everything_SaveDB() };
    lower_bool_or_ipc_error(success)
}

/// The `Everything_SaveRunHistory` function requests Everything to save the run history
/// to disk.
///
/// # Return
/// - The function returns `true` if the request to save the run history to disk
///   was successful.
/// - The function returns `false` if an error occurred.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - The run history is only saved to disk when you close an Everything search window or
///   exit Everything.
/// - Call `Everything_RunHistory` to write the run history to the file `Run History.csv`
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_SaveRunHistory() -> Option<bool> {
    // flush run history to disk
    let success = unsafe { sdk_sys::Everything_SaveRunHistory() };
    lower_bool_or_ipc_error(success)
}

/// The `Everything_DeleteRunHistory` function deletes all run history.
///
/// # Return
/// - The function returns `true` if run history is cleared.
/// - The function returns `false` if an error occurred.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - Calling this function will clear all run history from memory and disk.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_DeleteRunHistory() -> Option<bool> {
    // clear run history
    let success = unsafe { sdk_sys::Everything_DeleteRunHistory() };
    lower_bool_or_ipc_error(success)
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Primitive)]
#[allow(non_camel_case_types)]
pub enum TargetMachine {
    EVERYTHING_TARGET_MACHINE_X86 = sdk_sys::EVERYTHING_TARGET_MACHINE_X86, // Target machine is x86 (32 bit).
    EVERYTHING_TARGET_MACHINE_X64 = sdk_sys::EVERYTHING_TARGET_MACHINE_X64, // Target machine is x64 (64 bit).
    EVERYTHING_TARGET_MACHINE_ARM = sdk_sys::EVERYTHING_TARGET_MACHINE_ARM, // Target machine is ARM.
}

impl Display for TargetMachine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetMachine::EVERYTHING_TARGET_MACHINE_X86 => write!(f, "x86"),
            TargetMachine::EVERYTHING_TARGET_MACHINE_X64 => write!(f, "x64"),
            TargetMachine::EVERYTHING_TARGET_MACHINE_ARM => write!(f, "arm"),
        }
    }
}

/// The `Everything_GetTargetMachine` function retrieves the target machine of Everything.
///
/// # Return
/// - The function returns one of the following:
///    + `EVERYTHING_TARGET_MACHINE_X86` (1) -> Target machine is x86 (32 bit).
///    + `EVERYTHING_TARGET_MACHINE_X64` (2) -> Target machine is x64 (64 bit).
///    + `EVERYTHING_TARGET_MACHINE_ARM` (3) -> Target machine is ARM.
/// - The function returns `None` if target machine information is unavailable.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - Everything uses the version format: `<major>.<minor>.<revision>.<build>`
/// - The build part is incremental and unique for all Everything versions.
///
/// # Requirements
/// Requires Everything 1.4.0 or later. (Maybe 1.4.1 or later indicated in source code)
pub fn Everything_GetTargetMachine() -> Option<TargetMachine> {
    // The function returns 0 if target machine information is unavailable.
    let target = unsafe { sdk_sys::Everything_GetTargetMachine() };
    match target {
        0 => None,
        sdk_sys::EVERYTHING_TARGET_MACHINE_X86 => {
            Some(TargetMachine::EVERYTHING_TARGET_MACHINE_X86)
        }
        sdk_sys::EVERYTHING_TARGET_MACHINE_X64 => {
            Some(TargetMachine::EVERYTHING_TARGET_MACHINE_X64)
        }
        sdk_sys::EVERYTHING_TARGET_MACHINE_ARM => {
            Some(TargetMachine::EVERYTHING_TARGET_MACHINE_ARM)
        }
        _ => unreachable!(),
    }
}

/// The `Everything_IsFastSort` function checks if the specified file information is indexed
/// and has fast sort enabled.
///
/// # Arguments
/// * `sort_type` - The sort type, should be one of the values named `EVERYTHING_SORT_*`.
///
/// # Return
/// - The function returns `true` if the specified information is indexed and has fast sort enabled.
/// - The function returns `false` if the specified information does not have fast sort enabled or
///   if an error occurred. To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - The following sort types are always fast sort types:
///    + `EVERYTHING_SORT_NAME_ASCENDING` (1) -> Name ascending
///    + `EVERYTHING_SORT_NAME_DESCENDING` (2) -> Name descending
///    + `EVERYTHING_SORT_RUN_COUNT_ASCENDING` (19) -> Run count ascending
///    + `EVERYTHING_SORT_RUN_COUNT_DESCENDING` (20) -> Run count descending
///    + `EVERYTHING_SORT_DATE_RECENTLY_CHANGED_ASCENDING` (21) -> Recently changed ascending
///    + `EVERYTHING_SORT_DATE_RECENTLY_CHANGED_DESCENDING` (22) -> Recently changed descending
///    + `EVERYTHING_SORT_DATE_RUN_ASCENDING` (25) -> Date run ascending
///    + `EVERYTHING_SORT_DATE_RUN_DESCENDING` (26) -> Date run descending
///
/// # Requirements
/// Requires Everything 1.4.1 or later. (Maybe 1.4.1.859 or later indicated in source code)
pub fn Everything_IsFastSort(sort_type: SortType) -> Option<bool> {
    let is_fast_sort = unsafe { sdk_sys::Everything_IsFastSort(sort_type as u32) };
    lower_bool_or_ipc_error(is_fast_sort)
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Primitive)]
#[allow(non_camel_case_types)]
pub enum FileInfoType {
    EVERYTHING_IPC_FILE_INFO_FILE_SIZE = 1,     // File size
    EVERYTHING_IPC_FILE_INFO_FOLDER_SIZE = 2,   // Folder size
    EVERYTHING_IPC_FILE_INFO_DATE_CREATED = 3,  // Date created
    EVERYTHING_IPC_FILE_INFO_DATE_MODIFIED = 4, // Date modified
    EVERYTHING_IPC_FILE_INFO_DATE_ACCESSED = 5, // Date accessed
    EVERYTHING_IPC_FILE_INFO_ATTRIBUTES = 6,    // Attributes
}

/// The `Everything_IsFileInfoIndexed` function checks if the specified file information
/// is indexed.
///
/// # Arguments
/// * `file_info_type` - The file info type, should be one of the values
///   named `EVERYTHING_IPC_FILE_INFO_*`.
///
/// # Return
/// - The function returns `true` if the specified information is indexed.
/// - The function returns `false` if the specified information is not indexed OR if
///   an error occurred. (weird)
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Requirements
/// Requires Everything 1.4.1 or later. (Maybe 1.4.1.859 or later indicated in source code)
pub fn Everything_IsFileInfoIndexed(file_info_type: FileInfoType) -> Option<bool> {
    let is_file_info_indexed =
        unsafe { sdk_sys::Everything_IsFileInfoIndexed(file_info_type as u32) };
    lower_bool_or_ipc_error(is_file_info_indexed)
}

/// The `Everything_GetRunCountFromFileName` function gets the run count from a specified
/// file in the Everything index by file name.
///
/// # Arguments
/// * `file_name` - An os string that specifies a fully qualified file name in
///   the Everything index.
///
/// # Return
/// - The function returns the number of times the file has been run from Everything.
///   (maybe zero?)
/// - The function returns 0 if an error occurred.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - If you want to do one less memory copy (from OsStr to "valid" UTF-16 u16 array), you
///   should use [`everything_sdk_sys::Everything_GetRunCountFromFileName`] directly.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_GetRunCountFromFileName(file_name: impl AsRef<OsStr>) -> Option<u32> {
    let name = U16CString::from_os_str(file_name).expect("the nul value only in the end");
    let run_count = unsafe { sdk_sys::Everything_GetRunCountFromFileNameW(PCWSTR(name.as_ptr())) };
    // FIX: if run count is zero, last error will not set OK(0) in C code, what should I do?
    zero_or_ipc_error(run_count)
}

/// The `Everything_SetRunCountFromFileName` function sets the run count for a specified
/// file in the Everything index by file name.
///
/// # Arguments
/// * `file_name` - An os string that specifies a fully qualified file name in
///   the Everything index.
/// * `run_count` - The new run count.
///
/// # Return
/// - The function returns `true` if successful.
/// - The function returns 0 if an error occurred.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - Set the run count to 0 to remove any run history information for the specified file.
/// - The file does not have to exist. When the file is created it will have the correct
///   run history.
/// - Run history information is preserved between file deletion and creation.
/// - Calling this function will update the date run to the current time for the specified
///   file.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_SetRunCountFromFileName(file_name: impl AsRef<OsStr>, run_count: u32) -> bool {
    let name = U16CString::from_os_str(file_name).expect("the nul value only in the end");
    // set a file to show higher in the results by setting an exaggerated run count
    let success =
        unsafe { sdk_sys::Everything_SetRunCountFromFileNameW(PCWSTR(name.as_ptr()), run_count) };
    lower_bool(success)
}

/// The `Everything_IncRunCountFromFileName` function increments the run count by one for
/// a specified file in the Everything by file name.
///
/// # Arguments
/// * `file_name` - An os string that specifies a fully qualified file name in the Everything index.
///
/// # Return
/// - The function returns the new run count for the specifed file.
/// - The function returns 0 if an error occurred.
///   To get extended error information, call `Everything_GetLastError`.
///
/// # Remarks
/// - The file does not have to exist. When the file is created it will have the correct
///   run history.
/// - Run history information is preserved between file deletion and creation.
/// - Calling this function will update the date run to the current time for the specified
///   file.
/// - Incrementing a file with a run count of u32::MAX (4294967295 / 0xffffffff) will
///   do nothing.
///
/// # Requirements
/// Requires Everything 1.4.1 or later.
pub fn Everything_IncRunCountFromFileName(file_name: impl AsRef<OsStr>) -> Option<NonZeroU32> {
    let name = U16CString::from_os_str(file_name).expect("the nul value only in the end");
    // increment the run count in Everything.
    let new_run_count =
        unsafe { sdk_sys::Everything_IncRunCountFromFileNameW(PCWSTR(name.as_ptr())) };
    if new_run_count == 0 {
        match Everything_GetLastError() {
            LastError::EVERYTHING_ERROR_IPC => None,
            _ => unreachable!(),
        }
    } else {
        Some(NonZeroU32::new(new_run_count).unwrap())
    }
}

/// The `Everything_SdkVerison` function gets this SDK version defined in constant value
/// [`everything_sdk_sys::EVERYTHING_SDK_VERSION`].
#[cfg_attr(not(feature = "raw"), allow(dead_code))]
pub const fn Everything_SdkVerison() -> u32 {
    sdk_sys::EVERYTHING_SDK_VERSION
}
