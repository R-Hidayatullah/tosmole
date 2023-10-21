#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacVec2d {
    pub(crate) x: f32,
    pub(crate) y: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacVec3d {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacVec4d {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
    pub(crate) w: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacColor {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacColor8 {
    pub(crate) x: u8,
    pub(crate) y: u8,
    pub(crate) z: u8,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacQuaternion {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) z: f32,
    pub(crate) w: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacMatrix44 {
    pub(crate) axis_1: XacVec4d,
    pub(crate) axis_2: XacVec4d,
    pub(crate) axis_3: XacVec4d,
    pub(crate) pos: XacVec4d,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacChunk {
    pub(crate) type_id: u32,
    pub(crate) length: u32,
    pub(crate) version: u32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct XacFile {
    pub(crate) header: XacHeader,
    pub(crate) metadata: XacMetaData,
    pub(crate) node_hierarchy: XacNodeHierarchy,
    pub(crate) material_totals: XacMaterialTotals,
    pub(crate) material_definition: XacMaterialDefinition,
    pub(crate) shader_material: Vec<XacShaderMaterial>,
    pub(crate) mesh_data: Vec<XacMesh>,
    pub(crate) skinning: XacSkinning,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacHeader {
    pub(crate) magic: String,
    pub(crate) major_version: u8,
    pub(crate) minor_version: u8,
    pub(crate) big_endian: bool,
    pub(crate) multiply_order: u8,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacMetaData {
    pub(crate) reposition_mask: u32,
    pub(crate) repositioning_node: u32,
    pub(crate) exporter_major_version: u8,
    pub(crate) exporter_minor_version: u8,
    pub(crate) retarget_root_offset: f32,
    pub(crate) source_app: String,
    pub(crate) original_filename: String,
    pub(crate) export_date: String,
    pub(crate) actor_name: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacNodeHierarchy {
    pub(crate) num_nodes: u32,
    pub(crate) num_root_nodes: u32,
    pub(crate) node: Vec<XacNode>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacNode {
    pub(crate) rotation: XacQuaternion,
    pub(crate) scale_rotation: XacQuaternion,
    pub(crate) position: XacVec3d,
    pub(crate) scale: XacVec3d,
    pub(crate) parent_node_id: u32,
    pub(crate) num_children: u32,
    pub(crate) include_inbounds_calc: u32,
    pub(crate) transform: XacMatrix44,
    pub(crate) importance_factor: f32,
    pub(crate) name: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacMaterialTotals {
    pub(crate) num_total_materials: u32,
    pub(crate) num_standard_materials: u32,
    pub(crate) num_fx_materials: u32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacMaterialDefinition {
    pub(crate) ambient_color: XacVec4d,
    pub(crate) diffuse_color: XacVec4d,
    pub(crate) specular_color: XacVec4d,
    pub(crate) emissive_color: XacVec4d,
    pub(crate) shine: f32,
    pub(crate) shine_strength: f32,
    pub(crate) opacity: f32,
    pub(crate) ior: f32,
    pub(crate) double_sided: bool,
    pub(crate) wireframe: bool,
    pub(crate) num_layers: u8,
    pub(crate) name: String,
    pub(crate) layers: Vec<XacMaterialLayer>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacMaterialLayer {
    pub(crate) amount: f32,
    pub(crate) v_offset: f32,
    pub(crate) u_offset: f32,
    pub(crate) v_tiling: f32,
    pub(crate) u_tiling: f32,
    pub(crate) rotation_in_radians: f32,
    pub(crate) material_id: i16,
    pub(crate) map_type: u8,
    pub(crate) name: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacIntProperties {
    pub(crate) name_properties: String,
    pub(crate) value: u32,
}
#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacFloatProperties {
    pub(crate) name_properties: String,
    pub(crate) value: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacBoolProperties {
    pub(crate) name_properties: String,
    pub(crate) value: u8,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacStringProperties {
    pub(crate) name_properties: String,
    pub(crate) value: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacShaderMaterial {
    pub(crate) num_int: u32,
    pub(crate) num_float: u32,
    pub(crate) num_bool: u32,
    pub(crate) num_string: u32,
    pub(crate) flag: u32,
    pub(crate) name_material: String,
    pub(crate) name_shader: String,
    pub(crate) int_property: Vec<XacIntProperties>,
    pub(crate) float_property: Vec<XacFloatProperties>,
    pub(crate) bool_property: Vec<XacBoolProperties>,
    pub(crate) string_property: Vec<XacStringProperties>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacVerticesAttribute {
    pub(crate) type_id: u32,
    pub(crate) attribute_size: u32,
    pub(crate) keep_original: bool,
    pub(crate) scale_factor: bool,
    pub(crate) vertex_positions: Vec<XacVec3d>,
    pub(crate) vertex_normals: Vec<XacVec3d>,
    pub(crate) vertex_tangents: Vec<XacVec4d>,
    pub(crate) vertex_bi_tangents: Vec<XacVec4d>,
    pub(crate) vertex_uvs: Vec<XacVec2d>,
    pub(crate) vertex_colors_32: Vec<XacColor8>,
    pub(crate) vertex_colors_128: Vec<XacVec3d>,
    pub(crate) vertex_influences: Vec<u32>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacSubMesh {
    pub(crate) num_indices: u32,
    pub(crate) num_vertices: u32,
    pub(crate) material_id: u32,
    pub(crate) num_bones: u32,
    pub(crate) relative_indices: Vec<u32>,
    pub(crate) bone_id: Vec<u32>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacMesh {
    pub(crate) node_id: u32,
    pub(crate) num_influence_ranges: u32,
    pub(crate) num_vertices: u32,
    pub(crate) num_indices: u32,
    pub(crate) num_sub_meshes: u32,
    pub(crate) num_attribute_layer: u32,
    pub(crate) collision_mesh: bool,
    pub(crate) vertices_attribute: XacVerticesAttribute,
    pub(crate) sub_mesh: Vec<XacSubMesh>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacInfluenceData {
    pub(crate) weight: f32,
    pub(crate) bone_id: u32,
}
#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacInfluenceRange {
    pub(crate) first_influence_index: u32,
    pub(crate) num_influences: u32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub(crate) struct XacSkinning {
    pub(crate) node_id: u32,
    pub(crate) num_local_bones: u32,
    pub(crate) num_influences: u32,
    pub(crate) collision_mesh: bool,
    pub(crate) influence_data: Vec<XacInfluenceData>,
    pub(crate) influence_range: Vec<XacInfluenceRange>,
}
