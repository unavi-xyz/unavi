use wired_ecs::prelude::*;

pub fn add(app: &mut App) {
    app.add_system(Schedule::Startup, startup)
        .add_system(Schedule::Startup, test_multiple_registers)
        .add_system(Schedule::Update, test_heap_types)
        .add_system(Schedule::Update, test_large_data);
}

#[component]
#[derive(Clone, Debug, PartialEq)]
struct Point {
    x: usize,
}

#[component]
struct MyHeap {
    s: String,
    v: Vec<Point>,
}

const S: &str = "Hello, world!";

fn get_v() -> Vec<Point> {
    vec![Point { x: 1 }, Point { x: 2 }, Point { x: 3 }]
}

#[component]
struct MyLarge {
    v: Vec<usize>,
}

const N_LARGE: usize = 10_000;

fn startup(commands: Commands) {
    let _ = commands.spawn();

    let a = commands.spawn();
    a.insert(MyHeap {
        s: S.to_string(),
        v: get_v(),
    });

    let b = commands.spawn();
    b.insert(MyLarge {
        v: (0..N_LARGE).collect(),
    });
}

fn test_multiple_registers() {
    let a_1 = MyHeap::register();
    let a_2 = MyHeap::register();
    let b_1 = MyLarge::register();
    let b_2 = MyLarge::register();
    assert_ne!(a_1, b_1);
    assert_eq!(a_1, a_2);
    assert_eq!(b_1, b_2);
}

fn test_heap_types(items: Query<&MyHeap>) {
    assert_eq!(items.len(), 1);

    let mut iter = items.iter();

    let a = iter.next().expect("test value expected");
    assert_eq!(a.s, S);
    assert_eq!(a.v, get_v());
}

fn test_large_data(items: Query<&MyLarge>) {
    assert_eq!(items.len(), 1);

    let mut iter = items.iter();

    let a = iter.next().expect("test value expected");
    assert_eq!(a.v.len(), N_LARGE);
    assert_eq!(a.v[0], 0);
    assert_eq!(a.v[N_LARGE - 1], N_LARGE - 1);
}
