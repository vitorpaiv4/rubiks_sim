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

use bevy::input::mouse::{MouseMotion, MouseWheel};

#[derive(Component)]
struct CameraOrbit {
    radius: f32,
    yaw: f32,
    pitch: f32,
    auto_rotate: bool,
}

impl Default for CameraOrbit {
    fn default() -> Self {
        Self {
            radius: 9.434,
            yaw: 0.0,
            pitch: 0.5586,
            auto_rotate: false,
        }
    }
}

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
        CameraOrbit::default(),
    ));

    commands.spawn(TextBundle {
        text: Text::from_section(
            "U D R L F B (Shift=inv) | S=scramble | X=reset | Mouse/Scroll=camera | Espaço=auto-rotate | Esc=sair",
            TextStyle {
                font_size: 10.5,
                color: Color::rgba(0.6, 0.6, 0.6, 0.7),
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

fn girar_camera(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut motion_events: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut CameraOrbit)>,
) {
    for (mut transform, mut orbit) in &mut query {
        if keys.just_pressed(KeyCode::Space) {
            orbit.auto_rotate = !orbit.auto_rotate;
        }

        let is_dragging = mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.pressed(MouseButton::Right);
        if is_dragging {
            for motion in motion_events.read() {
                orbit.yaw -= motion.delta.x * 0.006;
                orbit.pitch = (orbit.pitch + motion.delta.y * 0.006).clamp(-1.45, 1.45);
            }
        } else {
            motion_events.clear();
        }

        for scroll in scroll_events.read() {
            orbit.radius = (orbit.radius - scroll.y * 0.6).clamp(3.5, 20.0);
        }

        if orbit.auto_rotate {
            orbit.yaw += 0.4 * time.delta_seconds();
        }

        let x = orbit.radius * orbit.pitch.cos() * orbit.yaw.sin();
        let y = orbit.radius * orbit.pitch.sin();
        let z = orbit.radius * orbit.pitch.cos() * orbit.yaw.cos();

        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}
