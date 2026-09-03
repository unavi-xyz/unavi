use crate::runtime::shared::registry::{
    event::{
        EventBus,
        SpatialReceptor,
    },
    transform::TransformSnapshots,
};

#[must_use]
pub fn spatial_receptors(bus: &EventBus, transforms: &TransformSnapshots) -> Vec<SpatialReceptor> {
    bus.spatial_receptors(transforms)
}
