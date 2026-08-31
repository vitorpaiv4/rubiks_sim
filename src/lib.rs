pub mod cube;
pub mod interaction;
pub mod ui;

use bevy::prelude::*;
use bevy::winit::WinitWindows;
use cube::CubePlugin;
use interaction::{CameraOrbit, InteractionPlugin};
use ui::UiPlugin;
use winit::window::Icon;

#[bevy_main]
pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rubik's Cube 3D".into(),
                resolution: (800.0_f32, 600.0_f32).into(),
                decorations: true,
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CubePlugin)
        .add_plugins(InteractionPlugin)
        .add_plugins(UiPlugin)
        .add_systems(Startup, (setup_scene, set_window_icon))
        .add_systems(Update, close_app)
        .run();
}

fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_query: Query<Entity, With<bevy::window::PrimaryWindow>>,
) {
    let Ok(entity) = primary_query.get_single() else { return; };
    let Some(primary) = windows.get_window(entity) else { return; };

    let icon_bytes = include_bytes!("../assets/icon.png");
    if let Ok(image) = image::load_from_memory(icon_bytes) {
        let image = image.into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        if let Ok(icon) = Icon::from_rgba(rgba, width, height) {
            primary.set_window_icon(Some(icon));
        }
    }
}

fn close_app(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<bevy::app::AppExit>) {
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyQ) {
        exit.send(bevy::app::AppExit);
    }
}

fn setup_scene(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.85,
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(4.0, 7.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            illuminance: 1600.0,
            shadows_enabled: false,
            ..default()
        },
        ..default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraOrbit::default(),
    ));
}
