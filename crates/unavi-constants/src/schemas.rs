use crate::WP_PREFIX;

const SCHEMA_PREFIX: &str = constcat::concat!(WP_PREFIX, "schemas/");

pub const CONNECT_URL_SCHEMA: &str = constcat::concat!(SCHEMA_PREFIX, "connect-url.json");
pub const REMOTE_RECORD_SCHEMA: &str = constcat::concat!(SCHEMA_PREFIX, "remote-record.json");
pub const SPACE_SCHEMA: &str = constcat::concat!(SCHEMA_PREFIX, "space.json");
