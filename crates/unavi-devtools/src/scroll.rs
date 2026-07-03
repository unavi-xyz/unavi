use bevy::{
    input::mouse::{
        MouseScrollUnit,
        MouseWheel,
    },
    picking::hover::HoverMap,
    platform::collections::HashSet,
    prelude::*,
};

const LINE_HEIGHT: f32 = 24.0;

/// Marks a node as mouse-wheel scrollable. The node should also set
/// `Node::overflow` to scroll on the desired axis.
#[derive(Component, Default)]
#[require(ScrollPosition)]
pub struct Scrollable;

/// Applies wheel input to the nearest [`Scrollable`] ancestor of the hovered
/// node. Bevy clamps the resulting offset to the content size during layout.
pub(crate) fn apply_wheel_scroll(
    mut wheel: MessageReader<MouseWheel>,
    hover: Res<HoverMap>,
    parents: Query<&ChildOf>,
    mut scrollables: Query<&mut ScrollPosition, With<Scrollable>>,
) {
    for msg in wheel.read() {
        let dy = match msg.unit {
            MouseScrollUnit::Line => msg.y * LINE_HEIGHT,
            MouseScrollUnit::Pixel => msg.y,
        };
        let mut targets = HashSet::new();
        for hits in hover.values() {
            for entity in hits.keys() {
                let mut e = *entity;
                loop {
                    if scrollables.contains(e) {
                        targets.insert(e);
                        break;
                    }
                    let Ok(parent) = parents.get(e) else {
                        break;
                    };
                    e = parent.parent();
                }
            }
        }
        for target in targets {
            if let Ok(mut pos) = scrollables.get_mut(target) {
                pos.0.y -= dy;
            }
        }
    }
}
