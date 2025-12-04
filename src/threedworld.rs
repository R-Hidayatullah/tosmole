use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct World {
    #[serde(rename = "ModelDir", default)]
    pub model_dirs: Vec<ModelDir>,
    #[serde(rename = "TexDir", default)]
    pub tex_dirs: Vec<TexDir>,
    #[serde(rename = "SubTexDir", default)]
    pub sub_tex_dirs: Vec<SubTexDir>,
    #[serde(rename = "AnimationDir", default)]
    pub animation_dirs: Vec<AnimationDir>,
    #[serde(rename = "ShaTexDir", default)]
    pub sha_tex_dirs: Vec<ShaTexDir>,
    #[serde(rename = "LightMap", default)]
    pub light_maps: Vec<LightMap>,
    #[serde(rename = "StandOnPos")]
    pub stand_on_pos: Option<Pos>,
    #[serde(rename = "Model", default)]
    pub models: Vec<Model>,
}

// ------------------- Directories -------------------
#[derive(Debug, Deserialize, Default)]
pub struct ModelDir {
    #[serde(rename = "@IpfName", default)]
    pub ipf_name: String,
    #[serde(rename = "@Path", default)]
    pub path: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct TexDir {
    #[serde(rename = "@IpfName", default)]
    pub ipf_name: String,
    #[serde(rename = "@Path", default)]
    pub path: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct SubTexDir {
    #[serde(rename = "@IpfName", default)]
    pub ipf_name: String,
    #[serde(rename = "@Path", default)]
    pub path: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct AnimationDir {
    #[serde(rename = "@IpfName", default)]
    pub ipf_name: String,
    #[serde(rename = "@Path", default)]
    pub path: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct ShaTexDir {
    #[serde(rename = "@IpfName", default)]
    pub ipf_name: String,
    #[serde(rename = "@Path", default)]
    pub path: String,
}

// ------------------- LightMap & Pos -------------------
#[derive(Debug, Deserialize, Default)]
pub struct LightMap {
    #[serde(rename = "@File", default)]
    pub file: String,
    #[serde(rename = "@Length", default)]
    pub length: Option<String>,
    #[serde(rename = "@Offset", default)]
    pub offset: Option<String>,
    #[serde(rename = "@Size", default)]
    pub size: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct Pos {
    #[serde(rename = "@pos", default)]
    pub pos: String,
}

// ------------------- Models -------------------
#[derive(Debug, Deserialize, Default)]
pub struct Model {
    #[serde(rename = "@File", default)]
    pub file: String,
    #[serde(rename = "@Model", default)]
    pub model: String,
    #[serde(rename = "@ShadowMap", default)]
    pub shadow_map: Option<String>,
    #[serde(rename = "@pos", default)]
    pub pos: Option<String>,
    #[serde(rename = "@rot", default)]
    pub rot: Option<String>,
    #[serde(rename = "@scale", default)]
    pub scale: Option<String>,
}

// ------------------- Tests -------------------
#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;
    use std::fs;
    use std::path::PathBuf;

    fn get_test_file_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push(filename);
        path
    }

    fn parse_world(filename: &str) -> World {
        let path = get_test_file_path(filename);
        let xml =
            fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read file {:?}", path));

        from_str(&xml).unwrap_or_else(|e| panic!("Failed to parse XML {}: {:?}", filename, e))
    }

    #[test]
    fn test_parse_barrack_world() {
        let world = parse_world("barrack.3dworld");

        assert!(!world.model_dirs.is_empty(), "No model directories found");
        assert!(!world.tex_dirs.is_empty(), "No texture directories found");
        assert!(!world.models.is_empty(), "No models found");

        // Check first model has a file
        let first_model = &world.models[0];
        assert!(
            !first_model.file.is_empty(),
            "First model has empty file attribute"
        );

        println!("Parsed barrack.3dworld successfully: {:#?}", world.models);
    }

    #[test]
    fn test_parse_barrack_noble_world() {
        let world = parse_world("barrack_noble.3dworld");

        assert!(!world.model_dirs.is_empty(), "No model directories found");
        assert!(!world.tex_dirs.is_empty(), "No texture directories found");
        assert!(!world.models.is_empty(), "No models found");

        // Check first model has a file
        let first_model = &world.models[0];
        assert!(
            !first_model.file.is_empty(),
            "First model has empty file attribute"
        );

        println!(
            "Parsed barrack_noble.3dworld successfully: {:#?}",
            world.models
        );
    }
}
