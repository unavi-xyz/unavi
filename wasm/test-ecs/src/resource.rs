use std::sync::{
    LazyLock,
    atomic::{AtomicUsize, Ordering},
};

use wired_ecs::prelude::*;

const X: usize = 1;

pub fn add(app: &mut App) {
    app.insert_resource(MyX { x: X })
        .add_system(Schedule::Startup, test_res)
        .add_system(Schedule::Update, test_res_mut);
}

#[component]
struct MyX {
    x: usize,
}

fn test_res(my_res: Res<MyX>) {
    assert_eq!(my_res.as_ref().x, X);
}

static COUNT: LazyLock<AtomicUsize> = LazyLock::new(AtomicUsize::default);

fn test_res_mut(mut my_res: ResMut<MyX>) {
    let count = COUNT.fetch_add(1, Ordering::AcqRel);
    let x = X + count;

    assert_eq!(my_res.as_ref().x, x);
    assert_eq!(my_res.as_mut().x, x);

    my_res.as_mut().x += 1;
}
