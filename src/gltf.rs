use mesh_tools::GltfBuilder;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, Cursor, Result, Seek, Write};

fn write_u32_le(cursor: &mut Cursor<&mut Vec<u8>>, value: u32) -> io::Result<()> {
    cursor.write_all(&value.to_le_bytes())
}

/// Export the glTF as a GLB file (in-memory bytes) without `byteorder`
pub fn export_glb_bytes(builder: &GltfBuilder) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);

    // GLB header
    cursor.write_all(b"glTF")?;
    write_u32_le(&mut cursor, 2)?; // version
    let length_pos = cursor.position();
    write_u32_le(&mut cursor, 0)?; // placeholder for total length

    // JSON chunk
    let json = serde_json::to_string(&builder.gltf)?;
    let json_len = json.len();
    let json_pad = (4 - (json_len % 4)) % 4;

    write_u32_le(&mut cursor, (json_len + json_pad) as u32)?; // chunk length
    write_u32_le(&mut cursor, 0x4E4F534A)?; // "JSON"
    cursor.write_all(json.as_bytes())?;
    for _ in 0..json_pad {
        cursor.write_all(&[0x20])?; // pad with space
    }

    // BIN chunk
    if !builder.buffer_data.is_empty() {
        let bin_len = builder.buffer_data.len();
        let bin_pad = (4 - (bin_len % 4)) % 4;

        write_u32_le(&mut cursor, (bin_len + bin_pad) as u32)?;
        write_u32_le(&mut cursor, 0x004E4942)?; // "BIN"
        cursor.write_all(&builder.buffer_data)?;
        for _ in 0..bin_pad {
            cursor.write_all(&[0])?;
        }
    }

    // Update total length
    let total_len = cursor.position() as u32;
    cursor.seek(io::SeekFrom::Start(length_pos))?;
    write_u32_le(&mut cursor, total_len)?;

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use crate::gltf::export_glb_bytes;
    use mesh_tools::GltfBuilder;

    #[test]
    fn export_into_glb_bytes() {
        let mut builder = GltfBuilder::new();

        // Cube A
        let mesh_a = builder.create_box(1.0);
        let node_a = builder.add_node(
            Some("CubeA".to_string()),
            Some(mesh_a),
            Some([0.0, 0.0, 0.0]),
            None,
            None,
        );

        // Cube B
        let mesh_b = builder.create_box(2.0);
        let rot_b = [
            0.0,
            (45f32.to_radians() / 2.0).sin(),
            0.0,
            (45f32.to_radians() / 2.0).cos(),
        ];
        let node_b = builder.add_node(
            Some("CubeB".to_string()),
            Some(mesh_b),
            Some([3.0, 0.0, 0.0]),
            Some(rot_b),
            Some([1.2, 1.0, 0.8]),
        );

        // Cube C
        let mesh_c = builder.create_box(0.5);
        let rot_c = [
            0.0,
            0.0,
            (30f32.to_radians() / 2.0).sin(),
            (30f32.to_radians() / 2.0).cos(),
        ];
        let node_c = builder.add_node(
            Some("CubeC".to_string()),
            Some(mesh_c),
            Some([-1.5, 1.0, -0.5]),
            Some(rot_c),
            None,
        );

        builder.add_scene(
            Some("ThreeCubesScene".to_string()),
            Some(vec![node_a, node_b, node_c]),
        );

        let glb = export_glb_bytes(&builder).expect("GLB export failed");
        println!("GLB size = {} bytes", glb.len());
    }
}
