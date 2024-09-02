use crate::client::Client;
use chrono::TimeZone;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Sender, mutex::Mutex};
use embassy_time::{Duration, Instant, Ticker};
use esp_println::dbg;
use serde::Deserialize;
use shared::DisplayUpdate;

#[derive(Debug)]
pub struct LocalClock {
    /// Time the device booted as unix timestamp in UTC time in seconds
    boot_time: u64,
    /// Offset from UTC in seconds
    timezone: chrono_tz::Tz,
}

impl LocalClock {
    pub fn new(curr_time: u64) -> Self {
        Self::with_offset(curr_time, chrono_tz::UTC)
    }

    fn with_offset(unix_time_utc: u64, offset: chrono_tz::Tz) -> Self {
        let boot_time = unix_time_utc - Instant::now().as_secs();
        Self {
            boot_time,
            timezone: offset,
        }
    }

    /// Returns time as UTC unix timestamp as seconds
    #[inline]
    fn now(&self) -> u64 {
        let from_boot = Instant::now().as_secs();
        self.boot_time + from_boot
    }

    /// Returns unix timestamp in local time
    #[inline]
    pub fn now_local(&self) -> u64 {
        self.timezone
            .timestamp_millis_opt(self.now() as i64 * 1000)
            .unwrap()
            .timestamp() as u64
    }

    pub fn get_time(&self) -> chrono::DateTime<chrono_tz::Tz> {
        // Todo get timezone from somewhere
        self.timezone
            .timestamp_millis_opt(self.now() as i64 * 1000)
            .unwrap()
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
    // raw_offset: u32,
    // week_number:,
    // dst: bool,
    // abbreviation:,
    // dst_offset: u32,
    // dst_from:,
    // dst_until:,
    // client_ip:,
}

pub async fn setup_datetime(
    spawner: &Spawner,
    client: &Mutex<CriticalSectionRawMutex, Client<'_>>,
    display_sender: Sender<'static, CriticalSectionRawMutex, DisplayUpdate, 10>,
) {
    let timezone = chrono_tz::Europe::Helsinki;
    let t;
    {
        let mut client_guard = client.lock().await;
        let response = client_guard.fetch_local_time(timezone).await;
        (t, _) = serde_json_core::from_slice::<WorlTimeApiResponse>(response).unwrap();
    }
    // let offset = t.raw_offset + t.dst_offset;

    let local_clock = LocalClock::with_offset(t.unixtime, timezone);

    dbg!(&local_clock);

    spawner.must_spawn(update_time(display_sender, local_clock));
}

#[embassy_executor::task]
pub async fn update_time(
    display_sender: Sender<'static, CriticalSectionRawMutex, DisplayUpdate, 10>,
    local_clock: LocalClock,
) {
    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        display_sender
            .send(DisplayUpdate::SetTime(local_clock.get_time()))
            .await;
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
