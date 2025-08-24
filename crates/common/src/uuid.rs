use chrono::{DateTime, Utc};
use uuid::{ContextV7, NoContext, Timestamp, Uuid};

pub fn uuid_generate() -> Uuid {
    Uuid::now_v7()
}

pub fn uuid_generate_ts(now: DateTime<Utc>) -> Uuid {
    Uuid::new_v7(Timestamp::from_unix(
        NoContext,
        now.timestamp() as u64,
        now.timestamp_subsec_nanos(),
    ))
}

pub fn uuid_generate_ctx(ctx: ContextV7, now: DateTime<Utc>) -> Uuid {
    Uuid::new_v7(Timestamp::from_unix(
        ctx,
        now.timestamp() as u64,
        now.timestamp_subsec_nanos(),
    ))
}
