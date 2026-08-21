#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod cube;

use bevy::prelude::*;
use cube::CubePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Rubik's Cube".into(),
                resolution: (640.0_f32, 480.0_f32).into(),
                decorations: true,
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(CubePlugin)
        .add_systems(Startup, setup_cena)
        .add_systems(Update, (girar_camera, fechar_app, atualizar_hud))
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
    focus: Vec3,
}

impl Default for CameraOrbit {
    fn default() -> Self {
        Self {
            radius: 9.434,
            yaw: 0.0,
            pitch: 0.5586,
            auto_rotate: false,
            focus: Vec3::ZERO,
        }
    }
}

#[derive(Component)]
struct HudStatusText;

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

    // Banner Superior: Timer, Movimentos e Status
    commands.spawn((
        TextBundle {
            text: Text::from_section(
                "TEMPO: 00:00.00 | MOVIMENTOS: 0 | STATUS: PRONTO",
                TextStyle {
                    font_size: 13.0,
                    color: Color::rgb(0.9, 0.9, 0.2),
                    ..default()
                },
            ),
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                left: Val::Px(8.0),
                ..default()
            },
            ..default()
        },
        HudStatusText,
    ));

    // Instruções Inferiores
    commands.spawn(TextBundle {
        text: Text::from_section(
            "U D R L F B (Shift=inv) | S=scramble | X=reset | Mouse/Meio=orbit/pan | Scroll=zoom | Espaço=auto | Esc/Q=sair",
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

fn atualizar_hud(
    timer_state: Res<cube::GameTimerState>,
    queue: Res<cube::MoveQueue>,
    mut query: Query<&mut Text, With<HudStatusText>>,
) {
    for mut text in &mut query {
        let total_secs = timer_state.elapsed;
        let mins = (total_secs / 60.0).floor() as u32;
        let secs = (total_secs % 60.0).floor() as u32;
        let millis = ((total_secs % 1.0) * 100.0).floor() as u32;

        let status = if !queue.queue.is_empty() && queue.queue.iter().any(|m| m.is_scramble) {
            "EMBARALHANDO..."
        } else if timer_state.is_solved {
            "RESOLVIDO! ★"
        } else if timer_state.is_running {
            "RESOLVENDO"
        } else if timer_state.is_scrambled {
            "EMBARALHADO (gire qualquer face)"
        } else {
            "PRONTO"
        };

        text.sections[0].value = format!(
            "TEMPO: {:02}:{:02}.{:02} | MOVS: {} | {}",
            mins, secs, millis, timer_state.move_count, status
        );

        text.sections[0].style.color = if timer_state.is_solved {
            Color::rgb(0.2, 1.0, 0.4)
        } else if timer_state.is_running {
            Color::rgb(1.0, 0.85, 0.2)
        } else {
            Color::rgb(0.8, 0.8, 0.8)
        };
    }
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

        let is_orbiting = mouse_buttons.pressed(MouseButton::Left) || mouse_buttons.pressed(MouseButton::Right);
        let is_panning = mouse_buttons.pressed(MouseButton::Middle);

        if is_orbiting || is_panning {
            for motion in motion_events.read() {
                if is_orbiting {
                    orbit.yaw -= motion.delta.x * 0.006;
                    orbit.pitch = (orbit.pitch + motion.delta.y * 0.006).clamp(-1.45, 1.45);
                } else if is_panning {
                    let forward = -Vec3::new(
                        orbit.pitch.cos() * orbit.yaw.sin(),
                        orbit.pitch.sin(),
                        orbit.pitch.cos() * orbit.yaw.cos(),
                    ).normalize();
                    let right = forward.cross(Vec3::Y).normalize();
                    let up = right.cross(forward).normalize();
                    orbit.focus += (-right * motion.delta.x + up * motion.delta.y) * 0.008;
                }
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

        transform.translation = orbit.focus + Vec3::new(x, y, z);
        transform.look_at(orbit.focus, Vec3::Y);
    }
}
