use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Sender};
use embassy_time::{Duration, Instant, Ticker};
use esp_println::dbg;
use serde::Deserialize;
use shared::DisplayUpdate;

#[derive(Debug)]
struct DateTime {
    /// Time the device booted as unix timestamp in UTC time in seconds
    boot_time: u64,
    /// Offset from UTC in seconds
    offset: u64,
}

impl DateTime {
    pub fn new(curr_time: u64) -> Self {
        Self::with_offset(curr_time, 0)
    }

    fn with_offset(unix_time_utc: u64, offset: u64) -> Self {
        let boot_time = unix_time_utc - Instant::now().as_secs();
        Self { boot_time, offset }
    }

    /// Returns time as UTC unix timestamp as seconds
    fn now(&self) -> u64 {
        let from_boot = Instant::now().as_secs();
        self.boot_time + from_boot
    }

    fn now_local(&self) -> u64 {
        self.now() + self.offset
    }
}
// use chrono::serde::ts_seconds;
#[derive(Debug, Deserialize)]
struct WorlTimeApiResponse {
    // utc_offset: &'a str,
    // timezone: &'a str,
    // day_of_week:,
    // day_of_year:,
    // #[serde(with = "ts_seconds")]
    // datetime: chrono::DateTime<T>,
    // utc_datetime: chrono::DateTime<Utc>,
    unixtime: u64,
    raw_offset: u32,
    // week_number:,
    // dst: bool,
    // abbreviation:,
    dst_offset: u32,
    // dst_from:,
    // dst_until:,
    // client_ip:,
}

pub async fn setup_datetime(
    spawner: &Spawner,
    client: &mut crate::client::Client<'_>,
    display_sender: Sender<'static, CriticalSectionRawMutex, DisplayUpdate, 10>,
) {
    let response = client.fetch_local_time(chrono_tz::Europe::Helsinki).await;
    // println!("{:?}", response);
    // println!("{:?}", from_utf8(response).unwrap());
    let (t, _) = serde_json_core::from_slice::<WorlTimeApiResponse>(response).unwrap();

    let offset = t.raw_offset + t.dst_offset;
    dbg!(offset);
    dbg!(t.unixtime);
    let datetime = DateTime::with_offset(t.unixtime, offset as u64);
    dbg!(&datetime);
    spawner.must_spawn(update_time(display_sender, datetime));
}

#[embassy_executor::task]
pub async fn update_time(
    display_sender: Sender<'static, CriticalSectionRawMutex, DisplayUpdate, 10>,
    datetime: DateTime,
) {
    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        let curr_time = datetime.now_local();
        // dbg!(curr_time);
        display_sender.send(DisplayUpdate::SetTime(curr_time)).await;
        ticker.next().await;
    }
}

/// This will **panic** if the number is not between 1 and 12
/// Use [try_month_name_short] for unfallible version
pub fn month_name_short(month_number: u32) -> &'static str {
    match month_number {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => panic!("Month number must be between 1 and 12"),
    }
}

/// Returns month name if the input is between 1 and 12 or [None] otherwise
pub fn try_month_name_short(month_number: u32) -> Option<&'static str> {
    match month_number {
        1..13 => Some(month_name_short(month_number)),
        _ => None,
    }
}
