use everything_sdk::raw::*;

// It looks no different from the ergonomic wrapper, why can't we just write the code like this?
//
// Think about that:
//   For any `Everything_*` function as below, we insert another `Everything_*` function between
//   them, which will cause the modifications of the mutable global shared states (the underhood
//   we know they are just the global static variables in C code), because they all have access
//   to them. Finally it will cause _everything_ to become messy, uncontrollable and unreliable.
//
// All we can do is to line them up, in some certain order, and let them move forward one by one
// to prevent confusion.
//
// Ref: <https://stackoverflow.com/questions/27791532/how-do-i-create-a-global-mutable-singleton>

fn main() {
    match Everything_IsDBLoaded() {
        Some(false) => panic!("The Everything database has not been fully loaded now."),
        None => panic!("Everything is required to run in the background."),
        _ => {
            // Now _Everything_ is OK!

            Everything_SetSearch("jpg");
            Everything_SetRequestFlags(
                RequestFlags::EVERYTHING_REQUEST_FILE_NAME
                    | RequestFlags::EVERYTHING_REQUEST_PATH
                    | RequestFlags::EVERYTHING_REQUEST_SIZE
                    | RequestFlags::EVERYTHING_REQUEST_RUN_COUNT,
            );
            Everything_SetMax(5);
            Everything_SetSort(SortType::EVERYTHING_SORT_DATE_RECENTLY_CHANGED_ASCENDING);

            assert_eq!(Everything_GetMatchCase(), false);

            Everything_Query(true);

            let visible_num_results = dbg!(Everything_GetNumResults());
            assert!(visible_num_results <= 5);
            let total_num_results = dbg!(Everything_GetTotResults());
            assert!(total_num_results >= visible_num_results);

            let is_attr_flag_set = dbg!(Everything_GetResultListRequestFlags())
                .contains(RequestFlags::EVERYTHING_REQUEST_ATTRIBUTES);
            assert!(!is_attr_flag_set);

            for index in 0..5 {
                let path: std::path::PathBuf = Everything_GetResultPath(index).unwrap().into();
                let filename = Everything_GetResultFileName(index).unwrap();
                let file_size = Everything_GetResultSize(index).unwrap();
                println!(
                    "Item[{}]: {} ({} bytes)",
                    index,
                    path.join(filename).display(),
                    file_size,
                );
            }

            let run_count = Everything_GetResultRunCount(2);
            println!("Run Count for Item[2]: `{}`", run_count);

            Everything_SetSearch("cargo");
            Everything_Query(true);
        }
    }

    let (major, minor, patch, build, taget) = (
        Everything_GetMajorVersion().unwrap(),
        Everything_GetMinorVersion().unwrap(),
        Everything_GetRevision().unwrap(),
        Everything_GetBuildNumber().unwrap(),
        Everything_GetTargetMachine().unwrap(),
    );
    println!("Everything.exe version is {major}.{minor}.{patch}.{build} ({taget})");

    let _is_in_appdata = Everything_IsAppData().unwrap();
}
