use std::{
    fs::File,
    io::{self, BufReader, Cursor, Read, Seek, SeekFrom},
    path::Path,
};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
enum XacChunkType {
    XacMeshId = 1,
    XacSkinningId = 2,
    XacMaterialDefinitionId = 3,
    XacShaderMaterialId = 5,
    XacMetadataId = 7,
    XacNodeHierarchyId = 11,
    XacMorphTargetId = 12,
    XacMaterialTotalId = 13,
}

#[derive(Debug, Serialize, Deserialize)]
enum XacVerticesAttributeType {
    XacPositionId = 0,
    XacNormalId = 1,
    XacTangentId = 2,
    XacUVCoordId = 3,
    XacColor32Id = 4,
    XacInfluenceRangeId = 5,
    XacColor128Id = 6,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacVec2d {
    x: f32,
    y: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacVec3d {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacVec4d {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacColor {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacColor8 {
    x: u8,
    y: u8,
    z: u8,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacQuaternion {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacMatrix44 {
    axis_1: XacVec4d,
    axis_2: XacVec4d,
    axis_3: XacVec4d,
    pos: XacVec4d,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacChunk {
    type_id: i32,
    length: u32,
    version: u32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct XacFile {
    header: XacHeader,
    metadata: XacMetaData,
    node_hierarchy: XacNodeHierarchy,
    material_totals: XacMaterialTotals,
    material_definition: XacMaterialDefinition,
    shader_material: Vec<XacShaderMaterial>,
    mesh_data: Vec<XacMesh>,
    skinning: XacSkinning,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacHeader {
    magic: String,
    major_version: u8,
    minor_version: u8,
    big_endian: bool,
    multiply_order: u8,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacMetaData {
    reposition_mask: i32,
    repositioning_node: i32,
    exporter_major_version: u8,
    exporter_minor_version: u8,
    retarget_root_offset: f32,
    source_app: String,
    original_filename: String,
    export_date: String,
    actor_name: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacNodeHierarchy {
    num_nodes: u32,
    num_root_nodes: u32,
    node: Vec<XacNode>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacNode {
    rotation: XacQuaternion,
    scale_rotation: XacQuaternion,
    position: XacVec3d,
    scale: XacVec3d,
    parent_node_id: i32,
    num_children: u32,
    include_inbounds_calc: i32,
    transform: XacMatrix44,
    importance_factor: f32,
    name: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacMaterialTotals {
    num_total_materials: u32,
    num_standard_materials: u32,
    num_fx_materials: u32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacMaterialDefinition {
    ambient_color: XacVec4d,
    diffuse_color: XacVec4d,
    specular_color: XacVec4d,
    emissive_color: XacVec4d,
    shine: f32,
    shine_strength: f32,
    opacity: f32,
    ior: f32,
    double_sided: bool,
    wireframe: bool,
    num_layers: u8,
    name: String,
    layers: Vec<XacMaterialLayer>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacMaterialLayer {
    amount: f32,
    v_offset: f32,
    u_offset: f32,
    v_tiling: f32,
    u_tiling: f32,
    rotation_in_radians: f32,
    material_id: i16,
    map_type: u8,
    name: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacIntProperties {
    name_properties: String,
    value: i32,
}
#[derive(Default, Debug, Serialize, Deserialize)]
struct XacFloatProperties {
    name_properties: String,
    value: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacBoolProperties {
    name_properties: String,
    value: u8,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacStringProperties {
    name_properties: String,
    value: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacShaderMaterial {
    num_int: u32,
    num_float: u32,
    num_bool: u32,
    num_string: u32,
    flag: u32,
    name_material: String,
    name_shader: String,
    int_property: Vec<XacIntProperties>,
    float_property: Vec<XacFloatProperties>,
    bool_property: Vec<XacBoolProperties>,
    string_property: Vec<XacStringProperties>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacVerticesAttribute {
    type_id: i32,
    attribute_size: u32,
    keep_original: bool,
    scale_factor: bool,
    vertex_positions: Vec<XacVec3d>,
    vertex_normals: Vec<XacVec3d>,
    vertex_tangents: Vec<XacVec4d>,
    vertex_bi_tangents: Vec<XacVec4d>,
    vertex_uvs: Vec<XacVec2d>,
    vertex_colors_32: Vec<XacColor8>,
    vertex_colors_128: Vec<XacVec3d>,
    vertex_influences: Vec<u32>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacSubMesh {
    num_indices: u32,
    num_vertices: u32,
    material_id: i32,
    num_bones: u32,
    relative_indices: Vec<u32>,
    bone_id: Vec<u32>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacMesh {
    node_id: i32,
    num_influence_ranges: u32,
    num_vertices: u32,
    num_indices: u32,
    num_sub_meshes: u32,
    num_attribute_layer: u32,
    collision_mesh: bool,
    vertices_attribute: XacVerticesAttribute,
    sub_mesh: Vec<XacSubMesh>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacInfluenceData {
    weight: f32,
    bone_id: i32,
}
#[derive(Default, Debug, Serialize, Deserialize)]
struct XacInfluenceRange {
    first_influence_index: i32,
    num_influences: u32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct XacSkinning {
    node_id: i32,
    num_local_bones: u32,
    num_influences: u32,
    collision_mesh: bool,
    influence_data: Vec<XacInfluenceData>,
    influence_range: Vec<XacInfluenceRange>,
}

const MAGIC_NUMBER: usize = 4;

impl XacFile {
    /// Load XAC data from a file specified by the file path.
    ///
    /// # Arguments
    ///
    /// * `file_path` - A path to the XAC file to be loaded.
    ///
    /// # Returns
    ///
    /// A Result containing the parsed `XacFile` or an IO error if the file cannot be read.
    pub fn load_from_file<P: AsRef<Path>>(file_path: P) -> io::Result<Self> {
        // Check if the file extension is '.xac'
        let file_path_str = file_path.as_ref().to_str().unwrap_or("");
        if !file_path_str.to_lowercase().ends_with(".xac") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid file extension. Expected '.xac'.",
            ));
        }

        // Open the file and create a buffered reader.
        let file = std::fs::File::open(file_path)?;
        let mut buf_reader = BufReader::new(file);

        // Delegate to load_from_reader for further processing.
        Self::load_from_reader(&mut buf_reader)
    }

    /// Load XAC data from a byte vector.
    ///
    /// # Arguments
    ///
    /// * `bytes` - A vector of bytes containing the XAC file data.
    ///
    /// # Returns
    ///
    /// A Result containing the parsed `XacFile` or an IO error if the byte vector is invalid.
    pub fn load_from_bytes(mut bytes: Vec<u8>) -> io::Result<Self> {
        // Create a cursor from the byte vector.
        let mut cursor = Cursor::new(&mut bytes);

        // Delegate to load_from_reader for further processing.
        Self::load_from_reader(&mut cursor)
    }

    fn load_from_reader<R: Read + Seek>(reader: &mut R) -> io::Result<Self> {
        let mut xac_data = XacFile::default();
        xac_data.read_header(reader)?;
        xac_data.read_chunk(reader)?;
        Ok(xac_data)
    }
    fn read_header<'a, R: Read + Seek>(&'a mut self, file: &'a mut R) -> io::Result<&mut Self> {
        const MAGIC_NUMBER: usize = 4;

        let mut magic = [0; MAGIC_NUMBER];
        file.read_exact(&mut magic).unwrap();
        self.header.magic = std::str::from_utf8(&magic).unwrap().to_string();

        if self.header.magic != "XAC " {
            panic!("Not an XAC file: invalid header magic");
        }

        self.header.major_version = file.read_u8().unwrap();
        self.header.minor_version = file.read_u8().unwrap();

        if self.header.major_version != 1 || self.header.minor_version != 0 {
            panic!(
                "Unsupported .xac version: expected v1.0, file is {}.{}",
                self.header.major_version, self.header.minor_version
            );
        }

        self.header.big_endian = self.xac_read_boolean(file);

        if self.header.big_endian {
            panic!("XAC file is encoded in big endian which is not supported by this importer");
        }

        self.header.multiply_order = file.read_u8().unwrap();
        Ok(self)
    }
    fn read_chunk<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        while file.stream_position().unwrap() < file.stream_len().unwrap() {
            let chunk = XacChunk {
                type_id: file.read_i32::<LittleEndian>().unwrap(),
                length: file.read_u32::<LittleEndian>().unwrap(),
                version: file.read_u32::<LittleEndian>().unwrap(),
            };
            let position = file.stream_position().unwrap();

            if chunk.type_id as u32 == XacChunkType::XacMeshId as u32 {
                self.read_mesh(file)?;
            }

            if chunk.type_id as u32 == XacChunkType::XacSkinningId as u32 {
                self.read_skinning(file)?;
            }
            if chunk.type_id as u32 == XacChunkType::XacMaterialDefinitionId as u32 {
                self.read_material_definition(file)?;
            }
            if chunk.type_id as u32 == XacChunkType::XacShaderMaterialId as u32 {
                self.read_shader_material(file)?;
            }

            if chunk.type_id as u32 == XacChunkType::XacMetadataId as u32 {
                self.read_metadata(file)?;
            }
            if chunk.type_id as u32 == XacChunkType::XacNodeHierarchyId as u32 {
                self.read_node_hierarchy(file)?;
            }
            if chunk.type_id as u32 == XacChunkType::XacMorphTargetId as u32 {
                // Unfinished
                println!("Morph Target Data Found!");
            }
            if chunk.type_id as u32 == XacChunkType::XacMaterialTotalId as u32 {
                self.read_material_total(file)?;
            }

            file.seek(SeekFrom::Start(position + chunk.length as u64))
                .unwrap();
        }
        Ok(self)
    }

    fn read_metadata<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        self.metadata.reposition_mask = file.read_i32::<LittleEndian>().unwrap();
        self.metadata.repositioning_node = file.read_i32::<LittleEndian>().unwrap();
        self.metadata.exporter_major_version = file.read_u8().unwrap();
        self.metadata.exporter_minor_version = file.read_u8().unwrap();
        file.read_u8().unwrap(); // Padding
        file.read_u8().unwrap(); // Padding
        self.metadata.retarget_root_offset = file.read_f32::<LittleEndian>().unwrap();
        self.metadata.source_app = self.xac_read_string(file)?;
        self.metadata.original_filename = self.xac_read_string(file)?;
        self.metadata.export_date = self.xac_read_string(file)?;
        self.metadata.actor_name = self.xac_read_string(file)?;
        Ok(self)
    }

    fn read_node_hierarchy<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        self.node_hierarchy.num_nodes = file.read_u32::<LittleEndian>().unwrap();
        self.node_hierarchy.num_root_nodes = file.read_u32::<LittleEndian>().unwrap();

        for _ in 0..self.node_hierarchy.num_nodes {
            let mut nodes = XacNode::default();

            nodes.rotation = self.xac_read_quaternion(file);
            nodes.scale_rotation = self.xac_read_quaternion(file);
            nodes.position = self.xac_read_vec3d(file);
            nodes.scale = self.xac_read_vec3d(file);

            file.read_i32::<LittleEndian>().unwrap(); //Padding
            file.read_i32::<LittleEndian>().unwrap(); //Padding
            file.read_i32::<LittleEndian>().unwrap(); //Padding
            file.read_i32::<LittleEndian>().unwrap(); //Padding
            file.read_i32::<LittleEndian>().unwrap(); //Padding

            nodes.parent_node_id = file.read_i32::<LittleEndian>().unwrap();
            nodes.num_children = file.read_u32::<LittleEndian>().unwrap();
            nodes.include_inbounds_calc = file.read_i32::<LittleEndian>().unwrap();

            nodes.transform = self.xac_read_matrix44(file);
            nodes.importance_factor = file.read_f32::<LittleEndian>().unwrap();
            nodes.name = self.xac_read_string(file)?;

            self.node_hierarchy.node.push(nodes);
        }

        Ok(self)
    }

    fn read_material_total<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        self.material_totals.num_total_materials = file.read_u32::<LittleEndian>().unwrap();
        self.material_totals.num_standard_materials = file.read_u32::<LittleEndian>().unwrap();
        self.material_totals.num_fx_materials = file.read_u32::<LittleEndian>().unwrap();
        Ok(self)
    }

    fn read_material_definition<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        self.material_definition.ambient_color = self.xac_read_vec4d(file);
        self.material_definition.diffuse_color = self.xac_read_vec4d(file);
        self.material_definition.specular_color = self.xac_read_vec4d(file);
        self.material_definition.emissive_color = self.xac_read_vec4d(file);
        self.material_definition.shine = file.read_f32::<LittleEndian>().unwrap();
        self.material_definition.shine_strength = file.read_f32::<LittleEndian>().unwrap();
        self.material_definition.opacity = file.read_f32::<LittleEndian>().unwrap();
        self.material_definition.ior = file.read_f32::<LittleEndian>().unwrap();
        self.material_definition.double_sided = self.xac_read_boolean(file);
        self.material_definition.wireframe = self.xac_read_boolean(file);
        file.read_u8().unwrap(); //Padding
        self.material_definition.num_layers = file.read_u8().unwrap();
        self.material_definition.name = self.xac_read_string(file)?;

        for _i in 0..self.material_definition.num_layers {
            let mut layer_info = XacMaterialLayer::default();

            layer_info.amount = file.read_f32::<LittleEndian>().unwrap();
            layer_info.u_offset = file.read_f32::<LittleEndian>().unwrap();
            layer_info.v_offset = file.read_f32::<LittleEndian>().unwrap();
            layer_info.u_tiling = file.read_f32::<LittleEndian>().unwrap();
            layer_info.v_tiling = file.read_f32::<LittleEndian>().unwrap();
            layer_info.rotation_in_radians = file.read_f32::<LittleEndian>().unwrap();
            layer_info.material_id = file.read_i16::<LittleEndian>().unwrap();
            layer_info.map_type = file.read_u8().unwrap();
            file.read_u8().unwrap(); //Padding

            layer_info.name = self.xac_read_string(file)?;

            self.material_definition.layers.push(layer_info);
        }

        Ok(self)
    }

    fn read_shader_material<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        let mut shader_data = XacShaderMaterial::default();
        shader_data.num_int = file.read_u32::<LittleEndian>().unwrap();
        shader_data.num_float = file.read_u32::<LittleEndian>().unwrap();
        file.read_u32::<LittleEndian>().unwrap(); // Padding
        shader_data.num_bool = file.read_u32::<LittleEndian>().unwrap();
        shader_data.flag = file.read_u32::<LittleEndian>().unwrap();
        shader_data.num_string = file.read_u32::<LittleEndian>().unwrap();
        shader_data.name_material = self.xac_read_string(file)?;
        shader_data.name_shader = self.xac_read_string(file)?;

        for _ in 0..shader_data.num_int {
            shader_data.int_property.push(XacIntProperties {
                name_properties: self.xac_read_string(file)?,
                value: file.read_i32::<LittleEndian>().unwrap(),
            });
        }

        for _ in 0..shader_data.num_float {
            shader_data.float_property.push(XacFloatProperties {
                name_properties: self.xac_read_string(file)?,
                value: file.read_f32::<LittleEndian>().unwrap(),
            });
        }

        for _ in 0..shader_data.num_bool {
            shader_data.bool_property.push(XacBoolProperties {
                name_properties: self.xac_read_string(file)?,
                value: file.read_u8().unwrap(),
            });
        }

        let skip = file.read_i32::<LittleEndian>().unwrap();
        for _ in 0..skip {
            file.read_u8().unwrap();
        }

        for _ in 0..shader_data.num_string {
            shader_data.string_property.push(XacStringProperties {
                name_properties: self.xac_read_string(file)?,
                value: self.xac_read_string(file)?,
            });
        }

        self.shader_material.push(shader_data);
        Ok(self)
    }

    fn read_mesh<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        let mut mesh_info = XacMesh::default();
        mesh_info.node_id = file.read_i32::<LittleEndian>().unwrap();
        mesh_info.num_influence_ranges = file.read_u32::<LittleEndian>().unwrap();
        mesh_info.num_vertices = file.read_u32::<LittleEndian>().unwrap();
        mesh_info.num_indices = file.read_u32::<LittleEndian>().unwrap();
        mesh_info.num_sub_meshes = file.read_u32::<LittleEndian>().unwrap();
        mesh_info.num_attribute_layer = file.read_u32::<LittleEndian>().unwrap();
        mesh_info.collision_mesh = self.xac_read_boolean(file);
        file.read_u8().unwrap(); // Padding
        file.read_u8().unwrap(); // Padding
        file.read_u8().unwrap(); // Padding

        for _ in 0..mesh_info.num_attribute_layer {
            let mut vertices_attribute = mesh_info.vertices_attribute;
            vertices_attribute.type_id = file.read_u32::<LittleEndian>().unwrap();
            vertices_attribute.attribute_size = file.read_u32::<LittleEndian>().unwrap();
            vertices_attribute.keep_original = self.xac_read_boolean(file);
            vertices_attribute.scale_factor = self.xac_read_boolean(file);
            file.read_u8().unwrap(); // Padding
            file.read_u8().unwrap(); // Padding

            match vertices_attribute.type_id {
                XacPositionId => {
                    for _ in 0..mesh_info.num_vertices {
                        vertices_attribute
                            .vertex_positions
                            .push(self.xac_read_vec3d(file));
                    }
                }
                XacNormalId => {
                    for _ in 0..mesh_info.num_vertices {
                        vertices_attribute
                            .vertex_normals
                            .push(self.xac_read_vec3d(file))
                    }
                }
                XacTangentId => {
                    if vertices_attribute.vertex_tangents.is_empty() {
                        for _ in 0..mesh_info.num_vertices {
                            vertices_attribute
                                .vertex_tangents
                                .push(self.xac_read_vec4d(file));
                        }
                    } else if vertices_attribute.vertex_bi_tangents.is_empty() {
                        for _ in 0..mesh_info.num_vertices {
                            vertices_attribute
                                .vertex_bi_tangents
                                .push(self.xac_read_vec4d(file));
                        }
                    }
                }
                XacUVCoordId => {
                    for _ in 0..mesh_info.num_vertices {
                        vertices_attribute
                            .vertex_uvs
                            .push(self.xac_read_vec2d(file));
                    }
                }
                XacColor32Id => {
                    for _ in 0..mesh_info.num_vertices {
                        vertices_attribute
                            .vertex_colors_32
                            .push(self.xac_read_color8(file));
                    }
                }
                XacInfluenceRangeId => {
                    for _ in 0..mesh_info.num_vertices {
                        vertices_attribute
                            .vertex_influences
                            .push(file.read_u32::<LittleEndian>().unwrap());
                    }
                }
                XacColor128Id => {
                    for _ in 0..mesh_info.num_vertices {
                        vertices_attribute
                            .vertex_colors_128
                            .push(self.xac_read_vec3d(file));
                    }
                }
                _ => {} // Handle other cases if needed
            }
            mesh_info.vertices_attribute = vertices_attribute;
        }

        for _ in 0..mesh_info.num_sub_meshes {
            let mut sub_mesh = XacSubMesh::default();
            sub_mesh.num_indices = file.read_u32::<LittleEndian>().unwrap();
            sub_mesh.num_vertices = file.read_u32::<LittleEndian>().unwrap();
            sub_mesh.material_id = file.read_i32::<LittleEndian>().unwrap();
            sub_mesh.num_bones = file.read_u32::<LittleEndian>().unwrap();

            for _ in 0..sub_mesh.num_indices {
                sub_mesh
                    .relative_indices
                    .push(file.read_u32::<LittleEndian>().unwrap());
            }
            for _ in 0..sub_mesh.num_bones {
                sub_mesh
                    .bone_id
                    .push(file.read_u32::<LittleEndian>().unwrap());
            }
            mesh_info.sub_mesh.push(sub_mesh);
        }
        self.mesh_data.push(mesh_info);
        Ok(self)
    }
    fn read_skinning<R: Read + Seek>(&mut self, file: &mut R) -> io::Result<&mut Self> {
        self.skinning.node_id = file.read_i32::<LittleEndian>().unwrap();
        self.skinning.num_local_bones = file.read_u32::<LittleEndian>().unwrap();
        self.skinning.num_influences = file.read_u32::<LittleEndian>().unwrap();
        self.skinning.collision_mesh = self.xac_read_boolean(file);
        file.read_u8().unwrap(); // Padding
        file.read_u8().unwrap(); // Padding
        file.read_u8().unwrap(); // Padding

        for _ in 0..self.skinning.num_influences {
            self.skinning.influence_data.push(XacInfluenceData {
                weight: file.read_f32::<LittleEndian>().unwrap(),
                bone_id: file.read_i32::<LittleEndian>().unwrap(),
            });
        }
        for _ in 0..self.mesh_data[self.mesh_data.len() - 1usize].num_influence_ranges {
            self.skinning.influence_range.push(XacInfluenceRange {
                first_influence_index: file.read_i32::<LittleEndian>().unwrap(),
                num_influences: file.read_u32::<LittleEndian>().unwrap(),
            });
        }
        Ok(self)
    }
    fn xac_read_string<R: Read + Seek>(&mut self, reader: &mut R) -> io::Result<String> {
        let mut text = String::new();
        let length = reader.read_i32::<LittleEndian>().unwrap();
        for _ in 0..length {
            let character = reader.read_u8().unwrap();
            text.push(character as char);
        }
        Ok(text)
    }

    fn xac_read_boolean<R: Read + Seek>(&mut self, file: &mut R) -> bool {
        file.read_u8().unwrap() != 0
    }

    fn xac_read_color8<R: Read + Seek>(&mut self, file: &mut R) -> XacColor8 {
        XacColor8 {
            x: file.read_u8().unwrap(),
            y: file.read_u8().unwrap(),
            z: file.read_u8().unwrap(),
        }
    }

    fn xac_read_color<R: Read + Seek>(&mut self, file: &mut R) -> XacColor {
        XacColor {
            x: file.read_f32::<LittleEndian>().unwrap(),
            y: file.read_f32::<LittleEndian>().unwrap(),
            z: file.read_f32::<LittleEndian>().unwrap(),
        }
    }

    fn xac_read_vec2d<R: Read + Seek>(&mut self, file: &mut R) -> XacVec2d {
        XacVec2d {
            x: file.read_f32::<LittleEndian>().unwrap(),
            y: file.read_f32::<LittleEndian>().unwrap(),
        }
    }

    fn xac_read_vec3d<R: Read + Seek>(&mut self, file: &mut R) -> XacVec3d {
        XacVec3d {
            x: file.read_f32::<LittleEndian>().unwrap(),
            y: file.read_f32::<LittleEndian>().unwrap(),
            z: file.read_f32::<LittleEndian>().unwrap(),
        }
    }

    fn xac_read_vec4d<R: Read + Seek>(&mut self, file: &mut R) -> XacVec4d {
        XacVec4d {
            x: file.read_f32::<LittleEndian>().unwrap(),
            y: file.read_f32::<LittleEndian>().unwrap(),
            z: file.read_f32::<LittleEndian>().unwrap(),
            w: file.read_f32::<LittleEndian>().unwrap(),
        }
    }

    fn xac_read_quaternion<R: Read + Seek>(&mut self, file: &mut R) -> XacQuaternion {
        XacQuaternion {
            x: file.read_f32::<LittleEndian>().unwrap(),
            y: file.read_f32::<LittleEndian>().unwrap(),
            z: file.read_f32::<LittleEndian>().unwrap(),
            w: file.read_f32::<LittleEndian>().unwrap(),
        }
    }

    fn xac_read_matrix44<R: Read + Seek>(&mut self, file: &mut R) -> XacMatrix44 {
        XacMatrix44 {
            axis_1: self.xac_read_vec4d(file),
            axis_2: self.xac_read_vec4d(file),
            axis_3: self.xac_read_vec4d(file),
            pos: self.xac_read_vec4d(file),
        }
    }
}

#[test]
fn test_xac_parser() {
    // Provide the path to the test IES file
    let file_path = "tests/archer_m_falconer01.xac";

    // Read the content of the file
    let mut file_content = Vec::new();
    let mut file = File::open(&file_path).expect("Failed to open file");
    file.read_to_end(&mut file_content)
        .expect("Failed to read file content");

    // Parse the IES file
    let _ = XacFile::load_from_bytes(file_content).expect("Failed to parse XSM file");
}
