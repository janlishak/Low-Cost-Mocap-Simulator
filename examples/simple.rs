use bevy::{gltf::GltfLoaderSettings, prelude::*, render::RenderPlugin, scene::SceneInstance, window::WindowResolution};
use bevy_flycam::prelude::*;
use bevy_capture::{
    encoder::frames,
    CameraTargetHeadless, Capture, CaptureBundle,
};
use std::{f32::consts::TAU, fs};
use bevy::pbr::CascadeShadowConfigBuilder;

fn main() -> AppExit {
    // Create the captures directory
    fs::create_dir_all("captures/simple").unwrap();

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins.set(RenderPlugin {
            synchronous_pipeline_compilation: true,
            ..default()
        })
        .set(WindowPlugin { 
            primary_window: Some(Window { 
                present_mode: bevy::window::PresentMode::Immediate, // no Vsync
                title: String::from("Minimal FPS Controller Example"),
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


    app.insert_resource(MovementSettings {
        sensitivity: 0.0001, // default: 0.00012
        speed: 3.0, // default: 12.0
    });
    app.insert_resource(KeyBindings {
        move_ascend: KeyCode::KeyE,
        move_descend: KeyCode::KeyQ,
        ..Default::default()
    });

    app.insert_resource(ClearColor(Color::Hsva(Hsva { hue: 0.0, saturation: 0.0, value: 0.0, alpha: 1.0 })));// green skybox color
    app.insert_resource(Recording::default());

    app.add_systems(Update, toggle_recording);
    app.add_systems(Update, monitor_recording);

    app.add_systems(Update, wait_for_all_scenes_ready);

    // Run the app
    app.run()
}

#[derive(Component)]
struct Cube;

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
        info!("Recording is active.");
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

    // // cube
    // commands.spawn((
    //     Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
    //     MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
    //     Transform::from_xyz(0.0, 0.5, 0.0),
    //     Cube
    // ));

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
        bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ))
    .with_child((
        Camera3d::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::AcesFitted,
        Camera::default().target_headless(512, 512, &mut images),
        CaptureBundle::default(),
    ));

    // commands.spawn(
    //     DirectionalLight {
    //         shadows_enabled: true,
    //         ..default()
    //     }
    // );


    // scene
    commands.spawn(SceneRoot(asset_server.load_with_settings(GltfAssetLabel::Scene(0).from_asset("scene.glb"),|settings: &mut GltfLoaderSettings| settings.load_cameras = false)));


    
}

fn wait_for_all_scenes_ready(
    spawner: Res<SceneSpawner>,
    mut has_run: Local<bool>,
    scene_objects_query: Query<(&Name, &Transform)>,
    scene_instance_query: Query<&SceneInstance>,
) {
    if *has_run {
        return;
    }

    // check check if there is a SceneInstance component
    for instance in &scene_instance_query {
        if spawner.instance_is_ready(**instance) {
            println!("Scene is ready: {:?}", **instance);
            for (name, transform) in &scene_objects_query {
                println!("Entity: {} | Transform: T{:?} R{:?}", name.as_str(), transform.translation, transform.rotation);
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
    mut capture: Query<&mut Capture>,
    mut cubes: Query<&mut Transform, With<Cube>>,
    mut frame: Local<u32>,
    mut recording: ResMut<Recording>,
) {
    if !recording.active {
        return;
    }

    let mut capture = capture.single_mut().unwrap();

    if !capture.is_capturing() {
        capture.start(frames::FramesEncoder::new("captures/simple/frames2"));
    }
 
    for mut transform in &mut cubes {
        transform.rotation = Quat::from_rotation_y(*frame as f32 / 60.0 * TAU);
    }

    *frame += 1;

    if *frame >= 500 {
        capture.stop();
        println!("Done");
        recording.active = false;
        // app_exit.write(AppExit::Success);
    }



    if recording.stop_next_frame{
        println!("set stopdqa");
        recording.stop_next_frame = false;
        capture.pause();
        recording.active = false;
    }

    if recording.single {
        recording.single = false;
        println!("set stop_next_frame");
        recording.stop_next_frame = true;  
        capture.resume();
    }
}


fn frame_limiter() {
    // std::thread::sleep(std::time::Duration::from_millis(150)); // ~30 FPS
}
