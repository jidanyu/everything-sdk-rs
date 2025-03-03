

use everything_sdk::*;
use chrono::{DateTime, NaiveDateTime, Utc};
use windows::Win32::Foundation::{FILETIME, SYSTEMTIME};
use windows::Win32::System::Time::{SystemTimeToFileTime,FileTimeToSystemTime};

fn main() {
    
    // Please make sure the Everything.exe is running in the background.
    for x in everything_sdk::global()
        .lock()
        .unwrap()
        .searcher()
        // .set_search("rc:last10secs c:\\Users\\jidan\\Desktop\\密数万象知识库")
        .set_search("jidanyu")
        .set_request_flags(
            RequestFlags::EVERYTHING_REQUEST_FILE_NAME
                | RequestFlags::EVERYTHING_REQUEST_PATH
                // | RequestFlags::EVERYTHING_REQUEST_DATE_RECENTLY_CHANGED
                | RequestFlags::EVERYTHING_REQUEST_DATE_MODIFIED,
        )
        .query()
    {
        // let time = x.date_recently_changed().unwrap();
        println!("name:{:?}", x.filepath().unwrap());
    }
    
}


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

// 从 FILETIME 中减去指定的纳秒数
fn subtract_nanoseconds(file_time: FILETIME, nanoseconds: u64) -> FILETIME {
    let mut adjusted_time = ((file_time.dwHighDateTime as u64) << 32) | (file_time.dwLowDateTime as u64);
    adjusted_time -= nanoseconds / 100; // 转换为 100 纳秒单位
    FILETIME {
        dwLowDateTime: adjusted_time as u32,
        dwHighDateTime: (adjusted_time >> 32) as u32,
    }
}


   // 获取当前系统时间
    // let mut system_time = SYSTEMTIME::default();
    // unsafe {
    //     system_time = windows::Win32::System::SystemInformation::GetSystemTime();
    // }

    // // 将 SYSTEMTIME 转换为 FILETIME
    // let mut file_time = FILETIME::default();
    // unsafe {
    //     SystemTimeToFileTime(&system_time, &mut file_time).expect("Failed to convert SYSTEMTIME to FILETIME");
    // }

    // // 减去 30 秒（30,000,000,000 纳秒，FILETIME 以 100 纳秒为单位）
    // let mut adjusted_file_time = subtract_nanoseconds(file_time, 30_000_000_000);

    // // 转换回 SYSTEMTIME
    // let mut adjusted_system_time = SYSTEMTIME::default();
    // unsafe {
    //     FileTimeToSystemTime(&adjusted_file_time, &mut adjusted_system_time)
    //         .expect("Failed to convert FILETIME to SYSTEMTIME");
    // }

    // // 打印结果
    // println!(
    //     "Adjusted SYSTEMTIME: {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
    //     adjusted_system_time.wYear, adjusted_system_time.wMonth, adjusted_system_time.wDay,
    //     adjusted_system_time.wHour, adjusted_system_time.wMinute, adjusted_system_time.wSecond
    // );

    // let adjusted_file_time_stamp = convert_filetime_to_u64(adjusted_file_time);
