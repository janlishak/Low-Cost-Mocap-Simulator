use bevy::{gltf::GltfLoaderSettings, prelude::*, render::RenderPlugin, scene::SceneInstance, window::WindowResolution};
use bevy_flycam::prelude::*;
use bevy_capture::{encoder::frames, CameraTargetHeadless, Capture, CaptureBundle};
use std::{f32::consts::TAU, fs};
use std::sync::atomic::{AtomicU8, Ordering};
use bevy::color::palettes::basic::WHITE;
use bevy::color::palettes::css::ORANGE_RED;
use bevy::core_pipeline::bloom::Bloom;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy_capture::animation::{keyboard_animation_control, setup_animation, setup_scene_once_loaded};
use bevy_capture::encoder::mem_encoder::MyCustomEncoder;
use shared_memory::ShmemConf;

fn main() -> AppExit {
    // Create the captures directory
    fs::create_dir_all("captures/simple").unwrap();
    // let enc = MyCustomEncoder::new("fuck_this_shit", 512, 512);
    let line_buffer_app = LineBuffer::new("bevy_line_input_app", 512);
    let pose_buffer_app = PoseBuffer::new("bevy_pose_input_app", 512);
    // let pose_buffer_app = PoseBuffer::new("bevy_line_input_app", 512);
    

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(RenderPlugin {
            synchronous_pipeline_compilation: true,
            ..default()
        })
        .set(WindowPlugin { 
            primary_window: Some(Window { 
                present_mode: bevy::window::PresentMode::Immediate, // no Vsync
                title: String::from("Mocap Simulator"),
                resolution: WindowResolution::new(512.0, 512.0),
                ..default()
            }), 
            ..default() 
        }),
        bevy_capture::CapturePlugin,
    ));

    app.add_plugins(NoCameraPlayerPlugin);

    app.add_systems(Update, update);
    app.add_systems(Startup, setup_3d);
    app.add_systems(Last, frame_limiter);
    app.add_systems(Update, toggle_ambient_light_system);
    app.add_systems(Update, draw_line_example);

    app.add_systems(Startup, setup_animation)
        .add_systems(Update, setup_scene_once_loaded)
        .add_systems(Update, keyboard_animation_control);

    app.add_systems(Update, toggle_gizmos_on_top);


    app.insert_resource(MovementSettings {
        sensitivity: 0.0001, // default: 0.00012
        speed: 3.0, // default: 12.0
    });
    app.insert_resource(KeyBindings {
        move_ascend: KeyCode::KeyE,
        move_descend: KeyCode::KeyQ,
        ..Default::default()
    });

    app.insert_resource(ClearColor(Color::Hsva(Hsva { hue: 0.0, saturation: 0.0, value: 0.0, alpha: 1.0 })));

    app.insert_resource(line_buffer_app);
    app.insert_resource(pose_buffer_app);
    app.add_systems(Update, draw_lines_from_buffer);
    app.add_systems(Update, write_camera_poses_system);
    app.add_systems(Update, oscillate_camera_index_1);

    // ambient light
    app.insert_resource(AmbientLight {
        color: WHITE.into(),
        brightness: 0.0,
        ..default()
    });

    app.insert_resource(Recording::default());

    app.add_systems(Update, toggle_recording);
    app.add_systems(Update, monitor_recording);

    app.add_systems(Update, wait_for_all_scenes_ready);

    // Run the app
    app.run()
}

#[derive(Component)]
struct Cube;

#[derive(Component)]
struct CameraIndex{
    index: usize
}

#[derive(Resource, Default)]
struct Recording {
    active: bool,
    single: bool,
    stop_next_frame: bool
}

fn toggle_recording(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut recording: ResMut<Recording>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        recording.active = !recording.active;
        info!("Recording toggled: {}", recording.active);
    }

    if keyboard_input.just_pressed(KeyCode::KeyP) {
        recording.active = true;
        recording.single = true;
        info!("Recording Single: {} {}", recording.active, recording.single);
    }
}

fn monitor_recording(recording: Res<Recording>) {
    if recording.active {
        // Replace with capture logic
        // info!("Recording is active.");
    } else {
        // info!("Recording is inactive.");
    }
}

fn setup_3d(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // // circular base
    // commands.spawn((
    //     Mesh3d(meshes.add(Circle::new(4.0))),
    //     MeshMaterial3d(materials.add(Color::WHITE)),
    //     Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    // ));

    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.25, 0.25, 0.25))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(2.071846, 0.31709337, -2.2220883),
        Cube
    ));

    // light
    // commands.spawn((
    //     PointLight {
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     Transform::from_xyz(4.0, 8.0, 4.0),
    // ));

    // camera
    commands
    .spawn((
        FlyCam,
        Camera3d::default(),
        Camera {
            hdr: true, // 1. HDR is required for bloom
            ..default()
        },
        bevy::core_pipeline::tonemapping::Tonemapping::TonyMcMapface,
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        // Transform::default(),
        Bloom {
            ..Bloom::NATURAL
        },
    ));
    // .with_child((
    //     Camera3d::default(),
    //     bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
    //     Camera::default().target_headless(512, 512, &mut images),
    //     CaptureBundle::default(),
    // ));

    // second camera
    // commands.spawn(
    //     (
    //         Camera3d::default(),
    //         bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
    //         Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    //         Camera::default().target_headless(512, 512, &mut images),
    //         CaptureBundle::default(),
    //         )
    // );

    // commands.spawn(
    //     DirectionalLight {
    //         shadows_enabled: true,
    //         ..default()
    //     }
    // );


    // scene
    // let path = "scene-static-triangle.glb";
    let path = "scene.glb";
    commands.spawn(SceneRoot(asset_server.load_with_settings(GltfAssetLabel::Scene(0).from_asset(path),|settings: &mut GltfLoaderSettings| settings.load_cameras = false)));


    
}

fn wait_for_all_scenes_ready(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spawner: Res<SceneSpawner>,
    mut has_run: Local<bool>,
    scene_objects_query: Query<(&Name, &Transform, Entity)>,
    scene_instance_query: Query<&SceneInstance>,
) {
    if *has_run {
        return;
    }

    // check check if there is a SceneInstance component
    for instance in &scene_instance_query {
        if spawner.instance_is_ready(**instance) {
            println!("Scene is ready: {:?}", **instance);

            let colors = [
                Color::srgb_u8(255, 0, 0),     // Red
                Color::srgb_u8(0, 255, 0),     // Green
                Color::srgb_u8(0, 0, 255),     // Blue
                Color::srgb_u8(255, 255, 0),   // Yellow
                Color::srgb_u8(255, 0, 255),   // Magenta
                Color::srgb_u8(0, 255, 255),   // Cyan
            ];

            let mut i = 0;
            for (name, transform, entity) in &scene_objects_query {
                println!("Entity: {} | Transform: T{:?} R{:?}", name.as_str(), transform.translation, Mat3::from_quat(transform.rotation));
                if name.as_str().starts_with("data_camera") {
                    let color = colors[i % colors.len()];
                    commands.spawn((
                        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                        MeshMaterial3d(materials.add(color)),
                        Transform::from_translation(Vec3::ZERO),
                    )).insert(ChildOf(entity));

                    commands.spawn(
                        (
                            Camera3d::default(),
                            bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
                            Transform::from_translation(Vec3::ZERO),
                            Camera::default().target_headless(512, 512, &mut images),
                            CaptureBundle::default(),
                        )
                    ).insert(ChildOf(entity));

                    commands.entity(entity).insert(CameraIndex { index: i });

                    i = i + 1;
                }
            }
            *has_run = true;
        } else {
            println!("Scene NOT ready: {:?}", **instance);
        }
    }
}

// fn update(
//     mut app_exit: EventWriter<AppExit>,
//     mut capture: Query<&mut Capture>,
//     mut cubes: Query<&mut Transform, With<Cube>>,
//     mut frame: Local<u32>,
// ) {
//     let mut capture = capture.single_mut().unwrap();
//     if !capture.is_capturing() {
//         capture.start(frames::FramesEncoder::new("captures/simple/frames"));
//     }

//     for mut transform in &mut cubes {
//         transform.rotation = Quat::from_rotation_y(*frame as f32 / 60.0 * TAU)
//     }

//     *frame += 1;

//     if *frame >= 15 {
//         capture.stop();
//         println!("Done");
//         app_exit.write(AppExit::Success);
//     }
// }


fn update(
    mut app_exit: EventWriter<AppExit>,
    time: Res<Time>,
    mut capture: Query<&mut Capture>,
    mut cubes: Query<&mut Transform, With<Cube>>,
    mut frame: Local<u32>,
    mut recording: ResMut<Recording>,
) {
    if !recording.active {
        return;
    }


    for (i, mut capture) in capture.iter_mut().enumerate() {
        if !capture.is_capturing() {
            let name = format!("cam{}_frame", i);
            let encoder = MyCustomEncoder::new(&name, 512, 512);
            capture.start(encoder);
        }
    }

    let rotation_speed = TAU / 2.0;  // One full rotation every 2 seconds

    for mut transform in &mut cubes {
        transform.rotate_y(time.delta_secs() / 2.);
    }

    *frame += 1;

    // if *frame >= 500 {
    //     capture.stop();
    //     println!("Done");
    //     recording.active = false;
    //     // app_exit.write(AppExit::Success);
    // }



    // if recording.stop_next_frame{
    //     println!("set stopdqa");
    //     recording.stop_next_frame = false;
    //     capture.pause();
    //     recording.active = false;
    // }
    //
    // if recording.single {
    //     recording.single = false;
    //     println!("set stop_next_frame");
    //     recording.stop_next_frame = true;
    //     capture.resume();
    // }
}


fn frame_limiter() {
    // std::thread::sleep(std::time::Duration::from_millis(150)); // ~30 FPS
}



fn toggle_ambient_light_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyL) {
        if ambient_light.brightness > 0.0 {
            ambient_light.brightness = 0.0;
        } else {
            ambient_light.brightness = 500.0; // Set desired brightness when toggled on
        }
        println!("Ambient light toggled. Brightness: {}", ambient_light.brightness);
    }
}

fn toggle_gizmos_on_top(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
        config.depth_bias = if config.depth_bias == 0.0 { -1.0 } else { 0.0 };
        println!(
            "Gizmo depth_bias toggled. Now: {}",
            if config.depth_bias < 0.0 { "On Top" } else { "Normal" }
        );
    }

    if keyboard_input.just_pressed(KeyCode::KeyG) {
        let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
        config.enabled = !config.enabled;
        println!("Gizmo depth_bias toggled. Now: {}", config.enabled);
    }
}


fn draw_line_example(mut gizmos: Gizmos) {
    let red_color = Color::linear_rgba(1.0, 0.0, 0.0, 1.0); 
    let green_color = Color::linear_rgba(0.0, 1.0, 0.0, 1.0); 
    let blue_color = Color::linear_rgba(0.0, 0.0, 1.0, 1.0); 
    
    gizmos.line(Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), red_color);   // X
    gizmos.line(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), green_color); // Y
    gizmos.line(Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), blue_color);  // Z
}

#[derive(Resource)]
pub struct LineBuffer {
    shmem: shared_memory::Shmem,
}

impl LineBuffer {
    pub fn new(name: &str, max_lines: usize) -> Self {
        let floats_per_line = 2 * 3; // start + end, Vec3
        let buffer_size = max_lines * floats_per_line * std::mem::size_of::<f32>();
        println!("buffer size {}", buffer_size);

        unsafe {
        let mut shmem = match ShmemConf::new().os_id(name).size(buffer_size).create() {
            Ok(mem) => mem,
            Err(e) => {
                if let shared_memory::ShmemError::MappingIdExists = e {
                    ShmemConf::new().os_id(name).open().unwrap()
                } else {
                    panic!("Failed to create/open shared memory: {}", e);
                }
            }
        };

        // Initialize the shared memory (fill with zeroes)
        
        shmem.as_slice_mut().fill(0);
        Self { shmem }
        }
    }

    pub fn read_lines(&self) -> Vec<(Vec3, Vec3)> {
        let data = unsafe { self.shmem.as_slice() };
        let float_data: &[f32] = bytemuck::cast_slice(data);

        let mut lines = Vec::new();
        for chunk in float_data.chunks_exact(6) {
            let start = Vec3::new(chunk[0], chunk[1], chunk[2]);
            let end = Vec3::new(chunk[3], chunk[4], chunk[5]);
            lines.push((start, end));
        }
        lines
    }
}

unsafe impl Send for LineBuffer {}
unsafe impl Sync for LineBuffer {}

fn draw_lines_from_buffer(mut gizmos: Gizmos, line_buffer: Res<LineBuffer>) {
    for (i, (start, end)) in line_buffer.read_lines().iter().enumerate() {
        // if i < 10 {
        //     println!("Line {}: Start {:?} -> End {:?}", i, start, end);
        // }
        gizmos.line(*start, *end, Color::linear_rgba(0.0, 1.0, 0.0, 1.0));
    }
}



// Send Real Camera Coords

#[derive(Resource)]
pub struct PoseBuffer {
    shmem: shared_memory::Shmem,
}

impl PoseBuffer {
    pub fn new(name: &str, max_lines: usize) -> Self {
        let floats_per_line = 12;
        let buffer_size = max_lines * floats_per_line * std::mem::size_of::<f32>();
        println!("buffer size {}", buffer_size);

        unsafe {
        let mut shmem = match ShmemConf::new().os_id(name).size(buffer_size).create() {
            Ok(mem) => mem,
            Err(e) => {
                if let shared_memory::ShmemError::MappingIdExists = e {
                    ShmemConf::new().os_id(name).open().unwrap()
                } else {
                    panic!("Failed to create/open shared memory: {}", e);
                }
            }
        };

        // Initialize the shared memory (fill with zeroes)
        
        shmem.as_slice_mut().fill(0);
        Self { shmem }
        }
    }

    pub fn set_line(&mut self, index: usize, transform: &GlobalTransform) {
        // println!("Set pose {}, {:?}", index, transform);
        let floats_per_line = 12;
        let offset = index * floats_per_line;
    
        let byte_slice = unsafe { self.shmem.as_slice_mut() };
        let float_slice = unsafe {
            std::slice::from_raw_parts_mut(
                byte_slice.as_mut_ptr() as *mut f32,
                byte_slice.len() / std::mem::size_of::<f32>(),
            )
        };
    
        assert!(offset + floats_per_line <= float_slice.len(), "Index out of bounds");
    
        let translation = transform.translation();
        let rotation: Mat3 = Mat3::from_quat(transform.rotation());
    
        float_slice[offset] = translation.x;
        float_slice[offset + 1] = translation.y;
        float_slice[offset + 2] = translation.z;
    
        let rot_cols = [rotation.x_axis, rotation.y_axis, rotation.z_axis];
        for (col_idx, col) in rot_cols.iter().enumerate() {
            float_slice[offset + 3 + col_idx * 3 + 0] = col.x;
            float_slice[offset + 3 + col_idx * 3 + 1] = col.y;
            float_slice[offset + 3 + col_idx * 3 + 2] = col.z;
        }
    }
}

unsafe impl Send for PoseBuffer {}
unsafe impl Sync for PoseBuffer {}


// system to sync camera positions

fn write_camera_poses_system(
    mut pose_buffer: ResMut<PoseBuffer>,
    query: Query<(&CameraIndex, &GlobalTransform)>,
) {
    for (camera_index, global_transform) in query.iter() {
        pose_buffer.set_line(camera_index.index, global_transform);
    }
}


fn oscillate_camera_index_1(
    time: Res<Time>,
    mut active: Local<bool>,
    mut testId: Local<u8>,
    mut origin_y: Local<f32>,
    mut direction: Local<f32>,
    mut is_initialized: Local<bool>,
    mut query: Query<(&mut Transform, &CameraIndex)>,
    keyboard_input: Res<ButtonInput<KeyCode>>
) {
    if keyboard_input.just_pressed(KeyCode::KeyM) {
        *active = !*active;
        info!("Camera Mover toggled: {}", *active);
    }

    if keyboard_input.just_pressed(KeyCode::KeyN) {
        *testId = (*testId + 1) % 4;
        info!("Camera Mover Test: {}", *testId);
    }

    if !*active{
        return;
    }



    let speed = 1.0;
    let amplitude = 1.0;

    for (mut transform, index) in query.iter_mut() {
        if index.index != 1 {
            continue;
        }

        // Initialize origin_y on first frame
        if !*is_initialized{
            *is_initialized = true;
            *origin_y = transform.translation.z;
            *direction = -1.0;
        }

        match *testId {
            0 => {
                transform.rotate_local_x(time.delta_secs() / 2.);
                return;
            }
            1 => {
                transform.rotate_local_y(time.delta_secs() / 2.);
                return;
            }
            2 => {
                transform.rotate_local_z(time.delta_secs() / 2.);
                return;
            }
            _ => {}
        }
        

        // Move
        transform.translation.z += *direction * speed * time.delta_secs();

        // Reverse direction if out of bounds
        let offset = transform.translation.z - *origin_y;
        if offset > amplitude {
            transform.translation.z = *origin_y + amplitude;
            *direction = -1.0;
        } else if offset < -amplitude {
            transform.translation.z = *origin_y - amplitude;
            *direction = 1.0;
        }
    }
}