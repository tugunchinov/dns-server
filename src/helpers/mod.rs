use std::time::{SystemTime, UNIX_EPOCH};

pub trait UnixTimeProvider: Sync + Send + Clone {
    fn unix_time_as_secs(&self) -> u64;
}

#[derive(Clone)]
pub struct SystemTimeProvider;

impl UnixTimeProvider for SystemTimeProvider {
    fn unix_time_as_secs(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
