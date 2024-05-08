use std::sync::Arc;

use didkit::JWK;
use dwn::{
    actor::{Actor, VerifiableCredential},
    store::{DataStore, MessageStore},
    DWN,
};
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

