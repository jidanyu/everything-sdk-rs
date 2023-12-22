//! Search for the first 5 "foo" results by Everything SDK IPC.

fn main() {
    // Please make sure the Everything.exe is running in the background.
    for x in everything_sdk::global()
        .lock()
        .unwrap()
        .searcher()
        .set_search("foo")
        .set_max(5)
        .query()
    {
        println!("{}", x.filepath().unwrap().display());
    }
}
