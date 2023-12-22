# Everything SDK in Rust

[<img alt="Everything Version" src="https://img.shields.io/badge/Everything-1.4.1-FF8000?style=for-the-badge" height="20">](https://www.voidtools.com/)
[<img alt="crates.io" src="https://img.shields.io/crates/v/everything-sdk.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/everything-sdk)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-everything_sdk-66c2a5?style=for-the-badge&logo=docs.rs" height="20">](https://docs.rs/everything-sdk)
[<img alt="MSRV" src="https://img.shields.io/badge/MSRV-1.70-ffc832?style=for-the-badge" height="20">](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)

Use [Everything SDK](https://www.voidtools.com/support/everything/sdk/) in __Rust way__. Types and Lifetime prevent you from accidentally calling the IPC query functions by mistake.

_No document proofing yet, but you could be able to try it out._

## Usage

```toml
[dependencies]
everything-sdk = "0.0.5"
```

_The Sample all you should know: [readme.rs](examples/readme.rs) ._

```rust
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
                        // | RequestFlags::EVERYTHING_REQUEST_ATTRIBUTES // no attr-data request
                        | RequestFlags::EVERYTHING_REQUEST_RUN_COUNT,
                )
                .set_max(5)
                .set_sort(SortType::EVERYTHING_SORT_DATE_RECENTLY_CHANGED_ASCENDING);

            // They have default value, check them in docs.
            assert_eq!(searcher.get_match_case(), false);

            // Send IPC query now, _block_ and wait for the result to return.
            // Some heavy query (like search single 'a') may take a lot of time in IPC data transfer, so
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
                let full_path = item.filepath().unwrap();
                println!(
                    "Item[{}]: {} ({} bytes)",
                    item.index(),
                    full_path.display(),
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
```

### The `async` feature

```toml
[dependencies]
everything-sdk = { version = "0.0.5", features = ["async"] }
```

There are __only two differences__ in async code compared to synchronous code.

```rust
let mut everything = global().lock().await; // get the global instance
let results = searcher.query().await; // the key point, unblocking query
```

_The complete Sample in __async__ mode with the same logic: [readme_async.rs](examples/readme_async.rs) ._

### The `raw` feature

```toml
[dependencies]
everything-sdk = { version = "0.0.5", features = ["raw"] }
```

```rust
use everything_sdk::raw::*;

fn main() {
    let (major, minor, patch, build, taget) = (
        Everything_GetMajorVersion().unwrap(),
        Everything_GetMinorVersion().unwrap(),
        Everything_GetRevision().unwrap(),
        Everything_GetBuildNumber().unwrap(),
        Everything_GetTargetMachine().unwrap(),
    );
    println!("Everything.exe version is {major}.{minor}.{patch}.{build} ({taget})");
}
```

_The complete Sample in __raw__ mode with the same logic: [readme_raw.rs](examples/readme_raw.rs) ._

### So why do we need these tedious steps?

> It looks no different from the ergonomic wrapper, why can't we just write the code like this?
>
> Think about that:
>
> For any `Everything_*` function as below, we insert another `Everything_*` function between
> them, which will cause the modifications of _the mutable global shared states_ (the underhood
> we know they are just the global static variables in C code), because they all have access
> to them. Finally it will cause _everything_ to become messy, uncontrollable and unreliable.
>
> All we can do is to _**line them up**_, in some certain order, and let them _**move forward**_ one by one
> to prevent chaos.

## License

This project use the [GPLv3 License](https://www.gnu.org/licenses/gpl-3.0.html).
