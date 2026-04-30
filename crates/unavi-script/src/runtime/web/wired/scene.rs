use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Clone)]
pub struct WiredScene {}

#[wasm_bindgen]
impl WiredScene {
    pub fn create_document(&self) -> JsValue {
        todo!()
    }
    pub fn get_document(&self, id: Vec<u8>) -> JsValue {
        todo!()
    }
    pub fn load_hsd(&self, blob_id: Vec<u8>) -> JsValue {
        todo!()
    }
    pub fn remove_document(&self, id: Vec<u8>) {}
    pub fn self_document(&self) -> JsValue {
        todo!()
    }
    pub fn self_node(&self) -> JsValue {
        todo!()
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Default, Clone)]
pub struct WiredSceneTypes {}

#[wasm_bindgen]
impl WiredSceneTypes {
    // document
    pub fn document_add_asset(&self, handle: JsValue, name: String, blob_id: Vec<u8>) {}
    pub fn document_assets(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn document_clone(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn document_create_material(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn document_create_mesh(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn document_create_node(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn document_drop(&self) {}
    pub fn document_global_transform(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn document_id(&self, handle: JsValue) -> String {
        todo!()
    }
    pub fn document_materials(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn document_meshes(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn document_new(&self) {}
    pub fn document_nodes(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn document_public(&self, handle: JsValue) -> bool {
        todo!()
    }
    pub fn document_remove_asset(&self, handle: JsValue, name: String) {}
    pub fn document_remove_material(&self, handle: JsValue, value: JsValue) {}
    pub fn document_remove_mesh(&self, handle: JsValue, value: JsValue) {}
    pub fn document_remove_node(&self, handle: JsValue, value: JsValue) {}
    pub fn document_rep(&self) {}
    pub fn document_roots(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn document_rotation(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn document_scale(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn document_set_public(&self, handle: JsValue, value: bool) {}
    pub fn document_set_rotation(&self, handle: JsValue, value: JsValue) {}
    pub fn document_set_scale(&self, handle: JsValue, value: JsValue) {}
    pub fn document_set_sync(&self, handle: JsValue, value: bool) {}
    pub fn document_set_transform(&self, handle: JsValue, value: JsValue) {}
    pub fn document_set_translation(&self, handle: JsValue, value: JsValue) {}
    pub fn document_sync(&self, handle: JsValue) -> bool {
        todo!()
    }
    pub fn document_transform(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn document_translation(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    // material
    pub fn material_alpha_cutoff(&self, handle: JsValue) -> f32 {
        todo!()
    }
    pub fn material_alpha_mode(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn material_base_color(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn material_clone(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn material_double_sided(&self, handle: JsValue) -> bool {
        todo!()
    }
    pub fn material_drop(&self) {}
    pub fn material_id(&self, handle: JsValue) -> String {
        todo!()
    }
    pub fn material_metallic(&self, handle: JsValue) -> f32 {
        todo!()
    }
    pub fn material_name(&self, handle: JsValue) -> Option<String> {
        todo!()
    }
    pub fn material_new(&self) {}
    pub fn material_rep(&self) {}
    pub fn material_roughness(&self, handle: JsValue) -> f32 {
        todo!()
    }
    pub fn material_set_alpha_cutoff(&self, handle: JsValue, value: f32) {}
    pub fn material_set_alpha_mode(&self, handle: JsValue, value: JsValue) {}
    pub fn material_set_base_color(&self, handle: JsValue, value: JsValue) {}
    pub fn material_set_double_sided(&self, handle: JsValue, value: bool) {}
    pub fn material_set_metallic(&self, handle: JsValue, value: f32) {}
    pub fn material_set_name(&self, handle: JsValue, value: Option<String>) {}
    pub fn material_set_roughness(&self, handle: JsValue, value: f32) {}
    pub fn material_set_sync(&self, handle: JsValue, value: bool) {}
    pub fn material_set_unlit(&self, handle: JsValue, value: bool) {}
    pub fn material_sync(&self, handle: JsValue) -> bool {
        todo!()
    }
    pub fn material_unlit(&self, handle: JsValue) -> bool {
        todo!()
    }
    // mesh
    pub fn mesh_clone(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn mesh_colors(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn mesh_drop(&self) {}
    pub fn mesh_id(&self, handle: JsValue) -> String {
        todo!()
    }
    pub fn mesh_indices(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn mesh_name(&self, handle: JsValue) -> Option<String> {
        todo!()
    }
    pub fn mesh_new(&self) {}
    pub fn mesh_normals(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn mesh_positions(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn mesh_rep(&self) {}
    pub fn mesh_set_colors(&self, handle: JsValue, value: JsValue) {}
    pub fn mesh_set_indices(&self, handle: JsValue, value: JsValue) {}
    pub fn mesh_set_name(&self, handle: JsValue, value: Option<String>) {}
    pub fn mesh_set_normals(&self, handle: JsValue, value: JsValue) {}
    pub fn mesh_set_positions(&self, handle: JsValue, value: JsValue) {}
    pub fn mesh_set_sync(&self, handle: JsValue, value: bool) {}
    pub fn mesh_set_tangents(&self, handle: JsValue, value: JsValue) {}
    pub fn mesh_set_topology(&self, handle: JsValue, value: String) {}
    pub fn mesh_set_uv0(&self, handle: JsValue, value: JsValue) {}
    pub fn mesh_set_uv1(&self, handle: JsValue, value: JsValue) {}
    pub fn mesh_sync(&self, handle: JsValue) -> bool {
        todo!()
    }
    pub fn mesh_tangents(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn mesh_topology(&self, handle: JsValue) -> String {
        todo!()
    }
    pub fn mesh_uv0(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn mesh_uv1(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    // node
    pub fn node_add_child(&self, handle: JsValue, child: JsValue) {}
    pub fn node_children(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn node_clone(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn node_collider(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn node_drop(&self) {}
    pub fn node_global_transform(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn node_id(&self, handle: JsValue) -> String {
        todo!()
    }
    pub fn node_material(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn node_mesh(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn node_name(&self, handle: JsValue) -> Option<String> {
        todo!()
    }
    pub fn node_new(&self) {}
    pub fn node_parent(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn node_remove_child(&self, handle: JsValue, child: JsValue) {}
    pub fn node_rep(&self) {}
    pub fn node_rigid_body(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn node_rotation(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn node_scale(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn node_set_collider(&self, handle: JsValue, value: JsValue) {}
    pub fn node_set_material(&self, handle: JsValue, value: JsValue) {}
    pub fn node_set_mesh(&self, handle: JsValue, value: JsValue) {}
    pub fn node_set_name(&self, handle: JsValue, value: Option<String>) {}
    pub fn node_set_rigid_body(&self, handle: JsValue, value: JsValue) {}
    pub fn node_set_rotation(&self, handle: JsValue, value: JsValue) {}
    pub fn node_set_scale(&self, handle: JsValue, value: JsValue) {}
    pub fn node_set_sync(&self, handle: JsValue, value: bool) {}
    pub fn node_set_transform(&self, handle: JsValue, value: JsValue) {}
    pub fn node_set_translation(&self, handle: JsValue, value: JsValue) {}
    pub fn node_sync(&self, handle: JsValue) -> bool {
        todo!()
    }
    pub fn node_transform(&self, handle: JsValue) -> JsValue {
        todo!()
    }
    pub fn node_translation(&self, handle: JsValue) -> JsValue {
        todo!()
    }
}
