use serde::{Deserialize, Serialize};

use crate::xac::{XACAttribute, XACChunk, XACChunkData};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RGBAColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SubMesh {
    pub name: String,
    pub textures: String,
    pub positions: Vec<Vector3>,
    pub normals: Vec<Vector3>,
    pub tangents: Vec<Vector4>,
    pub bitangents: Vec<Vector3>,
    pub uvcoords: Vec<Vector2>,
    pub colors32: Vec<u32>,
    pub colors128: Vec<RGBAColor>,
    pub original_vertex_numbers: Vec<u32>,
    pub indices: Vec<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Model {
    pub name: String,
    pub submeshes: Vec<SubMesh>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct SceneNode {
    pub name: String,
    pub transform: Option<[[f32; 4]; 4]>, // 4x4 transform matrix
    pub model: Option<Model>,             // optional model at this node
    pub children: Vec<SceneNode>,         // nested nodes
}

// Root scene can either be a single node or multiple nodes
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Scene {
    pub root_nodes: Vec<SceneNode>,
}

// --- XAC to Scene converter ---
impl Scene {
    pub fn from_xac_root(xac: &crate::xac::XACRoot) -> Self {
        let mut root_nodes = Vec::new();

        for entry in &xac.chunks {
            if entry.chunk.chunk_id != XACChunk::XACChunkMesh as u32 {
                continue;
            }

            let model = match entry.chunk.version {
                1 => {
                    if let XACChunkData::XACMesh(mesh) = &entry.chunk_data {
                        let submeshes = Scene::parse_vertex_data(
                            &mesh.vertex_attribute_layer,
                            &mesh.sub_meshes,
                            xac.get_texture_names(),
                        );
                        Some(Model {
                            name: String::new(),
                            submeshes,
                        })
                    } else {
                        None
                    }
                }
                2 => {
                    if let XACChunkData::XACMesh2(mesh) = &entry.chunk_data {
                        let submeshes = Scene::parse_vertex_data(
                            &mesh.vertex_attribute_layer,
                            &mesh.sub_meshes,
                            xac.get_texture_names(),
                        );
                        Some(Model {
                            name: String::new(),
                            submeshes,
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            };

            // Create a SceneNode to hold the model
            if let Some(model) = model {
                let node = SceneNode {
                    name: model.name.clone(),
                    transform: None, // you can fill this if you have node transform
                    model: Some(model),
                    children: Vec::new(),
                };

                root_nodes.push(node);
            }
        }

        Scene { root_nodes }
    }

    fn parse_vertex_data(
        layers: &[crate::xac::XACVertexAttributeLayer],
        submeshes: &[crate::xac::XACSubMesh],
        textures_data: Vec<String>,
    ) -> Vec<SubMesh> {
        // Helper to find layer by attribute type
        let find_layer = |attr_id| layers.iter().find(|l| l.layer_type_id == attr_id);

        let positions_layer = find_layer(crate::xac::XACAttribute::AttribPositions as u32);
        let normals_layer = find_layer(crate::xac::XACAttribute::AttribNormals as u32);
        let tangents_layer = find_layer(crate::xac::XACAttribute::AttribTangents as u32);
        let uvs_layer = find_layer(crate::xac::XACAttribute::AttribUvcoords as u32);
        let colors32_layer = find_layer(crate::xac::XACAttribute::AttribColors32 as u32);
        let colors128_layer = find_layer(crate::xac::XACAttribute::AttribColors128 as u32);
        let original_vertex_numbers_layer =
            find_layer(crate::xac::XACAttribute::AttribOrgvtxnumbers as u32);
        let bitangents_layer = find_layer(crate::xac::XACAttribute::AttribBitangents as u32);

        let positions_data = positions_layer.map(|l| &l.mesh_data);
        let normals_data = normals_layer.map(|l| &l.mesh_data);
        let tangents_data = tangents_layer.map(|l| &l.mesh_data);
        let uvs_data = uvs_layer.map(|l| &l.mesh_data);
        let colors32_data = colors32_layer.map(|l| &l.mesh_data);
        let colors128_data = colors128_layer.map(|l| &l.mesh_data);
        let original_vertex_numbers_data = original_vertex_numbers_layer.map(|l| &l.mesh_data);
        let bitangents_data = bitangents_layer.map(|l| &l.mesh_data);

        let mut vertex_offset: usize = 0;
        let mut parsed_submeshes = Vec::new();

        for submesh in submeshes {
            let mut s = SubMesh::default();

            // Assign material/texture
            s.textures = if submesh.material_index != 0 {
                textures_data
                    .get(submesh.material_index as usize)
                    .cloned()
                    .unwrap_or_default()
            } else {
                String::new()
            };

            for index_indices in 0..submesh.num_indices {
                s.indices.push(submesh.indices[index_indices as usize]);
            }
            for v in 0..submesh.num_verts {
                let actual_index = vertex_offset + v as usize;

                // Positions
                if let Some(data) = positions_data {
                    let offset = actual_index * 12;
                    if offset + 12 <= data.len() {
                        let px = f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
                        let py =
                            f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
                        let pz =
                            f32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap());
                        s.positions.push(Vector3 {
                            x: -px,
                            y: py,
                            z: pz,
                        });
                    }
                }

                // Normals
                if let Some(data) = normals_data {
                    let offset = actual_index * 12;
                    if offset + 12 <= data.len() {
                        let nx = f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
                        let ny =
                            f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
                        let nz =
                            f32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap());
                        s.normals.push(Vector3 {
                            x: -nx,
                            y: ny,
                            z: nz,
                        });
                    }
                }

                // Tangents
                if let Some(data) = tangents_data {
                    let offset = actual_index * 16;
                    if offset + 16 <= data.len() {
                        let tx = f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
                        let ty =
                            f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
                        let tz =
                            f32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap());
                        let tw =
                            f32::from_le_bytes(data[offset + 12..offset + 16].try_into().unwrap());
                        s.tangents.push(Vector4 {
                            x: tx,
                            y: ty,
                            z: tz,
                            w: tw,
                        });
                    }
                }

                // UVs
                if let Some(data) = uvs_data {
                    let offset = actual_index * 8;
                    if offset + 8 <= data.len() {
                        let u = f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
                        let v =
                            f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
                        s.uvcoords.push(Vector2 { x: u, y: v });
                    }
                }

                // Colors32
                if let Some(data) = colors32_data {
                    let offset = actual_index * 4;
                    if offset + 4 <= data.len() {
                        s.colors32.push(u32::from_le_bytes(
                            data[offset..offset + 4].try_into().unwrap(),
                        ));
                    }
                }

                // Colors128
                if let Some(data) = colors128_data {
                    let offset = actual_index * 16;
                    if offset + 16 <= data.len() {
                        let r = f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
                        let g =
                            f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
                        let b =
                            f32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap());
                        let a =
                            f32::from_le_bytes(data[offset + 12..offset + 16].try_into().unwrap());
                        s.colors128.push(RGBAColor { r, g, b, a });
                    }
                }

                // Bitangents
                if let Some(data) = bitangents_data {
                    let offset = actual_index * 12;
                    if offset + 12 <= data.len() {
                        let bx = f32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
                        let by =
                            f32::from_le_bytes(data[offset + 4..offset + 8].try_into().unwrap());
                        let bz =
                            f32::from_le_bytes(data[offset + 8..offset + 12].try_into().unwrap());
                        s.bitangents.push(Vector3 {
                            x: bx,
                            y: by,
                            z: bz,
                        });
                    }
                }
            }

            vertex_offset += submesh.num_verts as usize;
            parsed_submeshes.push(s);
        }

        parsed_submeshes
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_scene_from_xac_file() -> io::Result<()> {
        // Path to a test XAC file
        let path = "tests/npc_lecifer_set.xac";

        // Load XACRoot from file
        let xac_root = crate::xac::XACRoot::from_file(path)?;

        // Convert to Scene
        let scene = Scene::from_xac_root(&xac_root);

        // Debug print
        println!("Scene root nodes: {:?}", scene.root_nodes);

        // Assert that the scene is not empty
        assert!(
            !scene.root_nodes.is_empty(),
            "Scene should contain at least one root node"
        );

        // Optionally check if the first root node has a model
        if let Some(first_node) = scene.root_nodes.first() {
            assert!(
                first_node.model.is_some(),
                "First root node should have a model"
            );
        }

        Ok(())
    }

    #[test]
    fn test_scene_from_xac_memory() -> io::Result<()> {
        // Load file into memory
        let data = std::fs::read("tests/archer_m_falconer01.xac")?;

        // Parse XACRoot from bytes
        let xac_root = crate::xac::XACRoot::from_bytes(&data)?;

        println!("Textures Name: {:#?}", xac_root.get_texture_names());

        // Convert to Scene
        let scene = Scene::from_xac_root(&xac_root);

        // Debug print
        // println!("Scene root nodes: {:?}", scene.root_nodes);

        // Basic assertion
        assert!(
            !scene.root_nodes.is_empty(),
            "Scene should contain at least one root node"
        );

        Ok(())
    }

    #[test]
    fn print_default_empty_scene() {
        // Create a default empty scene
        let scene = Scene::default();

        // Print it
        println!("Default empty scene: {:#?}", scene);

        // Assert it's empty
        assert!(
            scene.root_nodes.is_empty(),
            "Default scene should have no root nodes"
        );
    }
}
