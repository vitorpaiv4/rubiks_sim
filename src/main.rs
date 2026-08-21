mod cube;
mod retro;

use bevy::prelude::*;
use bevy::render::view::Msaa;
use cube::CubePlugin;
use retro::RetroPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rubik's Cube".into(),
                resolution: (640.0_f32, 480.0_f32).into(),
                decorations: false,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((CubePlugin, RetroPlugin))
        .add_systems(Startup, setup_cena)
        .add_systems(Update, (girar_camera, fechar_app))
        .run();
}

fn fechar_app(keys: Res<ButtonInput<KeyCode>>, mut exit: EventWriter<bevy::app::AppExit>) {
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyQ) {
        exit.send(bevy::app::AppExit);
    }
}

#[derive(Component)]
struct CameraOrbit;

fn setup_cena(mut commands: Commands) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.8,
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(3.0, 6.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            illuminance: 1500.0,
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
        CameraOrbit,
    ));

    commands.spawn(TextBundle {
        text: Text::from_section(
            "U D R L F B  |  Shift+inverso  |  S=scramble  |  X=reset  |  Esc=sair",
            TextStyle {
                font_size: 11.0,
                color: Color::rgba(0.6, 0.6, 0.6, 0.6),
                ..default()
            },
        ),
        style: Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(8.0),
            left: Val::Px(8.0),
            ..default()
        },
        ..default()
    });
}

fn girar_camera(time: Res<Time>, mut query: Query<&mut Transform, With<CameraOrbit>>) {
    for mut transform in &mut query {
        let rotacao = Quat::from_rotation_y(0.5 * time.delta_seconds());
        transform.translation = rotacao * transform.translation;
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
