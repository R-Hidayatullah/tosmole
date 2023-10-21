#![allow(dead_code)]

use std::io::{Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::xac::xac_enum::XacChunkType::{
    XacMaterialDefinitionId, XacMaterialTotalId, XacMeshId, XacMetadataId, XacMorphTargetId,
    XacNodeHierarchyId, XacShaderMaterialId, XacSkinningId,
};
use crate::xac::xac_enum::XacVerticesAttributeType::{
    XacColor128Id, XacColor32Id, XacInfluenceRangeId, XacNormalId, XacPositionId, XacTangentId,
    XacUVCoordId,
};
use crate::xac::xac_struct::{
    XacBoolProperties, XacChunk, XacFile, XacFloatProperties, XacInfluenceData, XacInfluenceRange,
    XacIntProperties, XacMaterialLayer, XacMesh, XacNode, XacShaderMaterial, XacStringProperties,
    XacSubMesh,
};
use crate::xac::xac_util::{
    xac_read_boolean, xac_read_color8, xac_read_matrix44, xac_read_quaternion, xac_read_string,
    xac_read_vec2d, xac_read_vec3d, xac_read_vec4d,
};

const MAGIC_NUMBER: usize = 4;

pub fn xac_parse<R: Read + Seek>(xac_file: &mut R) -> XacFile {
    let mut xac_actor = XacFile::default();
    read_header(xac_file, &mut xac_actor);
    read_chunk(xac_file, &mut xac_actor);
    xac_actor
}

fn read_header<'a, R: Read + Seek>(file: &'a mut R, xac: &'a mut XacFile) -> &'a mut XacFile {
    let mut magic = [0; MAGIC_NUMBER];
    file.read_exact(&mut magic).unwrap();
    xac.header.magic = std::str::from_utf8(&magic).unwrap().to_string();
    if xac.header.magic != "XAC " {
        panic!("Not an XAC file: invalid header magic");
    }
    xac.header.major_version = file.read_u8().unwrap();
    xac.header.minor_version = file.read_u8().unwrap();
    if xac.header.major_version != 1 || xac.header.minor_version != 0 {
        panic!(
            "Unsupported .xac version: expected v1.0, file is {}.{}",
            xac.header.major_version, xac.header.minor_version
        );
    }
    xac.header.big_endian = xac_read_boolean(file);
    if xac.header.big_endian {
        panic!("XAC file is encoded in big endian which is not supported by this importer");
    }
    xac.header.multiply_order = file.read_u8().unwrap();
    xac
}

fn read_chunk<'a, R: Read + Seek>(file: &'a mut R, xac: &'a mut XacFile) -> &'a mut XacFile {
    while file.stream_position().unwrap() < file.stream_len().unwrap() {
        let chunk = XacChunk {
            type_id: file.read_u32::<LittleEndian>().unwrap(),
            length: file.read_u32::<LittleEndian>().unwrap(),
            version: file.read_u32::<LittleEndian>().unwrap(),
        };
        let position = file.stream_position().unwrap();

        if chunk.type_id == XacMeshId as u32 {
            read_mesh(file, xac);
        }

        if chunk.type_id == XacSkinningId as u32 {
            read_skinning(file, xac);
        }
        if chunk.type_id == XacMaterialDefinitionId as u32 {
            read_material_definition(file, xac);
        }
        if chunk.type_id == XacShaderMaterialId as u32 {
            read_shader_material(file, xac);
        }

        if chunk.type_id == XacMetadataId as u32 {
            read_metadata(file, xac);
        }
        if chunk.type_id == XacNodeHierarchyId as u32 {
            read_node_hierarchy(file, xac);
        }
        if chunk.type_id == XacMorphTargetId as u32 {
            //Unfinished
            println!("Morph Target Data Found!");
        }
        if chunk.type_id == XacMaterialTotalId as u32 {
            read_material_total(file, xac);
        }

        file.seek(SeekFrom::Start(position + chunk.length as u64))
            .unwrap();
    }
    xac
}

fn read_metadata<'a, R: Read + Seek>(file: &'a mut R, xac: &'a mut XacFile) -> &'a mut XacFile {
    xac.metadata.reposition_mask = file.read_u32::<LittleEndian>().unwrap();
    xac.metadata.repositioning_node = file.read_u32::<LittleEndian>().unwrap();
    xac.metadata.exporter_major_version = file.read_u8().unwrap();
    xac.metadata.exporter_minor_version = file.read_u8().unwrap();
    file.read_u8().unwrap(); //Padding
    file.read_u8().unwrap(); //Padding
    xac.metadata.retarget_root_offset = file.read_f32::<LittleEndian>().unwrap();
    xac.metadata.source_app = xac_read_string(file);
    xac.metadata.original_filename = xac_read_string(file);
    xac.metadata.export_date = xac_read_string(file);
    xac.metadata.actor_name = xac_read_string(file);
    xac
}

fn read_node_hierarchy<'a, R: Read + Seek>(
    file: &'a mut R,
    xac: &'a mut XacFile,
) -> &'a mut XacFile {
    xac.node_hierarchy.num_nodes = file.read_u32::<LittleEndian>().unwrap();
    xac.node_hierarchy.num_root_nodes = file.read_u32::<LittleEndian>().unwrap();

    for _ in 0..xac.node_hierarchy.num_nodes {
        let mut nodes = XacNode::default();

        nodes.rotation = xac_read_quaternion(file);
        nodes.scale_rotation = xac_read_quaternion(file);
        nodes.position = xac_read_vec3d(file);
        nodes.scale = xac_read_vec3d(file);

        file.read_i32::<LittleEndian>().unwrap(); //Padding
        file.read_i32::<LittleEndian>().unwrap(); //Padding
        file.read_i32::<LittleEndian>().unwrap(); //Padding
        file.read_i32::<LittleEndian>().unwrap(); //Padding
        file.read_i32::<LittleEndian>().unwrap(); //Padding

        nodes.parent_node_id = file.read_u32::<LittleEndian>().unwrap();
        nodes.num_children = file.read_u32::<LittleEndian>().unwrap();
        nodes.include_inbounds_calc = file.read_u32::<LittleEndian>().unwrap();

        nodes.transform = xac_read_matrix44(file);
        nodes.importance_factor = file.read_f32::<LittleEndian>().unwrap();
        nodes.name = xac_read_string(file);

        xac.node_hierarchy.node.push(nodes);
    }

    xac
}

fn read_material_total<'a, R: Read + Seek>(
    file: &'a mut R,
    xac: &'a mut XacFile,
) -> &'a mut XacFile {
    xac.material_totals.num_total_materials = file.read_u32::<LittleEndian>().unwrap();
    xac.material_totals.num_standard_materials = file.read_u32::<LittleEndian>().unwrap();
    xac.material_totals.num_fx_materials = file.read_u32::<LittleEndian>().unwrap();
    xac
}

fn read_material_definition<'a, R: Read + Seek>(
    file: &'a mut R,
    xac: &'a mut XacFile,
) -> &'a mut XacFile {
    xac.material_definition.ambient_color = xac_read_vec4d(file);
    xac.material_definition.diffuse_color = xac_read_vec4d(file);
    xac.material_definition.specular_color = xac_read_vec4d(file);
    xac.material_definition.emissive_color = xac_read_vec4d(file);
    xac.material_definition.shine = file.read_f32::<LittleEndian>().unwrap();
    xac.material_definition.shine_strength = file.read_f32::<LittleEndian>().unwrap();
    xac.material_definition.opacity = file.read_f32::<LittleEndian>().unwrap();
    xac.material_definition.ior = file.read_f32::<LittleEndian>().unwrap();
    xac.material_definition.double_sided = xac_read_boolean(file);
    xac.material_definition.wireframe = xac_read_boolean(file);
    file.read_u8().unwrap(); //Padding
    xac.material_definition.num_layers = file.read_u8().unwrap();
    xac.material_definition.name = xac_read_string(file);

    for _i in 0..xac.material_definition.num_layers {
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

        layer_info.name = xac_read_string(file);

        xac.material_definition.layers.push(layer_info);
    }
    xac
}

fn read_shader_material<'a, R: Read + Seek>(
    file: &'a mut R,
    xac: &'a mut XacFile,
) -> &'a mut XacFile {
    let mut shader_data = XacShaderMaterial::default();
    shader_data.num_int = file.read_u32::<LittleEndian>().unwrap();
    shader_data.num_float = file.read_u32::<LittleEndian>().unwrap();
    file.read_u32::<LittleEndian>().unwrap(); //Padding
    shader_data.num_bool = file.read_u32::<LittleEndian>().unwrap();
    shader_data.flag = file.read_u32::<LittleEndian>().unwrap();
    shader_data.num_string = file.read_u32::<LittleEndian>().unwrap();
    shader_data.name_material = xac_read_string(file);
    shader_data.name_shader = xac_read_string(file);

    for _ in 0..shader_data.num_int {
        shader_data.int_property.push(XacIntProperties {
            name_properties: xac_read_string(file),
            value: file.read_u32::<LittleEndian>().unwrap(),
        });
    }

    for _ in 0..shader_data.num_float {
        shader_data.float_property.push(XacFloatProperties {
            name_properties: xac_read_string(file),
            value: file.read_f32::<LittleEndian>().unwrap(),
        });
    }

    for _ in 0..shader_data.num_bool {
        shader_data.bool_property.push(XacBoolProperties {
            name_properties: xac_read_string(file),
            value: file.read_u8().unwrap(),
        });
    }
    let skip = file.read_i32::<LittleEndian>().unwrap();
    for _ in 0..skip {
        file.read_u8().unwrap();
    }
    for _ in 0..shader_data.num_string {
        shader_data.string_property.push(XacStringProperties {
            name_properties: xac_read_string(file),
            value: xac_read_string(file),
        });
    }
    xac.shader_material.push(shader_data);
    xac
}

fn read_mesh<'a, R: Read + Seek>(file: &'a mut R, xac: &'a mut XacFile) -> &'a mut XacFile {
    let mut mesh_info = XacMesh::default();
    mesh_info.node_id = file.read_u32::<LittleEndian>().unwrap();
    mesh_info.num_influence_ranges = file.read_u32::<LittleEndian>().unwrap();
    mesh_info.num_vertices = file.read_u32::<LittleEndian>().unwrap();
    mesh_info.num_indices = file.read_u32::<LittleEndian>().unwrap();
    mesh_info.num_sub_meshes = file.read_u32::<LittleEndian>().unwrap();
    mesh_info.num_attribute_layer = file.read_u32::<LittleEndian>().unwrap();
    mesh_info.collision_mesh = xac_read_boolean(file);
    file.read_u8().unwrap(); //Padding
    file.read_u8().unwrap(); //Padding
    file.read_u8().unwrap(); //Padding

    for _ in 0..mesh_info.num_attribute_layer {
        let mut vertices_attribute = mesh_info.vertices_attribute;
        vertices_attribute.type_id = file.read_u32::<LittleEndian>().unwrap();
        vertices_attribute.attribute_size = file.read_u32::<LittleEndian>().unwrap();
        vertices_attribute.keep_original = xac_read_boolean(file);
        vertices_attribute.scale_factor = xac_read_boolean(file);
        file.read_u8().unwrap(); //Padding
        file.read_u8().unwrap(); //Padding

        if vertices_attribute.type_id == XacPositionId as u32 {
            for _ in 0..mesh_info.num_vertices {
                vertices_attribute
                    .vertex_positions
                    .push(xac_read_vec3d(file));
            }
        }

        if vertices_attribute.type_id == XacNormalId as u32 {
            for _ in 0..mesh_info.num_vertices {
                vertices_attribute.vertex_normals.push(xac_read_vec3d(file))
            }
        }
        if vertices_attribute.type_id == XacTangentId as u32 {
            if vertices_attribute.vertex_tangents.is_empty() {
                for _ in 0..mesh_info.num_vertices {
                    vertices_attribute
                        .vertex_tangents
                        .push(xac_read_vec4d(file));
                }
            } else if vertices_attribute.vertex_bi_tangents.is_empty() {
                for _ in 0..mesh_info.num_vertices {
                    vertices_attribute
                        .vertex_bi_tangents
                        .push(xac_read_vec4d(file));
                }
            }
        }
        if vertices_attribute.type_id == XacUVCoordId as u32 {
            for _ in 0..mesh_info.num_vertices {
                vertices_attribute.vertex_uvs.push(xac_read_vec2d(file));
            }
        }

        if vertices_attribute.type_id == XacColor32Id as u32 {
            for _ in 0..mesh_info.num_vertices {
                vertices_attribute
                    .vertex_colors_32
                    .push(xac_read_color8(file));
            }
        }
        if vertices_attribute.type_id == XacInfluenceRangeId as u32 {
            for _ in 0..mesh_info.num_vertices {
                vertices_attribute
                    .vertex_influences
                    .push(file.read_u32::<LittleEndian>().unwrap());
            }
        }
        if vertices_attribute.type_id == XacColor128Id as u32 {
            for _ in 0..mesh_info.num_vertices {
                vertices_attribute
                    .vertex_colors_128
                    .push(xac_read_vec3d(file));
            }
        }
        mesh_info.vertices_attribute = vertices_attribute;
    }

    for _ in 0..mesh_info.num_sub_meshes {
        let mut sub_mesh = XacSubMesh::default();
        sub_mesh.num_indices = file.read_u32::<LittleEndian>().unwrap();
        sub_mesh.num_vertices = file.read_u32::<LittleEndian>().unwrap();
        sub_mesh.material_id = file.read_u32::<LittleEndian>().unwrap();
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
    xac.mesh_data.push(mesh_info);
    xac
}

fn read_skinning<'a, R: Read + Seek>(file: &'a mut R, xac: &'a mut XacFile) -> &'a mut XacFile {
    xac.skinning.node_id = file.read_u32::<LittleEndian>().unwrap();
    xac.skinning.num_local_bones = file.read_u32::<LittleEndian>().unwrap();
    xac.skinning.num_influences = file.read_u32::<LittleEndian>().unwrap();
    xac.skinning.collision_mesh = xac_read_boolean(file);
    file.read_u8().unwrap(); //Padding
    file.read_u8().unwrap(); //Padding
    file.read_u8().unwrap(); //Padding

    for _ in 0..xac.skinning.num_influences {
        xac.skinning.influence_data.push(XacInfluenceData {
            weight: file.read_f32::<LittleEndian>().unwrap(),
            bone_id: file.read_u32::<LittleEndian>().unwrap(),
        });
    }
    for _ in 0..xac.mesh_data[xac.mesh_data.len() - 1usize].num_influence_ranges {
        xac.skinning.influence_range.push(XacInfluenceRange {
            first_influence_index: file.read_u32::<LittleEndian>().unwrap(),
            num_influences: file.read_u32::<LittleEndian>().unwrap(),
        });
    }
    xac
}
