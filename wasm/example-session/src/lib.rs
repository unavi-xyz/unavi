//! Session state: a counter every present peer sees and any of them may
//! advance.
//!
//! One key on the script's own prim. The value is replicated and attributed to
//! whoever wrote it, and it is gone when everyone leaves — nothing here
//! reaches the document, which is what `commit` is for.

use serde::{
    Deserialize,
    Serialize,
};

wired_prelude::generate_script!(Script);

use crate::wired::scene::types::Prim;

const KEY: &str = "state:counter";

#[derive(Default, Serialize, Deserialize)]
struct Counter {
    ticks: u64,
}

struct Script {
    prim: Prim,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        Ok(Self {
            prim: wired::scene::api::self_prim()?,
        })
    }

    fn fixed_update(&mut self) -> anyhow::Result<()> {
        let mut counter = read(&self.prim).unwrap_or_default();
        counter.ticks = counter.ticks.saturating_add(1);

        // One flush per tick, whatever changed: a batch is how state is meant
        // to be written, and it lands atomically.
        let bytes = postcard::to_allocvec(&counter)?;
        if let Err(err) = self.prim.set_session(&[(KEY.to_string(), Some(bytes))]) {
            println!("session write failed: {err:?}");
        }
        Ok(())
    }
}

/// What the session says now, which may be what another peer wrote.
fn read(prim: &Prim) -> Option<Counter> {
    prim.session()
        .into_iter()
        .find(|(key, _)| key == KEY)
        .and_then(|(_, bytes)| postcard::from_bytes(&bytes).ok())
}
