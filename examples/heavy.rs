//! Search for the single "a" results by Everything SDK IPC, request **ALL** results, no limit on quantity.

use std::time::Duration;

use everything_sdk::{RequestFlags, SortType};

fn main() {
    // Please make sure the Everything.exe is running in the background.
    let mut everything = everything_sdk::global().lock().unwrap();
    let mut searcher = everything.searcher();

    let results = searcher
        .set_search("a")
        // .set_request_flags(RequestFlags::default())
        // .set_sort(SortType::EVERYTHING_SORT_DATE_RUN_DESCENDING)
        .set_max(u32::MAX)
        .query();

    let (num, total) = (results.num(), results.total());
    let middle = results.at(total / 2).unwrap();
    println!(
        "[Heavy Search] Number {num} == Total {total}, middle = {}",
        middle.filepath().unwrap().display()
    );
}
