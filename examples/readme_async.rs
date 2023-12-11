use everything_sdk::*;

#[tokio::main]
async fn main() {
    // At first, we should clearly understand that Everything-SDK IPC code is
    // based on **global mutable static variables** (the internal state is
    // stored in them) for now.

    // Even if you use async or multi-processes to query by IPC at the same time, they
    // will only be processed serially by the Everything.exe (ver 1.4.1) process.

    // So we need and can only do the query serially via global states.
    // Here we use the async version [`futures::Mutex`], so await it.
    let mut everything = global().lock().await;

    // All other things are consistent with the sync version. (expect searcher.query())

    match everything.is_db_loaded() {
        Ok(false) => panic!("The Everything database has not been fully loaded now."),
        Err(EverythingError::Ipc) => panic!("Everything is required to run in the background."),
        _ => {
            let mut searcher = everything.searcher();

            searcher.set_search("jpg");
            searcher
                .set_request_flags(
                    RequestFlags::EVERYTHING_REQUEST_FILE_NAME
                        | RequestFlags::EVERYTHING_REQUEST_PATH
                        | RequestFlags::EVERYTHING_REQUEST_SIZE
                        | RequestFlags::EVERYTHING_REQUEST_RUN_COUNT,
                )
                .set_max(5)
                .set_sort(SortType::EVERYTHING_SORT_DATE_RECENTLY_CHANGED_ASCENDING);

            assert_eq!(searcher.get_match_case(), false);

            // Send IPC query in Async, await for the result. So we are _unblocking_ now.
            // Some hevy query (like search single 'a') may take a lot of time in IPC data transfer.
            // So during this time, tokio goes to deal with other tasks.
            // When the IPC done, it will yield back for us.
            let results = searcher.query().await;

            let visible_num_results = dbg!(results.num());
            assert!(visible_num_results <= 5);
            let total_num_results = dbg!(results.total());
            assert!(total_num_results >= visible_num_results);

            let is_attr_flag_set =
                dbg!(results.request_flags()).contains(RequestFlags::EVERYTHING_REQUEST_ATTRIBUTES);
            assert!(!is_attr_flag_set);

            // We have all the data for the visible results in our memory, so we don't need
            // async streams to access it.
            for item in results.iter() {
                println!(
                    "Item[{}]: {} ({} bytes)",
                    item.index(),
                    item.path().join(item.filename()).display(),
                    item.size().unwrap(),
                );
            }

            let run_count = results
                .at(2)
                .expect("I'm pretty sure there are at least 3 results.")
                .run_count()
                .unwrap();
            println!("Run Count for Item[2]: `{}`", run_count);

            // Remember, because of global variables, there can only be one `everything`, `searcher`
            // and `results` at any time during the entire program lifetime.

            // Even being in Async mode, it doesn't change this thing.

            drop(results);
            searcher.set_search("cargo");
            let _results = searcher.query();
            // The `searcher` will be dropped here as out of scope.
        }
    }

    // So we can use `everything` again for now, to check the Everything.exe version.
    let (major, minor, patch, build, taget) = everything.version().unwrap();
    println!("Everything.exe version is {major}.{minor}.{patch}.{build} ({taget})");

    // Remember the LIFETIME again!
    assert!(global().try_lock().is_none());
    drop(everything);
    // We could take the lock now, await it, get it, use it, and return it immediately.
    let _is_in_appdata = global().lock().await.is_appdata().unwrap();
}
