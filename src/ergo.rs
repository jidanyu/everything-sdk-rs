use std::ffi::OsStr;
use std::ffi::OsString;
use std::marker::PhantomData;
use std::path::Path;
use std::path::PathBuf;
use std::sync::OnceLock;

use crate::raw;

pub use raw::FileInfoType;
pub use raw::RequestFlags;
pub use raw::SortType;

pub mod error {
    use super::RequestFlags;
    use thiserror::Error as ThisError;

    pub type Result<T> = std::result::Result<T, EverythingError>;

    #[non_exhaustive]
    #[derive(ThisError, Debug)]
    pub enum EverythingError {
        #[error("Failed to allocate memory for the search query.")]
        Memory,
        #[error("IPC is not available.")]
        Ipc,
        #[error("Failed to register the search query window class.")]
        RegisterClassEx,
        #[error("Failed to create the search query window.")]
        CreateWindow,
        #[error("Failed to create the search query thread.")]
        CreateThread,
        #[error("Invalid index. The index must be greater or equal to 0 and less than the number of visible results.")]
        InvalidIndex,
        #[error("Invalid call.")]
        InvalidCall,
        #[error("invalid request data, request data first.")]
        InvalidRequest(#[from] InvalidRequestError),
        #[error("bad parameter.")]
        InvalidParameter,
        #[error("not supported when using set_request_flags or set_sort to non-default value. (that is in query verison 2)")]
        UnsupportedInQueryVersion2,
    }

    #[non_exhaustive]
    #[derive(ThisError, Debug)]
    pub enum InvalidRequestError {
        #[error("should set the request flag {0:?}")]
        RequestFlagsNotSet(RequestFlags),
    }
}

pub use error::{EverythingError, InvalidRequestError, Result};

use tracing::debug;
use widestring::U16CStr;

mod helper {
    use super::*;

    pub fn is_default_request_flags(request_flags: RequestFlags) -> bool {
        request_flags == RequestFlags::default()
    }

    pub fn is_default_sort_type(sort_type: SortType) -> bool {
        sort_type == SortType::default()
    }

    // when send IPC query, try version 2 first (if we specified some non-version 1 request flags or sort)
    pub fn should_use_query_version_2(request_flags: RequestFlags, sort_type: SortType) -> bool {
        !is_default_request_flags(request_flags) || !is_default_sort_type(sort_type)
    }
}

#[cfg(not(feature = "async"))]
pub fn global() -> &'static std::sync::Mutex<EverythingGlobal> {
    static EVERYTHING_CELL: OnceLock<std::sync::Mutex<EverythingGlobal>> = OnceLock::new();
    EVERYTHING_CELL.get_or_init(|| std::sync::Mutex::new(EverythingGlobal {}))
}

#[cfg(feature = "async")]
pub fn global() -> &'static futures::lock::Mutex<EverythingGlobal> {
    static EVERYTHING_CELL: OnceLock<futures::lock::Mutex<EverythingGlobal>> = OnceLock::new();
    EVERYTHING_CELL.get_or_init(|| futures::lock::Mutex::new(EverythingGlobal {}))
}

#[non_exhaustive]
#[derive(Debug)]
pub struct EverythingGlobal {}

impl Drop for EverythingGlobal {
    /// NEVER call this, as the static variable would not be dropped.
    fn drop(&mut self) {
        // So this will not be called too.
        // We don't need this, `raw::Everything_Reset` in `EverythingSearcher` will
        // free the allocated memory.
        raw::Everything_CleanUp();
        unreachable!()
    }
}

impl EverythingGlobal {
    /// New the only one searcher.
    ///
    /// There is **at most one** searcher can exist globally at the same time.
    pub fn searcher<'a>(&'a mut self) -> EverythingSearcher<'a> {
        EverythingSearcher {
            _phantom: PhantomData::<&'a ()>,
        }
    }

    // --- General ---

    /// Everything uses the version format: `<major>.<minor>.<revision>.<build>`.
    /// The build part is incremental and unique for all Everything versions.
    pub fn version(&self) -> Result<(u32, u32, u32, u32, raw::TargetMachine)> {
        Ok((
            self.get_major_version()?,
            self.get_minor_version()?,
            self.get_revision()?,
            self.get_build_number()?,
            self.get_target_machine()?,
        ))
    }

    pub fn get_major_version(&self) -> Result<u32> {
        raw::Everything_GetMajorVersion().ok_or(EverythingError::Ipc)
    }

    pub fn get_minor_version(&self) -> Result<u32> {
        raw::Everything_GetMinorVersion().ok_or(EverythingError::Ipc)
    }

    pub fn get_revision(&self) -> Result<u32> {
        raw::Everything_GetRevision().ok_or(EverythingError::Ipc)
    }

    pub fn get_build_number(&self) -> Result<u32> {
        raw::Everything_GetBuildNumber().ok_or(EverythingError::Ipc)
    }

    pub fn get_target_machine(&self) -> Result<raw::TargetMachine> {
        raw::Everything_GetTargetMachine().ok_or(EverythingError::Ipc)
    }

    /// Request Everything to save settings and data to disk and exit.
    pub fn save_and_exit(&mut self) -> Result<bool> {
        raw::Everything_Exit().ok_or(EverythingError::Ipc)
    }

    /// Check if Everything's database is loaded.
    ///
    /// When Everything is loading, any queries will appear to return no results.
    /// Use this to determine if the database has been loaded before performing a query.
    pub fn is_db_loaded(&self) -> Result<bool> {
        raw::Everything_IsDBLoaded().ok_or(EverythingError::Ipc)
    }

    /// Check if Everything is running as administrator or as a standard user.
    pub fn is_admin(&self) -> Result<bool> {
        raw::Everything_IsAdmin().ok_or(EverythingError::Ipc)
    }

    /// Check if Everything is saving settings and data to `%APPDATA%\Everything` or to the same location
    /// as the `Everything.exe`.
    pub fn is_appdata(&self) -> Result<bool> {
        raw::Everything_IsAppData().ok_or(EverythingError::Ipc)
    }

    /// Request Everything to forcefully rebuild the Everything index.
    ///
    /// Requesting a rebuild will mark all indexes as dirty and start the rebuild process.
    /// Use `self.is_db_loaded()` to determine if the database has been rebuilt before
    /// performing a query.
    pub fn rebuild_db(&mut self) -> Result<bool> {
        // rebuild the database.
        raw::Everything_RebuildDB().ok_or(EverythingError::Ipc)
    }

    /// Request Everything to rescan all folder indexes.
    ///
    /// Everything will begin updating all folder indexes in the background.
    pub fn update_all_folder_indexes(&mut self) -> Result<bool> {
        // Request all folder indexes be rescanned.
        raw::Everything_UpdateAllFolderIndexes().ok_or(EverythingError::Ipc)
    }

    /// Request Everything to save the index to disk.
    ///
    /// The index is only saved to disk when you exit Everything.
    /// Call this to write the index to the file: `Everything.db`.
    pub fn save_db(&mut self) -> Result<bool> {
        // flush index to disk
        raw::Everything_SaveDB().ok_or(EverythingError::Ipc)
    }

    // --- Run History ---

    /// Request Everything to save the run history to disk.
    ///
    /// The run history is only saved to disk when you close an Everything search window or
    /// exit Everything.
    /// Call this to write the run history to the file: `Run History.csv`.
    pub fn save_run_history(&mut self) -> Result<bool> {
        // flush run history to disk
        raw::Everything_SaveRunHistory().ok_or(EverythingError::Ipc)
    }

    /// Delete all run history.
    ///
    /// Calling this function will clear all run history from memory and disk.
    pub fn delete_run_history(&mut self) -> Result<bool> {
        // clear run history
        raw::Everything_DeleteRunHistory().ok_or(EverythingError::Ipc)
    }

    /// Gets the run count from a specified file in the Everything index by file name.
    pub fn get_run_count(&self, filename: impl AsRef<Path>) -> Result<u32> {
        raw::Everything_GetRunCountFromFileName(filename.as_ref()).ok_or(EverythingError::Ipc)
    }

    /// Sets the run count for a specified file in the Everything index by file name.
    pub fn set_run_count(&mut self, filename: impl AsRef<Path>, run_count: u32) -> Result<()> {
        if raw::Everything_SetRunCountFromFileName(filename.as_ref(), run_count) {
            Ok(())
        } else {
            Err(EverythingError::Ipc)
        }
    }

    /// Increments the run count by one for a specified file in the Everything by file name.
    pub fn inc_run_count(&mut self, filename: impl AsRef<Path>) -> Result<u32> {
        raw::Everything_IncRunCountFromFileName(filename.as_ref())
            .map(|n| n.get())
            .ok_or(EverythingError::Ipc)
    }

    // --- Others ---

    /// Check if the specified file information is indexed and has fast sort enabled.
    pub fn is_fast_sort(&self, sort_type: SortType) -> Result<bool> {
        raw::Everything_IsFastSort(sort_type).ok_or(EverythingError::Ipc)
    }

    /// Check if the specified file information is indexed.
    pub fn is_file_info_indexed(&self, file_info_type: FileInfoType) -> Result<bool> {
        raw::Everything_IsFileInfoIndexed(file_info_type).ok_or(EverythingError::Ipc)
    }
}

#[non_exhaustive]
pub struct EverythingSearcher<'a> {
    _phantom: PhantomData<&'a ()>,
}

impl Drop for EverythingSearcher<'_> {
    fn drop(&mut self) {
        raw::Everything_Reset(); // CAUTION!
        debug!("[Drop] EverythingSearcher is dropped! (did Reset)");
    }
}

impl<'a> EverythingSearcher<'a> {
    // --- Manipulating the search state ---
    /// empty string "" by default.
    pub fn set_search(&mut self, text: impl AsRef<OsStr>) -> &'_ mut EverythingSearcher<'a> {
        raw::Everything_SetSearch(text);
        self
    }

    /// disable (false) by default.
    pub fn set_match_path(&mut self, enable: bool) -> &'_ mut EverythingSearcher<'a> {
        raw::Everything_SetMatchPath(enable);
        self
    }

    /// disable (false) by default.
    pub fn set_match_case(&mut self, enable: bool) -> &'_ mut EverythingSearcher<'a> {
        raw::Everything_SetMatchCase(enable);
        self
    }

    /// disable (false) by default.
    pub fn set_match_whole_word(&mut self, enable: bool) -> &'_ mut EverythingSearcher<'a> {
        raw::Everything_SetMatchWholeWord(enable);
        self
    }

    /// disable (false) by default.
    pub fn set_regex(&mut self, enable: bool) -> &'_ mut EverythingSearcher<'a> {
        raw::Everything_SetRegex(enable);
        self
    }

    /// `u32::MAX` (0xffffffff) by default, which means all results.
    pub fn set_max(&mut self, max_results: u32) -> &'_ mut EverythingSearcher<'a> {
        raw::Everything_SetMax(max_results);
        self
    }

    /// zero (0) by default.
    pub fn set_offset(&mut self, offset: u32) -> &'_ mut EverythingSearcher<'a> {
        raw::Everything_SetOffset(offset);
        self
    }

    /// The default sort is EVERYTHING_SORT_NAME_ASCENDING (1). This sort is free.
    pub fn set_sort(&mut self, sort_type: SortType) -> &'_ mut EverythingSearcher<'a> {
        raw::Everything_SetSort(sort_type);
        self
    }

    /// The default request flags are EVERYTHING_REQUEST_FILE_NAME | EVERYTHING_REQUEST_PATH (0x00000003).
    pub fn set_request_flags(&mut self, flags: RequestFlags) -> &'_ mut EverythingSearcher<'a> {
        raw::Everything_SetRequestFlags(flags);
        self
    }

    // --- Reading the search state ---
    pub fn get_search(&self) -> OsString {
        raw::Everything_GetSearch()
    }

    pub fn get_match_path(&self) -> bool {
        raw::Everything_GetMatchPath()
    }

    pub fn get_match_case(&self) -> bool {
        raw::Everything_GetMatchCase()
    }

    pub fn get_match_whole_word(&self) -> bool {
        raw::Everything_GetMatchWholeWord()
    }

    pub fn get_regex(&self) -> bool {
        raw::Everything_GetRegex()
    }

    pub fn get_max(&self) -> u32 {
        raw::Everything_GetMax()
    }

    pub fn get_offset(&self) -> u32 {
        raw::Everything_GetOffset()
    }

    pub fn get_sort(&self) -> SortType {
        raw::Everything_GetSort()
    }

    pub fn get_request_flags(&self) -> RequestFlags {
        raw::Everything_GetRequestFlags()
    }
}

impl<'a> EverythingSearcher<'a> {
    #[cfg(not(feature = "async"))]
    /// Execute an Everything IPC query with the current search state.
    ///
    /// It may take some time if you query a lot of items. Therefore, blocking needs to be
    /// considered in specific situations. (run it in new thread or use the `async` feature)
    pub fn query<'b>(&'b mut self) -> EverythingResults<'b> {
        raw::Everything_Query(true);
        EverythingResults {
            _phantom: PhantomData::<&'b ()>,
        }
    }

    #[cfg(feature = "async")]
    pub async fn query<'b>(&'b mut self) -> EverythingResults<'b> {
        non_blocking::QueryFuture::<'b>::new().await
    }

    /// Query and sort the results by path then file name in place.
    ///
    /// **NOT RECOMMENDED!** Use searcher.set_sort(_) instead.
    pub fn _query_and_sort_by_path<'b>(&'b mut self) -> EverythingResults<'b> {
        raw::Everything_Query(true);
        // SortResultsByPath is CPU Intensive. Sorting by path can take several seconds.
        // For improved performance, use [`raw::Everything_SetSort`]
        raw::Everything_SortResultsByPath();
        EverythingResults {
            _phantom: PhantomData::<&'b ()>,
        }
    }
}

#[cfg(feature = "async")]
mod non_blocking {
    use std::{
        marker::PhantomData,
        pin::Pin,
        sync::{Arc, Mutex},
        task::{Context, Poll, Waker},
        thread,
    };

    use windows::{
        core::w,
        Win32::{
            Foundation::{FALSE, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM},
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, GetClassInfoExW, PeekMessageW,
                PostMessageW, RegisterClassExW, WaitMessage, HWND_MESSAGE, MSG, PM_NOREMOVE,
                WINDOW_EX_STYLE, WM_COPYDATA, WM_USER, WNDCLASSEXW, WS_OVERLAPPED,
            },
        },
    };

    use tracing::debug;

    use super::EverythingResults;
    use crate::raw;

    #[non_exhaustive]
    pub struct QueryFuture<'a> {
        // query_expected: ExpectedParams,
        shared_state: Arc<Mutex<SharedState>>,
        _phantom: PhantomData<&'a ()>,
    }

    /// Shared state between the future and the waiting thread
    struct SharedState {
        /// Whether or not the sleep time has elapsed
        completed: bool,

        /// The waker for the task that `TimerFuture` is running on.
        /// The thread can use this after setting `completed = true` to tell
        /// `TimerFuture`'s task to wake up, see that `completed = true`, and
        /// move forward.
        waker: Option<Waker>,
    }

    impl<'a> std::future::Future for QueryFuture<'a> {
        type Output = EverythingResults<'a>;
        fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            debug!("poll() called");
            let mut shared_state = self.shared_state.lock().unwrap();
            if shared_state.completed {
                let results = EverythingResults {
                    _phantom: PhantomData::<&'a ()>,
                };
                debug!("Poll::Ready(_)!");
                Poll::Ready(results)
            } else {
                shared_state.waker = Some(cx.waker().clone());
                debug!("Poll::Pending");
                Poll::Pending
            }
        }
    }

    impl<'a> QueryFuture<'a> {
        pub fn new() -> Self {
            debug!("QueryFuture::new() start");

            let shared_state = Arc::new(Mutex::new(SharedState {
                completed: false,
                waker: None,
            }));

            // Spawn the new thread
            let thread_shared_state = shared_state.clone();
            thread::spawn(move || {
                debug!("thread::spawn");
                unsafe {
                    debug!("first time for init");
                    raw::Everything_SetReplyID(CUSTOM_REPLY_ID);
                    debug_assert_eq!(raw::Everything_GetReplyID(), CUSTOM_REPLY_ID);
                    let hwnd = create_window().unwrap();
                    raw::Everything_SetReplyWindow(hwnd);
                    debug_assert_eq!(raw::Everything_GetReplyWindow(), hwnd);

                    debug!("Execute Query with _FALSE_");
                    assert!(raw::Everything_Query(false));

                    let mut msg: MSG = MSG::default();
                    debug!("WaitMessage()...");
                    WaitMessage().unwrap(); // will blocking
                    debug!("WaitMessage() Done, One msg at least, then PeekMessageW()...");
                    if PeekMessageW(&mut msg, hwnd, 0, 0, PM_NOREMOVE) == FALSE {
                        panic!("There must be a message in the queue after WaitMessage().");
                    }
                    debug!("Gooooooot it! WM_{:#06x} ({})", msg.message, msg.message);
                    if msg.message != WM_USER_IS_QUERY_REPLY_DONE {
                        panic!("Must be only one type message set by us.");
                    }
                    debug!("Yes, we did it. (now we have results)");
                    DestroyWindow(hwnd).unwrap();
                    debug!("DestroyWindow() Done");

                    let mut shared_state = thread_shared_state.lock().unwrap();
                    // Signal that the Query has completed and wake up the last
                    // task on which the future was polled, if one exists.
                    shared_state.completed = true;
                    debug!("set .completed to true");
                    if let Some(waker) = shared_state.waker.take() {
                        debug!("waker.wake()");
                        waker.wake()
                    }
                }
            });

            debug!("QueryFuture::new() end");
            Self {
                shared_state,
                _phantom: PhantomData::<&'a ()>,
            }
        }
    }

    const WM_USER_IS_QUERY_REPLY_DONE: u32 = WM_USER + 42;
    const CUSTOM_REPLY_ID: u32 = 9527;

    extern "system" fn wndproc(
        hwnd: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            match message {
                WM_COPYDATA => {
                    if raw::Everything_IsQueryReply(message, wparam, lparam, CUSTOM_REPLY_ID) {
                        debug!("[wndproc] Everything_IsQueryReply() -> YEEEESSSSSS!! (So copy done and PostMessage(WM_USER_IS_QUERY_REPLY_DONE))");
                        PostMessageW(hwnd, WM_USER_IS_QUERY_REPLY_DONE, WPARAM(0), LPARAM(0))
                            .unwrap();
                        LRESULT(1)
                    } else {
                        // DefWindowProcW(hwnd, message, wparam, lparam)
                        panic!("!!!! Everything_IsQueryReply() -> NOOOO!!");
                    }
                }
                _ => {
                    debug!(
                        "[wndproc] DefWindowProcW( msg => WM_{:#06x} ({}) )",
                        message, message
                    );
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
            }
        }
    }

    fn create_window() -> windows::core::Result<HWND> {
        unsafe {
            let instance: HINSTANCE = GetModuleHandleW(None)?.into();
            assert!(!instance.is_invalid());

            let window_class_name = w!("EVERYTHING_SDK_RUST");

            let mut wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                hInstance: instance,
                lpszClassName: window_class_name,
                lpfnWndProc: Some(wndproc),
                ..Default::default()
            };

            if GetClassInfoExW(instance, window_class_name, &mut wc).is_err() {
                let atom = RegisterClassExW(&wc);
                assert!(atom != 0);
            }

            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                window_class_name,
                w!("The window for async query in everything-sdk-rs crate"),
                WS_OVERLAPPED,
                0,
                0,
                0,
                0,
                // Ref: https://devblogs.microsoft.com/oldnewthing/20171218-00/?p=97595
                HWND_MESSAGE,
                None,
                instance,
                None,
            );

            assert_ne!(hwnd, HWND(0));

            Ok(hwnd)
        }
    }
}

#[non_exhaustive]
pub struct EverythingResults<'a> {
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Drop for EverythingResults<'a> {
    fn drop(&mut self) {
        // I want to free memory for the results, but no api just for it.
        // and should not call [`raw::Everything_Reset`], for long live reuse EverythingSearcher.
        debug!("[Drop] EverythingResults is dropped!");
    }
}

impl<'a> EverythingResults<'a> {
    /// the results logic length, for available index in iterator.
    pub fn len(&self) -> u32 {
        self.num()
    }

    pub fn at(&self, index: u32) -> Option<EverythingItem<'a>> {
        self.iter().nth(index as usize)
    }

    pub fn iter(&self) -> Iter<'a> {
        Iter {
            next_index: 0,
            length: self.len(),
            request_flags: self.request_flags(),
            _phantom: PhantomData::<&'a ()>,
        }
    }

    pub fn request_flags(&self) -> RequestFlags {
        raw::Everything_GetResultListRequestFlags()
    }

    pub fn sort_type(&self) -> SortType {
        raw::Everything_GetResultListSort()
    }

    fn is_query_version_2(&self) -> bool {
        helper::should_use_query_version_2(self.request_flags(), self.sort_type())
    }

    pub fn num_files(&self) -> Result<u32> {
        if self.is_query_version_2() {
            Err(EverythingError::UnsupportedInQueryVersion2)
        } else {
            let num = raw::Everything_GetNumFileResults();
            Ok(num) // would not be error (EVERYTHING_ERROR_INVALIDCALL), zero is valid.
        }
    }

    pub fn num_folders(&self) -> Result<u32> {
        if self.is_query_version_2() {
            Err(EverythingError::UnsupportedInQueryVersion2)
        } else {
            let num = raw::Everything_GetNumFolderResults();
            Ok(num) // would not be error (EVERYTHING_ERROR_INVALIDCALL), zero is valid.
        }
    }

    /// the number of visible file and folder results.
    pub fn num(&self) -> u32 {
        let num = raw::Everything_GetNumResults();
        num // would not be error (EVERYTHING_ERROR_INVALIDCALL), zero is valid.
    }

    pub fn total_files(&self) -> Result<u32> {
        if self.is_query_version_2() {
            Err(EverythingError::UnsupportedInQueryVersion2)
        } else {
            let num = raw::Everything_GetTotFileResults();
            Ok(num) // would not be error (EVERYTHING_ERROR_INVALIDCALL), zero is valid.
        }
    }

    pub fn total_folders(&self) -> Result<u32> {
        if self.is_query_version_2() {
            Err(EverythingError::UnsupportedInQueryVersion2)
        } else {
            let num = raw::Everything_GetTotFolderResults();
            Ok(num) // would not be error (EVERYTHING_ERROR_INVALIDCALL), zero is valid.
        }
    }

    pub fn total(&self) -> u32 {
        let total = raw::Everything_GetTotResults();
        total // would not be error (EVERYTHING_ERROR_INVALIDCALL), zero is valid.
    }
}

#[non_exhaustive]
pub struct EverythingItem<'a> {
    index: u32,
    request_flags: RequestFlags,
    _phantom: PhantomData<&'a ()>,
}

#[non_exhaustive]
pub struct Iter<'a> {
    next_index: u32,
    length: u32,
    request_flags: RequestFlags,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = EverythingItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next_index < self.length {
            let index = self.next_index;
            self.next_index += 1;
            Some(EverythingItem {
                index,
                request_flags: self.request_flags,
                _phantom: PhantomData::<&'a ()>,
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rest = usize::try_from(self.length - self.next_index).unwrap();
        (rest, Some(rest))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let index = self.next_index + u32::try_from(n).unwrap();
        if index < self.length {
            self.next_index = index + 1;
            Some(EverythingItem {
                index,
                request_flags: self.request_flags,
                _phantom: PhantomData::<&'a ()>,
            })
        } else {
            self.next_index = self.length;
            None
        }
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {}

impl<'a> IntoIterator for EverythingResults<'a> {
    type Item = EverythingItem<'a>;
    type IntoIter = Iter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            next_index: 0,
            length: self.len(),
            request_flags: self.request_flags(),
            _phantom: PhantomData::<&'a ()>,
        }
    }
}

impl<'a> EverythingItem<'a> {
    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn is_volume(&self) -> bool {
        raw::Everything_IsVolumeResult(self.index)
    }

    pub fn is_folder(&self) -> bool {
        raw::Everything_IsFolderResult(self.index)
    }

    pub fn is_file(&self) -> bool {
        raw::Everything_IsFileResult(self.index)
    }

    pub fn filename(&self) -> Result<OsString> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_FILE_NAME)?;
        Ok(raw::Everything_GetResultFileName(self.index).unwrap())
    }

    pub fn path(&self) -> Result<PathBuf> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_PATH)?;
        Ok(raw::Everything_GetResultPath(self.index).unwrap().into())
    }

    /// A convenient function to get the full path by Everything_GetResultFullPathName.
    ///
    /// Different from the [`full_path_name`], this is an unofficial function provided for
    /// the special case. (We can use [`raw::Everything_GetResultFullPathName`] with the
    /// two default flags EVERYTHING_REQUEST_PATH and EVERYTHING_REQUEST_FILE_NAME)
    pub fn filepath(&self) -> Result<PathBuf> {
        // A bit weird but this is a special case in the official documentation.
        self.need_flags_set(
            RequestFlags::EVERYTHING_REQUEST_PATH | RequestFlags::EVERYTHING_REQUEST_FILE_NAME,
        )?;
        let buf_len = u32::from(raw::Everything_GetResultFullPathNameSizeHint(self.index).unwrap());
        let mut buf = vec![0; buf_len as usize];
        let n_wchar =
            u32::from(raw::Everything_GetResultFullPathName(self.index, &mut buf).unwrap());
        assert_eq!(buf_len, n_wchar + 1);
        Ok(U16CStr::from_slice(&buf).unwrap().to_os_string().into())
    }

    /// Get the full path name, can be with len limit if you need.
    ///
    /// Similar to x.path().join(x.filename()) if parent path is NOT drive root (like C:).
    /// (Ref: <https://github.com/nodejs/node/issues/14405>)
    ///
    /// Buf if the pathname is too long, you can choose to cut off the tail, reduce the
    /// memory consumption, or limit the max size of buffer memory allocation.
    pub fn full_path_name(&self, max_len: Option<u32>) -> Result<PathBuf> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_FULL_PATH_AND_FILE_NAME)?;
        let size_hint =
            u32::from(raw::Everything_GetResultFullPathNameSizeHint(self.index).unwrap());
        let buf_len = std::cmp::min(size_hint, max_len.unwrap_or(u32::MAX)) as usize;
        let mut buf = vec![0; buf_len];
        let n_wchar =
            u32::from(raw::Everything_GetResultFullPathName(self.index, &mut buf).unwrap());
        assert_eq!(size_hint, n_wchar + 1);
        Ok(U16CStr::from_slice(&buf).unwrap().to_os_string().into())
    }

    // Check if the corresponding flags are set. (usually just check a single flag)
    fn need_flags_set(&self, flags: RequestFlags) -> Result<()> {
        if self.request_flags.contains(flags) {
            Ok(())
        } else {
            Err(EverythingError::InvalidRequest(
                InvalidRequestError::RequestFlagsNotSet(flags),
            ))
        }
    }

    pub fn extension(&self) -> Result<OsString> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_EXTENSION)?;
        Ok(raw::Everything_GetResultExtension(self.index).unwrap())
    }

    pub fn size(&self) -> Result<u64> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_SIZE)?;
        let file_size = raw::Everything_GetResultSize(self.index).unwrap();
        // If request flag `RequestFlags::EVERYTHING_REQUEST_ATTRIBUTES` is not set, the GetResultSize function
        // will success, but the file_size for folder will be Some(-1). If the ATTRIBUTES flag is set. the
        // GetResultSize will success too, but the file_size for folder will be Some(0).
        //
        // There is no relevant explanation in the documentation about that. (so wired, maybe we do not know
        // whether this index points to a file or a directory unless we have ATTRIBUTES.)
        //
        // So for consistency, we will get Ok(0) for folder index regardless of whether the request flag
        // `RequestFlags::EVERYTHING_REQUEST_ATTRIBUTES` had been set.
        u64::try_from(file_size).or_else(|_e| {
            if raw::Everything_IsFolderResult(self.index) {
                debug_assert_eq!(file_size, -1); // file_size will most likely be -1
                Ok(0)
            } else {
                panic!(
                    "file size should not be a negative integer => {}",
                    file_size
                )
            }
        })
    }

    pub fn date_created(&self) -> Result<u64> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_DATE_CREATED)?;
        Ok(raw::Everything_GetResultDateCreated(self.index).unwrap())
    }

    pub fn date_modified(&self) -> Result<u64> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_DATE_MODIFIED)?;
        Ok(raw::Everything_GetResultDateModified(self.index).unwrap())
    }

    pub fn date_accessed(&self) -> Result<u64> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_DATE_ACCESSED)?;
        Ok(raw::Everything_GetResultDateAccessed(self.index).unwrap())
    }

    pub fn attributes(&self) -> Result<u32> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_ATTRIBUTES)?;
        Ok(raw::Everything_GetResultAttributes(self.index).unwrap())
    }

    pub fn file_list_filename(&self) -> Result<OsString> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_FILE_LIST_FILE_NAME)?;
        Ok(raw::Everything_GetResultFileListFileName(self.index).unwrap())
    }

    pub fn run_count(&self) -> Result<u32> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_RUN_COUNT)?;
        Ok(raw::Everything_GetResultRunCount(self.index))
    }

    pub fn date_run(&self) -> Result<u64> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_DATE_RUN)?;
        Ok(raw::Everything_GetResultDateRun(self.index).unwrap())
    }

    pub fn date_recently_changed(&self) -> Result<u64> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_DATE_RECENTLY_CHANGED)?;
        Ok(raw::Everything_GetResultDateRecentlyChanged(self.index).unwrap())
    }

    pub fn highlighted_filename(&self) -> Result<OsString> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_HIGHLIGHTED_FILE_NAME)?;
        Ok(raw::Everything_GetResultHighlightedFileName(self.index).unwrap())
    }

    pub fn highlighted_path(&self) -> Result<OsString> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_HIGHLIGHTED_PATH)?;
        Ok(raw::Everything_GetResultHighlightedPath(self.index).unwrap())
    }

    pub fn highlighted_full_path_and_filename(&self) -> Result<OsString> {
        self.need_flags_set(RequestFlags::EVERYTHING_REQUEST_HIGHLIGHTED_FULL_PATH_AND_FILE_NAME)?;
        Ok(raw::Everything_GetResultHighlightedFullPathAndFileName(self.index).unwrap())
    }
}
