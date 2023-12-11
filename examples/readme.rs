use everything_sdk::*;

fn main() {
    // At first, we should clearly understand that Everything-SDK IPC code is
    // based on **global mutable static variables** (the internal state is
    // stored in them), at least that's the case for now.

    // Even if you use multiple processes to query by IPC at the same time, they
    // will only be processed serially by the Everything.exe (ver 1.4.1) process.

    // So we need and can only do the query serially via global states.
    let mut everything = global().try_lock().unwrap();

    // Check whether the Everything.exe in the background is running.
    match everything.is_db_loaded() {
        Ok(false) => panic!("The Everything database has not been fully loaded now."),
        Err(EverythingError::Ipc) => panic!("Everything is required to run in the background."),
        _ => {
            // Now _Everything_ is OK!

            // We got the searcher, which can be reused for multiple times queries and cleans up
            // memory when it has been dropped.
            let mut searcher = everything.searcher();

            // Set the query parameters, chaining call is optional.
            searcher.set_search("jpg");
            searcher
                .set_request_flags(
                    RequestFlags::EVERYTHING_REQUEST_FILE_NAME
                | RequestFlags::EVERYTHING_REQUEST_PATH
                | RequestFlags::EVERYTHING_REQUEST_SIZE
                // | RequestFlags::EVERYTHING_REQUEST_ATTRIBUTES // no attributes-data request
                | RequestFlags::EVERYTHING_REQUEST_RUN_COUNT,
                )
                .set_max(5)
                .set_sort(SortType::EVERYTHING_SORT_DATE_RECENTLY_CHANGED_ASCENDING);

            // They have default value, check them in docs.
            assert_eq!(searcher.get_match_case(), false);

            // Send IPC query now, _block_ and wait for the result to return.
            // Some hevy query (like search single 'a') may take a lot of time in IPC data transfer, so
            // if you need unblocking, do them in a new thread or enable the `async` feature in crate.
            let results = searcher.query();

            // We set the max-limit(5) for query, so we can check these 5 or less results.
            let visible_num_results = dbg!(results.num());
            assert!(visible_num_results <= 5);
            // But we also know the total number of results if max not set. (just know, no IPC data copy)
            let total_num_results = dbg!(results.total());
            assert!(total_num_results >= visible_num_results);

            // Make sure you set the corresponding `RequestFlags` for getting result props.
            let is_attr_flag_set =
                dbg!(results.request_flags()).contains(RequestFlags::EVERYTHING_REQUEST_ATTRIBUTES);
            // So we have no corresponding data to call item.attributes() in for-loop as below.
            assert!(!is_attr_flag_set);

            // Walking the 5 query results from Everything IPC by iterator.
            for item in results.iter() {
                println!(
                    "Item[{}]: {} ({} bytes)",
                    item.index(),
                    item.path().join(item.filename()).display(),
                    // We have set the `RequestFlags::EVERYTHING_REQUEST_SIZE` for it before.
                    item.size().unwrap(),
                );
            }

            // Or you are only interested in the run count of the 3rd result in Everything Run History.
            let run_count = results
                .at(2)
                .expect("I'm pretty sure there are at least 3 results.")
                .run_count()
                .unwrap();
            println!("Run Count for Item[2]: `{}`", run_count);

            // Remember, because of global variables, there can only be one `everything`, `searcher`
            // and `results` at any time during the entire program lifetime.

            drop(results);
            // When the `results` lifetime over, we can do the next query by `searcher`.
            searcher.set_search("cargo");
            let _results = searcher.query();

            // So the opposite, we can not call this by `everything` for the lifetime limit.
            // let _ = everything.version().unwrap();

            // But the `searcher` will be dropped here as out of scope.
        }
    }

    // So we can use `everything` again for now, to check the Everything.exe version.
    let (major, minor, patch, build, taget) = everything.version().unwrap();
    println!("Everything.exe version is {major}.{minor}.{patch}.{build} ({taget})");

    // Remember the LIFETIME again!
    global().try_lock().expect_err("Prev lock is still held.");
    drop(everything);
    let _is_in_appdata = global()
        .try_lock()
        .expect("We could take the lock now, use it, and return it immediately.")
        .is_appdata()
        .unwrap();
}
