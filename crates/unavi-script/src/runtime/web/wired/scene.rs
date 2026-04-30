use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredScene {}

#[wasm_bindgen]
impl WiredScene {
    pub fn create_document(&self) {}
    pub fn get_document(&self) {}
    pub fn load_hsd(&self) {}
    pub fn remove_document(&self) {}
    pub fn self_document(&self) {}
    pub fn self_node(&self) {}
}

#[wasm_bindgen]
#[derive(Default, Clone, Copy)]
pub struct WiredSceneTypes {}

#[wasm_bindgen]
impl WiredSceneTypes {
    // document
    pub fn document_add_asset(&self) {}
    pub fn document_assets(&self) {}
    pub fn document_clone(&self) {}
    pub fn document_create_material(&self) {}
    pub fn document_create_mesh(&self) {}
    pub fn document_create_node(&self) {}
    pub fn document_drop(&self) {}
    pub fn document_global_transform(&self) {}
    pub fn document_id(&self) {}
    pub fn document_materials(&self) {}
    pub fn document_meshes(&self) {}
    pub fn document_new(&self) {}
    pub fn document_nodes(&self) {}
    pub fn document_public(&self) {}
    pub fn document_remove_asset(&self) {}
    pub fn document_remove_material(&self) {}
    pub fn document_remove_mesh(&self) {}
    pub fn document_remove_node(&self) {}
    pub fn document_rep(&self) {}
    pub fn document_roots(&self) {}
    pub fn document_rotation(&self) {}
    pub fn document_scale(&self) {}
    pub fn document_set_public(&self) {}
    pub fn document_set_rotation(&self) {}
    pub fn document_set_scale(&self) {}
    pub fn document_set_sync(&self) {}
    pub fn document_set_transform(&self) {}
    pub fn document_set_translation(&self) {}
    pub fn document_sync(&self) {}
    pub fn document_transform(&self) {}
    pub fn document_translation(&self) {}
    // material
    pub fn material_alpha_cutoff(&self) {}
    pub fn material_alpha_mode(&self) {}
    pub fn material_base_color(&self) {}
    pub fn material_clone(&self) {}
    pub fn material_double_sided(&self) {}
    pub fn material_drop(&self) {}
    pub fn material_id(&self) {}
    pub fn material_metallic(&self) {}
    pub fn material_name(&self) {}
    pub fn material_new(&self) {}
    pub fn material_rep(&self) {}
    pub fn material_roughness(&self) {}
    pub fn material_set_alpha_cutoff(&self) {}
    pub fn material_set_alpha_mode(&self) {}
    pub fn material_set_base_color(&self) {}
    pub fn material_set_double_sided(&self) {}
    pub fn material_set_metallic(&self) {}
    pub fn material_set_name(&self) {}
    pub fn material_set_roughness(&self) {}
    pub fn material_set_sync(&self) {}
    pub fn material_set_unlit(&self) {}
    pub fn material_sync(&self) {}
    pub fn material_unlit(&self) {}
    // mesh
    pub fn mesh_clone(&self) {}
    pub fn mesh_colors(&self) {}
    pub fn mesh_drop(&self) {}
    pub fn mesh_id(&self) {}
    pub fn mesh_indices(&self) {}
    pub fn mesh_name(&self) {}
    pub fn mesh_new(&self) {}
    pub fn mesh_normals(&self) {}
    pub fn mesh_positions(&self) {}
    pub fn mesh_rep(&self) {}
    pub fn mesh_set_colors(&self) {}
    pub fn mesh_set_indices(&self) {}
    pub fn mesh_set_name(&self) {}
    pub fn mesh_set_normals(&self) {}
    pub fn mesh_set_positions(&self) {}
    pub fn mesh_set_sync(&self) {}
    pub fn mesh_set_tangents(&self) {}
    pub fn mesh_set_topology(&self) {}
    pub fn mesh_set_uv0(&self) {}
    pub fn mesh_set_uv1(&self) {}
    pub fn mesh_sync(&self) {}
    pub fn mesh_tangents(&self) {}
    pub fn mesh_topology(&self) {}
    pub fn mesh_uv0(&self) {}
    pub fn mesh_uv1(&self) {}
    // node
    pub fn node_add_child(&self) {}
    pub fn node_children(&self) {}
    pub fn node_clone(&self) {}
    pub fn node_collider(&self) {}
    pub fn node_drop(&self) {}
    pub fn node_global_transform(&self) {}
    pub fn node_id(&self) {}
    pub fn node_material(&self) {}
    pub fn node_mesh(&self) {}
    pub fn node_name(&self) {}
    pub fn node_new(&self) {}
    pub fn node_parent(&self) {}
    pub fn node_remove_child(&self) {}
    pub fn node_rep(&self) {}
    pub fn node_rigid_body(&self) {}
    pub fn node_rotation(&self) {}
    pub fn node_scale(&self) {}
    pub fn node_set_collider(&self) {}
    pub fn node_set_material(&self) {}
    pub fn node_set_mesh(&self) {}
    pub fn node_set_name(&self) {}
    pub fn node_set_rigid_body(&self) {}
    pub fn node_set_rotation(&self) {}
    pub fn node_set_scale(&self) {}
    pub fn node_set_sync(&self) {}
    pub fn node_set_transform(&self) {}
    pub fn node_set_translation(&self) {}
    pub fn node_sync(&self) {}
    pub fn node_transform(&self) {}
    pub fn node_translation(&self) {}
}
