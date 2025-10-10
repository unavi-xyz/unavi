use wired_ecs::prelude::*;

pub fn add(app: &mut App) {
    app.add_system(Schedule::Startup, startup)
        .add_system(Schedule::Update, test_heap_types)
        .add_system(Schedule::Update, test_large_data);
}

#[derive(Decode, Encode, Clone, Debug, PartialEq)]
struct Point {
    x: usize,
}

#[derive(Decode, Encode, Component, Clone, Debug)]
struct MyHeap {
    s: String,
    v: Vec<Point>,
}

const S: &str = "Hello, world!";

fn get_v() -> Vec<Point> {
    vec![Point { x: 1 }, Point { x: 2 }, Point { x: 3 }]
}

#[derive(Decode, Encode, Component, Clone, Debug)]
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

fn test_heap_types(items: Query<&MyHeap>) {
    assert_eq!(items.len(), 1);

    let mut iter = items.iter();

    let a = iter.next().unwrap();
    assert_eq!(a.s, S);
    assert_eq!(a.v, get_v());
}

fn test_large_data(items: Query<&MyLarge>) {
    assert_eq!(items.len(), 1);

    let mut iter = items.iter();

    let a = iter.next().unwrap();
    assert_eq!(a.v.len(), N_LARGE);
    assert_eq!(a.v[0], 0);
    assert_eq!(a.v[N_LARGE - 1], N_LARGE - 1);
}
