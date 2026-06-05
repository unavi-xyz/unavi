use serde::{
    Deserialize,
    Serialize,
};
use wired_prelude::wired_kv::TypedKv;

use crate::wired::kv::types::Kv;

wired_prelude::generate_script!(Script);

const KEY: &str = "counter";

#[derive(Default, Serialize, Deserialize)]
struct Counter {
    ticks: u64,
}

struct Script {
    kv: TypedKv<Kv>,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        let kv = TypedKv::default();
        if kv.get::<Counter>(KEY).expect("decode").is_none() {
            kv.set(KEY, &Counter::default()).expect("seed");
        }
        Ok(Self { kv })
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        let mut counter = self
            .kv
            .get::<Counter>(KEY)
            .expect("decode")
            .unwrap_or_default();
        counter.ticks = counter.ticks.saturating_add(1);
        if let Err(err) = self.kv.set(KEY, &counter) {
            println!("kv set failed: {err}");
        }
        Ok(())
    }
}
