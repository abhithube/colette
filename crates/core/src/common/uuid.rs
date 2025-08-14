use chrono::{DateTime, Utc};
use uuid::{ClockSequence, ContextV7, NoContext, Timestamp, Uuid};

#[derive(Debug, Default)]
pub struct UuidGenerator {
    context: UuidContext,
    timestamp: DateTime<Utc>,
}

impl UuidGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_batched(mut self, value: bool) -> Self {
        if value {
            self.context = UuidContext::Context(ContextV7::new());
        }

        self
    }

    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;

        self
    }

    pub fn generate(&self) -> Uuid {
        Uuid::new_v7(Timestamp::from_unix(
            &self.context,
            self.timestamp.timestamp() as u64,
            self.timestamp.timestamp_subsec_nanos(),
        ))
    }
}

#[derive(Debug)]
enum UuidContext {
    Context(ContextV7),
    NoContext(NoContext),
}

impl Default for UuidContext {
    fn default() -> Self {
        Self::NoContext(NoContext)
    }
}

impl ClockSequence for UuidContext {
    type Output = u128;

    fn generate_sequence(&self, seconds: u64, nanos: u32) -> Self::Output {
        match self {
            Self::Context(context) => context.generate_sequence(seconds, nanos).into(),
            Self::NoContext(context) => context.generate_sequence(seconds, nanos).into(),
        }
    }
}
