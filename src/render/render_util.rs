use crate::xac::xac_struct::XacFile;

use super::render_struct::BevyMesh;

pub(crate) fn xac_to_mesh(xac_file: XacFile) -> Vec<Vec<BevyMesh>> {
    let mut nodemeshes: Vec<Vec<BevyMesh>> = Vec::new();
    let mut materials: Vec<String> = Vec::new();

    for shader in &xac_file.shader_material {
        for name_shader in &shader.string_property {
            materials.push(name_shader.value.clone());
        }
    }
    let num_material = materials.len() as u32;

    for mesh_node in xac_file.mesh_data {
        let mut meshes = Vec::new();
        let mut number: usize = 0;

        for sub_mesh in mesh_node.sub_mesh {
            let mut submeshes = BevyMesh::default();
            if sub_mesh.material_id >= 1 {
                submeshes.name_texture = materials
                    .get((sub_mesh.material_id % num_material) as usize)
                    .unwrap()
                    .to_string();
                for num in number + 0..number + sub_mesh.num_vertices as usize {
                    if mesh_node.vertices_attribute.vertex_positions.len() != 0 {
                        submeshes.positions.push([
                            mesh_node
                                .vertices_attribute
                                .vertex_positions
                                .get(num)
                                .unwrap()
                                .x,
                            mesh_node
                                .vertices_attribute
                                .vertex_positions
                                .get(num)
                                .unwrap()
                                .y,
                            mesh_node
                                .vertices_attribute
                                .vertex_positions
                                .get(num)
                                .unwrap()
                                .z,
                        ]);
                    }
                    if mesh_node.vertices_attribute.vertex_normals.len() != 0 {
                        submeshes.normals.push([
                            mesh_node
                                .vertices_attribute
                                .vertex_normals
                                .get(num)
                                .unwrap()
                                .x,
                            mesh_node
                                .vertices_attribute
                                .vertex_normals
                                .get(num)
                                .unwrap()
                                .y,
                            mesh_node
                                .vertices_attribute
                                .vertex_normals
                                .get(num)
                                .unwrap()
                                .z,
                        ]);
                    }
                    if mesh_node.vertices_attribute.vertex_tangents.len() != 0 {
                        submeshes.tangents.push([
                            mesh_node
                                .vertices_attribute
                                .vertex_tangents
                                .get(num)
                                .unwrap()
                                .x,
                            mesh_node
                                .vertices_attribute
                                .vertex_tangents
                                .get(num)
                                .unwrap()
                                .y,
                            mesh_node
                                .vertices_attribute
                                .vertex_tangents
                                .get(num)
                                .unwrap()
                                .z,
                            mesh_node
                                .vertices_attribute
                                .vertex_tangents
                                .get(num)
                                .unwrap()
                                .w,
                        ]);
                    }
                    if mesh_node.vertices_attribute.vertex_bi_tangents.len() != 0 {
                        submeshes.bi_tangents.push([
                            mesh_node
                                .vertices_attribute
                                .vertex_bi_tangents
                                .get(num)
                                .unwrap()
                                .x,
                            mesh_node
                                .vertices_attribute
                                .vertex_bi_tangents
                                .get(num)
                                .unwrap()
                                .y,
                            mesh_node
                                .vertices_attribute
                                .vertex_bi_tangents
                                .get(num)
                                .unwrap()
                                .z,
                            mesh_node
                                .vertices_attribute
                                .vertex_bi_tangents
                                .get(num)
                                .unwrap()
                                .w,
                        ]);
                    }

                    number = number + 1;
                }
                for num_uvs in 0..sub_mesh.num_vertices as usize {
                    submeshes.uv_set.push([
                        mesh_node
                            .vertices_attribute
                            .vertex_uvs
                            .get(num_uvs)
                            .unwrap()
                            .x,
                        mesh_node
                            .vertices_attribute
                            .vertex_uvs
                            .get(num_uvs)
                            .unwrap()
                            .y,
                    ]);
                }
                submeshes.mesh_indices = sub_mesh.relative_indices;
            }
            if submeshes.positions.len() != 0 {
                meshes.push(submeshes);
            }
        }
        if meshes.len() != 0 {
            nodemeshes.push(meshes);
        }
    }

    nodemeshes
}
