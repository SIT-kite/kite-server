use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Timelike};
use prost_types::Timestamp;

pub mod badge;
pub mod balance;

// TODO: Add mod here.
pub trait ToDateTime {
    fn timestamp(self) -> DateTime<Local>;
}

pub trait ToTimestamp {
    fn datetime(self) -> Timestamp;
}

impl ToDateTime for Timestamp {
    fn timestamp(self) -> DateTime<Local> {
        let (secs, nsecs) = (self.seconds, self.nanos);
        let dt = NaiveDateTime::from_timestamp_opt(secs, nsecs as u32).unwrap();

        Local::from_local_datetime(&Local, &dt).unwrap()
    }
}

impl ToTimestamp for DateTime<Local> {
    fn datetime(self) -> Timestamp {
        let (secs, nsecs) = (self.timestamp(), self.nanosecond());
        Timestamp {
            seconds: secs,
            nanos: nsecs as i32,
        }
    }
}
