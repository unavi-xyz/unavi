use std::sync::{
    LazyLock,
    atomic::{AtomicUsize, Ordering},
};

use wired_ecs::prelude::*;

pub fn add(app: &mut App) {
    app.add_system(Schedule::Startup, startup)
        .add_system(Schedule::Update, test_mut_query)
        .add_system(Schedule::Update, test_mutation)
        .add_system(Schedule::Update, test_query_0)
        .add_system(Schedule::Update, test_query_1)
        .add_system(Schedule::Update, test_query_2)
        .order_systems(test_mutation, SystemOrder::After, test_mut_query);
}

#[derive(Decode, Encode, Component, Clone)]
struct MyPoint {
    x: usize,
    y: usize,
}

const X1: usize = 1;
const Y1: usize = 2;

const X2: usize = 3;
const Y2: usize = 4;

fn startup(commands: Commands) {
    let _ = commands.spawn();

    let a = commands.spawn();
    a.insert(MyPoint { x: X1, y: Y1 });

    let b = commands.spawn();
    b.insert(MyPoint { x: X2, y: Y2 });

    let c = commands.spawn();
    c.insert(A);
    c.insert(B);
    c.insert(C);
    c.insert(D);
}

static COUNT: LazyLock<AtomicUsize> = LazyLock::new(AtomicUsize::default);

fn test_mut_query(mut points: Query<&mut MyPoint>) {
    assert_eq!(points.len(), 2);

    let count = COUNT.fetch_add(1, Ordering::AcqRel);
    let x1 = X1 + count;
    let y1 = Y1 + count;
    let x2 = X2 + count;
    let y2 = Y2 + count;

    {
        let mut iter = points.iter();

        let a = iter.next().unwrap();
        assert_eq!(a.x, x1);
        assert_eq!(a.y, y1);

        let b = iter.next().unwrap();
        assert_eq!(b.x, x2);
        assert_eq!(b.y, y2);
    }

    {
        let mut iter_mut = points.iter_mut();

        let a = iter_mut.next().unwrap();
        assert_eq!(a.x, x1);
        assert_eq!(a.y, y1);
        a.x += 1;
        a.y += 1;

        let b = iter_mut.next().unwrap();
        assert_eq!(b.x, x2);
        assert_eq!(b.y, y2);
        b.x += 1;
        b.y += 1;
    }
}

/// Test that the mutated [MyPoint] data is seen in future systems.
fn test_mutation(points: Query<&MyPoint>) {
    assert_eq!(points.len(), 2);

    let count = COUNT.load(Ordering::Acquire);
    let x1 = X1 + count;
    let y1 = Y1 + count;
    let x2 = X2 + count;
    let y2 = Y2 + count;

    let mut iter = points.iter();

    let a = iter.next().unwrap();
    assert_eq!(a.x, x1);
    assert_eq!(a.y, y1);

    let b = iter.next().unwrap();
    assert_eq!(b.x, x2);
    assert_eq!(b.y, y2);
}

#[derive(Decode, Encode, Component)]
struct A;

#[derive(Decode, Encode, Component)]
struct B;

#[derive(Decode, Encode, Component)]
struct C;

#[derive(Decode, Encode, Component)]
struct D;

fn test_query_0(points: Query<&A>) {
    assert_eq!(points.len(), 1);
}
fn test_query_1(points: Query<(&A,)>) {
    assert_eq!(points.len(), 1);
}
fn test_query_2(points: Query<(&A, &B)>) {
    assert_eq!(points.len(), 1);
}
// fn test_query_3(points: Query<(&A, &B, &C)>) {
//     assert_eq!(points.len(), 1);
// }
// fn test_query_4(points: Query<(&A, &B, &C, &D)>) {
//     assert_eq!(points.len(), 1);
// }
