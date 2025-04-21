use bevy::{
    app::{RunMode, ScheduleRunnerPlugin},
    prelude::*,
    render::RenderPlugin,
    time::TimeUpdateStrategy,
    winit::WinitPlugin,
};
use bevy_capture::{
    encoder::frames,
    CameraTargetHeadless, Capture, CaptureBundle,
};
use std::{f32::consts::TAU, fs, time::Duration};

fn main() -> AppExit {
    // Create the captures directory
    fs::create_dir_all("captures/simple").unwrap();

    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
            // .build()
            // // Disable the WinitPlugin to prevent the creation of a window
            // // .disable::<WinitPlugin>()
            // // Make sure pipelines are ready before rendering
            // .set(RenderPlugin {
            //     synchronous_pipeline_compilation: true,
            //     ..default()
            // }),
        // Add the ScheduleRunnerPlugin to run the app in loop mode
        // ScheduleRunnerPlugin {
        //     run_mode: RunMode::Loop { wait: None },
        // },
        // Add the CapturePlugin
        bevy_capture::CapturePlugin,
    ));

    // Update the time at a fixed rate of 60 FPS
    // app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
    //     1.0 / 60.0,
    // )));

    // Setup
    // app.add_systems(Startup, setup);

    // Update
    app.add_systems(Update, update);
    app.add_systems(Startup, setup_3d);

    // Run the app
    app.run()
}

#[derive(Component)]
struct Cube;

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        Camera2d,
        Camera::default().target_headless(512, 512, &mut images),
        CaptureBundle::default(),
    ));

    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(128.0, 128.0))),
        MeshMaterial2d(materials.add(Color::srgb(0.0, 0.0, 1.0))),
        Cube,
    ));
}

fn setup_3d(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Cube
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    // camera
    commands.spawn((
        Camera3d::default(),
        Camera::default().target_headless(512, 512, &mut images),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        CaptureBundle::default(),
    ));
}

fn update(
    mut app_exit: EventWriter<AppExit>,
    mut capture: Query<&mut Capture>,
    mut cubes: Query<&mut Transform, With<Cube>>,
    mut frame: Local<u32>,
) {
    let mut capture = capture.single_mut().unwrap();
    if !capture.is_capturing() {
        capture.start(frames::FramesEncoder::new("captures/simple/frames"));
    }

    for mut transform in &mut cubes {
        transform.rotation = Quat::from_rotation_y(*frame as f32 / 60.0 * TAU)
    }

    *frame += 1;

    if *frame >= 15 {
        capture.stop();
        println!("Done");
        app_exit.write(AppExit::Success);
    }
}
