use wired_ecs::prelude::*;

pub fn add(app: &mut App) {
    app.add_system(Schedule::Startup, startup)
        .add_system(Schedule::Update, test_with_without);
}

#[derive(Decode, Encode, Component, Clone)]
struct CompX {
    value: f64,
}

#[derive(Decode, Encode, Component, Clone)]
struct CompY;

const X1: f64 = 1.0;
const X2: f64 = 2.0;

fn startup(commands: Commands) {
    let _ = commands.spawn();

    let a = commands.spawn();
    a.insert(CompX { value: X1 });
    a.insert(CompY);

    let b = commands.spawn();
    b.insert(CompX { value: X2 });
}

fn test_with_without(
    items_with: Query<&CompX, With<CompY>>,
    items_without: Query<&CompX, Without<CompY>>,
) {
    assert_eq!(items_with.len(), 1);
    assert_eq!(items_without.len(), 1);

    {
        let mut iter = items_with.iter();
        let a = iter.next().unwrap();
        assert_eq!(a.value, X1);
    }

    {
        let mut iter = items_without.iter();
        let b = iter.next().unwrap();
        assert_eq!(b.value, X2);
    }
}
