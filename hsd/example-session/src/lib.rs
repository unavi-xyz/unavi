//! Session state: a counter every present peer sees and any of them may
//! advance.
//!
//! `Counter` declares the destination of each field rather than the
//! mechanism: `ticks` is `#[state(session)]`, so the generated `sync` flushes
//! it as a `shared` edit on the script's own prim, one atomic batch per tick.
//! The value is replicated and attributed to whoever wrote it, and it is gone
//! when everyone leaves — nothing here reaches the document, which is what
//! `commit` is for.

wired_guest::generate_script!(Script);

/// What the session says the counter is, adopted back by `sync` before the
/// next tick advances it.
#[wired_guest::state]
#[derive(Default)]
struct Counter {
    #[state(session)]
    ticks: u64,
}

struct Script {
    prim:    (u64, u64),
    counter: Counter,
}

impl ScriptBehavior for Script {
    fn init() -> anyhow::Result<Self> {
        Ok(Self {
            prim:    wired::scene::document::script_prim(),
            counter: Counter::default(),
        })
    }

    fn fixed_update(
        &mut self,
        _tick: exports::wired::script::lifecycle::Tick,
    ) -> anyhow::Result<()> {
        self.counter.sync(self.prim)?;
        self.counter
            .set_ticks(self.counter.ticks().saturating_add(1));
        Ok(())
    }
}
