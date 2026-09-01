use crate::runtime::shared::registry::{
    event::{
        EventBus,
        SpatialReceptor,
    },
    transform::TransformSnapshots,
};

pub fn spatial_receptors(bus: &EventBus, transforms: &TransformSnapshots) -> Vec<SpatialReceptor> {
    bus.spatial_receptors(transforms)
}
