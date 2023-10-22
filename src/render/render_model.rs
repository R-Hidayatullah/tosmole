//! Load a cubemap texture onto a cube like a skybox and cycle through different compressed texture formats

use bevy::render::render_resource::PrimitiveTopology;
use elementtree::Element;
use std::{f32::consts::PI, io::Read};

use std::fs::File;

use bevy::{
    asset::LoadState,
    core_pipeline::Skybox,
    input::mouse::MouseMotion,
    prelude::*,
    render::{
        render_resource::{TextureViewDescriptor, TextureViewDimension},
        renderer::RenderDevice,
        texture::CompressedImageFormats,
    },
};

use super::{render_struct::BevyMesh, render_util::xac_to_mesh};
use crate::ipf::{
    ipf_parser::{ipf_get_data, ipf_get_data_image, ipf_get_data_xac, ipf_parse},
    ipf_struct::IpfFile,
};
use crate::xac::xac_parser::xac_parse;
use crate::xml::map_struct::{Model, World};

const CUBEMAPS: &[(&str, CompressedImageFormats)] = &[(
    "textures/Ryfjallet_cubemap.png",
    CompressedImageFormats::NONE,
)];

pub(crate) fn render() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (asset_loaded, camera_controller, animate_light_direction),
        )
        .run();
}
#[derive(Resource)]
struct Cubemap {
    is_loaded: bool,
    index: usize,
    image_handle: Handle<Image>,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /*

    let args: Vec<String> = std::env::args().collect();
    let args_count = std::env::args().count();
    if args_count == 1 {
        println!("Usage :\n1. tosmole example.ipf\n2. tosmole example.ipf index_number");
    } else if args_count == 2 {
        println!("Parse first index.");
        let path_file = &args[1];
        let mut location = BufReader::new(File::open(path_file).unwrap());
        let ipf_data = ipf_parse(&mut location);
        ipf_get_data(&mut location, &ipf_data, 0);
        println!("\nFinish parsing first index.");
    } else if args_count >= 3 {
        let path_file = &args[1];
        let index_list = &args[2];
    let index_list = 0;
    println!("Parse index : {}", index_list);
    let mut location = BufReader::new(File::open(path_file).unwrap());
    let ipf_data = ipf_parse(&mut location);
    let mut berkas = ipf_get_data_xac(
        &mut location,
        &ipf_data,
        index_list.to_string().parse::<usize>().unwrap(),
    );
    if berkas.is_some() {
        let mut ipf_data = IpfFile::default();
        let mut  location=BufReader::new(File::open("C:\\Program Files (x86)\\Steam\\steamapps\\common\\TreeOfSavior\\data\\bg_texture.ipf").unwrap());

        if ipf_data
            .file_table
            .get(0)
            .unwrap()
            .container_name
            .contains("bg_")
        {
            location=BufReader::new(File::open("C:\\Program Files (x86)\\Steam\\steamapps\\common\\TreeOfSavior\\data\\bg_texture.ipf").unwrap());
            ipf_data = ipf_parse(&mut location);
        } else if ipf_data
            .file_table
            .get(0)
            .unwrap()
            .container_name
            .contains("char_")
        {
            location=BufReader::new(File::open("C:\\Program Files (x86)\\Steam\\steamapps\\common\\TreeOfSavior\\data\\char_texture.ipf").unwrap());
            ipf_data = ipf_parse(&mut location);
        } else if ipf_data
            .file_table
            .get(0)
            .unwrap()
            .container_name
            .contains("item_")
        {
            location=BufReader::new(File::open("C:\\Program Files (x86)\\Steam\\steamapps\\common\\TreeOfSavior\\data\\item_texture.ipf").unwrap());
            ipf_data = ipf_parse(&mut location);
        }

        let mut texture_data: hashbrown::HashMap<String, crate::ipf::ipf_struct::IPFFileTable> =
            hashbrown::HashMap::new();

        for file_data in ipf_data.file_table {
            texture_data.insert(file_data.filename.clone(), file_data);
        }
        println!("Hash length : {}", texture_data.len());*/

    let path_file = std::fs::read_dir(
        "C:\\Users\\Ridwan Hidayatullah\\Music\\c_klaipeda\\bg_hi.ipf\\city\\c_orsha",
    );

    let path_xml =
        "C:\\Users\\Ridwan Hidayatullah\\Music\\c_klaipeda\\bg.ipf\\hi_entity\\c_orsha.3dworld";
    let mut location = std::fs::File::open(path_xml).unwrap();
    let mut contents = String::new();
    location
        .read_to_string(&mut contents)
        .expect("Unable to read string");

    let node_xml = Element::from_reader(contents.as_bytes()).unwrap();
    let mut desc_map = World::default();

    for child in node_xml.find_all("Model") {
        let mut model_data = Model::default();
        model_data.filename = child.get_attr("File").unwrap().to_string();
        model_data.model_name = child.get_attr("Model").unwrap().to_string();
        let mut position = child.get_attr("pos").unwrap().split_ascii_whitespace();
        model_data.position = [
            position.next().unwrap().parse::<f32>().unwrap(),
            position.next().unwrap().parse::<f32>().unwrap(),
            position.next().unwrap().parse::<f32>().unwrap(),
        ];
        let mut rotation = child.get_attr("rot").unwrap().split_ascii_whitespace();

        model_data.rotation = [
            rotation.next().unwrap().parse::<f32>().unwrap(),
            rotation.next().unwrap().parse::<f32>().unwrap(),
            rotation.next().unwrap().parse::<f32>().unwrap(),
            rotation.next().unwrap().parse::<f32>().unwrap(),
        ];
        let mut scale = child.get_attr("scale").unwrap().split_ascii_whitespace();
        model_data.scale = [
            scale.next().unwrap().parse::<f32>().unwrap(),
            scale.next().unwrap().parse::<f32>().unwrap(),
            scale.next().unwrap().parse::<f32>().unwrap(),
        ];

        desc_map.model.push(model_data);
    }
    // directional 'sun' light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 32000.0,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 2.0, 0.0)
            .with_rotation(Quat::from_rotation_x(-PI / 4.)),
        ..default()
    });

    let skybox_handle = asset_server.load(CUBEMAPS[0].0);
    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(100.0, 100.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraController::default(),
        Skybox(skybox_handle.clone()),
    ));

    // ambient light
    // NOTE: The ambient light is used to scale how bright the environment map is so with a bright
    // environment map, use an appropriate color and brightness to match
    commands.insert_resource(AmbientLight {
        color: Color::rgb_u8(210, 220, 240),
        brightness: 1.0,
    });

    commands.insert_resource(Cubemap {
        is_loaded: false,
        index: 0,
        image_handle: skybox_handle,
    });

    for single_file in path_file.unwrap() {
        let path_data = single_file.unwrap().path();
        let data_path = path_data.to_str().unwrap();
        if data_path.contains("xac") {
            let mut model_data = Model::default();
            for index in 0..desc_map.model.len() {
                if data_path.contains(desc_map.model.get(index).unwrap().filename.as_str()) {
                    model_data = desc_map.model.get(index).unwrap().clone();
                }
            }

            let mut file_loc = File::open(path_data).unwrap();
            let mut xac_file = xac_parse(&mut file_loc);
            let mut bevy_mesh = xac_to_mesh(xac_file);

            for bev in bevy_mesh {
                for bav in bev {
                    let cube_mesh_handle: Handle<Mesh> = meshes.add(create_xac_mesh(bav));
                    /*
                                    let image_data = ipf_get_data_image(
                                        &mut location,
                                        &ipf_data,
                                        texture_data.get(&bav.name_texture).unwrap().idx,
                                    )
                                    .unwrap();
                    */

                    commands.spawn((PbrBundle {
                        mesh: cube_mesh_handle,
                        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                        transform: Transform {
                            translation: Vec3 {
                                x: model_data.position[0],
                                y: model_data.position[1],
                                z: model_data.position[2],
                            },
                            rotation: Quat::from_array(model_data.rotation),
                            scale: Vec3 {
                                x: model_data.scale[0],
                                y: model_data.scale[0],
                                z: model_data.scale[0],
                            },
                        },
                        ..default()
                    },));
                }
            }
        }
    }
}

fn create_xac_mesh(bav: BevyMesh) -> Mesh {
    let mut mesh_bevy = Mesh::new(PrimitiveTopology::TriangleList);
    if bav.positions.len() != 0 {
        mesh_bevy.insert_attribute(Mesh::ATTRIBUTE_POSITION, bav.positions);
    }
    if bav.normals.len() != 0 {
        mesh_bevy.insert_attribute(Mesh::ATTRIBUTE_NORMAL, bav.normals);
    }
    if bav.tangents.len() != 0 {
        mesh_bevy.insert_attribute(Mesh::ATTRIBUTE_TANGENT, bav.tangents);
    }
    if bav.bi_tangents.len() != 0 {} //Bevy doesnt supported bitangent yet
    if bav.uv_set.len() != 0 {
        mesh_bevy.insert_attribute(Mesh::ATTRIBUTE_UV_0, bav.uv_set);
    }
    if bav.mesh_indices.len() != 0 {
        mesh_bevy.set_indices(Some(bevy::render::mesh::Indices::U32(bav.mesh_indices)));
    }
    mesh_bevy
}

const CUBEMAP_SWAP_DELAY: f32 = 3.0;

fn asset_loaded(
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    mut cubemap: ResMut<Cubemap>,
    mut skyboxes: Query<&mut Skybox>,
) {
    if !cubemap.is_loaded
        && asset_server.get_load_state(cubemap.image_handle.clone_weak()) == LoadState::Loaded
    {
        info!("Swapping to {}...", CUBEMAPS[cubemap.index].0);
        let image = images.get_mut(&cubemap.image_handle).unwrap();
        // NOTE: PNGs do not have any metadata that could indicate they contain a cubemap texture,
        // so they appear as one texture. The following code reconfigures the texture as necessary.
        if image.texture_descriptor.array_layer_count() == 1 {
            image.reinterpret_stacked_2d_as_array(
                image.texture_descriptor.size.height / image.texture_descriptor.size.width,
            );
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
        }

        for mut skybox in &mut skyboxes {
            skybox.0 = cubemap.image_handle.clone();
        }

        cubemap.is_loaded = true;
    }
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() * 0.5);
    }
}

#[derive(Component)]
pub struct CameraController {
    pub enabled: bool,
    pub initialized: bool,
    pub sensitivity: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_run: KeyCode,
    pub mouse_key_enable_mouse: MouseButton,
    pub keyboard_key_enable_mouse: KeyCode,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            initialized: false,
            sensitivity: 0.5,
            key_forward: KeyCode::W,
            key_back: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::E,
            key_down: KeyCode::Q,
            key_run: KeyCode::ShiftLeft,
            mouse_key_enable_mouse: MouseButton::Left,
            keyboard_key_enable_mouse: KeyCode::M,
            walk_speed: 80.0,
            run_speed: 200.0,
            friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
        }
    }
}

pub fn camera_controller(
    time: Res<Time>,
    mut mouse_events: EventReader<MouseMotion>,
    mouse_button_input: Res<Input<MouseButton>>,
    key_input: Res<Input<KeyCode>>,
    mut move_toggled: Local<bool>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
    let dt = time.delta_seconds();

    if let Ok((mut transform, mut options)) = query.get_single_mut() {
        if !options.initialized {
            let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
            options.yaw = yaw;
            options.pitch = pitch;
            options.initialized = true;
        }
        if !options.enabled {
            return;
        }

        // Handle key input
        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(options.key_forward) {
            axis_input.z += 1.0;
        }
        if key_input.pressed(options.key_back) {
            axis_input.z -= 1.0;
        }
        if key_input.pressed(options.key_right) {
            axis_input.x += 1.0;
        }
        if key_input.pressed(options.key_left) {
            axis_input.x -= 1.0;
        }
        if key_input.pressed(options.key_up) {
            axis_input.y += 1.0;
        }
        if key_input.pressed(options.key_down) {
            axis_input.y -= 1.0;
        }
        if key_input.just_pressed(options.keyboard_key_enable_mouse) {
            *move_toggled = !*move_toggled;
        }

        // Apply movement update
        if axis_input != Vec3::ZERO {
            let max_speed = if key_input.pressed(options.key_run) {
                options.run_speed
            } else {
                options.walk_speed
            };
            options.velocity = axis_input.normalize() * max_speed;
        } else {
            let friction = options.friction.clamp(0.0, 1.0);
            options.velocity *= 1.0 - friction;
            if options.velocity.length_squared() < 1e-6 {
                options.velocity = Vec3::ZERO;
            }
        }
        let forward = transform.forward();
        let right = transform.right();
        transform.translation += options.velocity.x * dt * right
            + options.velocity.y * dt * Vec3::Y
            + options.velocity.z * dt * forward;

        // Handle mouse input
        let mut mouse_delta = Vec2::ZERO;
        if mouse_button_input.pressed(options.mouse_key_enable_mouse) || *move_toggled {
            for mouse_event in mouse_events.iter() {
                mouse_delta += mouse_event.delta;
            }
        }

        if mouse_delta != Vec2::ZERO {
            // Apply look update
            options.pitch = (options.pitch - mouse_delta.y * 0.5 * options.sensitivity * dt)
                .clamp(-PI / 2., PI / 2.);
            options.yaw -= mouse_delta.x * options.sensitivity * dt;
            transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, options.yaw, options.pitch);
        }
    }
}
