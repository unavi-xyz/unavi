use std::sync::{
    LazyLock,
    atomic::{AtomicUsize, Ordering},
};

use wired_ecs::prelude::*;

pub fn add(app: &mut App) {
    app.add_system(Schedule::Update, test_local);
}

static COUNT: LazyLock<AtomicUsize> = LazyLock::new(AtomicUsize::default);

fn test_local(mut my_res: Local<usize>) {
    let count = COUNT.fetch_add(1, Ordering::AcqRel);
    assert_eq!(*my_res.as_ref(), count);

    *my_res.as_mut() += 1;
}
