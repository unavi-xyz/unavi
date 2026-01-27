use bevy::prelude::*;

use crate::stage::Attrs;

pub fn parse_material_attrs(attrs: &Attrs, node: Entity, commands: &mut Commands) {
    if attrs.material_base_color.is_some()
        || attrs.material_base_color_texture.is_some()
        || attrs.material_double_sided.is_some()
        || attrs.material_metallic.is_some()
        || attrs.material_metallic_roughness_texture.is_some()
        || attrs.material_normal_texture.is_some()
        || attrs.material_occlusion_texture.is_some()
        || attrs.material_roughness.is_some()
    {
        // TODO
    } else {
        commands
            .entity(node)
            .remove::<MeshMaterial3d<StandardMaterial>>();
    }
}
