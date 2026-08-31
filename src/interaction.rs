use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::input::touch::Touches;
use crate::cube::{Cubie, MoveCommand, MoveQueue, RotationState, CUBIE_SIZE};
use crate::ui::UiHoverState;

#[derive(Component)]
pub struct CameraOrbit {
    pub radius: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub auto_rotate: bool,
    pub focus: Vec3,
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

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<CubeInteractionState>()
            .add_systems(Update, handle_cube_and_camera_interaction);
    }
}

#[derive(Resource, Default)]
pub struct CubeInteractionState {
    pub drag: Option<ActiveDrag>,
    pub is_orbiting: bool,
}

pub struct ActiveDrag {
    pub start_screen_pos: Vec2,
    pub hit_world_pos: Vec3,
    pub hit_normal: Vec3,
    pub logical_pos: IVec3,
    pub is_resolved: bool,
}

fn intersect_ray_aabb(ray_origin: Vec3, ray_dir: Vec3, aabb_min: Vec3, aabb_max: Vec3) -> Option<(f32, Vec3)> {
    let mut tmin = (aabb_min.x - ray_origin.x) / ray_dir.x;
    let mut tmax = (aabb_max.x - ray_origin.x) / ray_dir.x;
    let mut normal_x = if ray_dir.x > 0.0 { Vec3::NEG_X } else { Vec3::X };

    if tmin > tmax {
        std::mem::swap(&mut tmin, &mut tmax);
        normal_x = -normal_x;
    }

    let mut tymin = (aabb_min.y - ray_origin.y) / ray_dir.y;
    let mut tymax = (aabb_max.y - ray_origin.y) / ray_dir.y;
    let mut normal_y = if ray_dir.y > 0.0 { Vec3::NEG_Y } else { Vec3::Y };

    if tymin > tymax {
        std::mem::swap(&mut tymin, &mut tymax);
        normal_y = -normal_y;
    }

    if (tmin > tymax) || (tymin > tmax) {
        return None;
    }

    let mut hit_normal = normal_x;
    if tymin > tmin {
        tmin = tymin;
        hit_normal = normal_y;
    }

    if tymax < tmax {
        tmax = tymax;
    }

    let mut tzmin = (aabb_min.z - ray_origin.z) / ray_dir.z;
    let mut tzmax = (aabb_max.z - ray_origin.z) / ray_dir.z;
    let mut normal_z = if ray_dir.z > 0.0 { Vec3::NEG_Z } else { Vec3::Z };

    if tzmin > tzmax {
        std::mem::swap(&mut tzmin, &mut tzmax);
        normal_z = -normal_z;
    }

    if (tmin > tzmax) || (tzmin > tmax) {
        return None;
    }

    if tzmin > tmin {
        tmin = tzmin;
        hit_normal = normal_z;
    }

    if tmin < 0.0 {
        return None;
    }

    Some((tmin, hit_normal))
}

fn handle_cube_and_camera_interaction(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    touches: Res<Touches>,
    mut motion_events: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mut interaction: ResMut<CubeInteractionState>,
    mut queue: ResMut<MoveQueue>,
    rotation_state: Res<RotationState>,
    ui_hover: Res<UiHoverState>,
    windows: Query<&Window>,
    mut orbit_query: Query<(&mut Transform, &mut CameraOrbit, &Camera, &GlobalTransform)>,
    cubies: Query<(&Cubie, &Transform), Without<CameraOrbit>>,
) {
    let Ok((mut camera_transform, mut orbit, camera, camera_global)) = orbit_query.get_single_mut() else {
        return;
    };
    let Some(window) = windows.iter().next() else {
        return;
    };

    if keys.just_pressed(KeyCode::Space) {
        orbit.auto_rotate = !orbit.auto_rotate;
    }

    // Identifica posição de cursor ativo (Mouse ou Touch)
    let current_pointer_pos = if let Some(touch) = touches.iter().next() {
        Some(touch.position())
    } else {
        window.cursor_position()
    };

    let pointer_just_pressed = mouse_buttons.just_pressed(MouseButton::Left) || touches.any_just_pressed();
    let pointer_pressed = mouse_buttons.pressed(MouseButton::Left) || touches.iter().next().is_some();
    let pointer_just_released = mouse_buttons.just_released(MouseButton::Left) || touches.any_just_released();

    let right_pressed = mouse_buttons.pressed(MouseButton::Right);
    let middle_pressed = mouse_buttons.pressed(MouseButton::Middle);

    // 1. INÍCIO DO CLIQUE / TOQUE
    if pointer_just_pressed && !ui_hover.is_hovering_ui {
        if let Some(screen_pos) = current_pointer_pos {
            if let Some(ray) = camera.viewport_to_world(camera_global, screen_pos) {
                let ray_origin = ray.origin;
                let ray_dir = ray.direction.normalize();

                let mut closest_hit: Option<(f32, Vec3, IVec3, Vec3)> = None;

                for (cubie, transform) in &cubies {
                    let half = Vec3::splat(CUBIE_SIZE * 0.5);
                    let min = transform.translation - half;
                    let max = transform.translation + half;

                    if let Some((t, normal)) = intersect_ray_aabb(ray_origin, ray_dir, min, max) {
                        if closest_hit.map_or(true, |(closest_t, _, _, _)| t < closest_t) {
                            let hit_world = ray_origin + ray_dir * t;
                            closest_hit = Some((t, normal, cubie.logical_pos, hit_world));
                        }
                    }
                }

                if let Some((_, hit_normal, logical_pos, hit_world)) = closest_hit {
                    // Clicou diretamente sobre uma peça do cubo
                    interaction.drag = Some(ActiveDrag {
                        start_screen_pos: screen_pos,
                        hit_world_pos: hit_world,
                        hit_normal,
                        logical_pos,
                        is_resolved: false,
                    });
                    interaction.is_orbiting = false;
                } else {
                    // Clicou no espaço vazio / fundo -> inicia órbita de câmera
                    interaction.drag = None;
                    interaction.is_orbiting = true;
                }
            }
        }
    }

    // 2. PROCESSAMENTO DO ARRASTE NA PEÇA DO CUBO
    if let Some(ref mut drag) = interaction.drag {
        if !drag.is_resolved && pointer_pressed && rotation_state.anim.is_none() {
            if let Some(screen_pos) = current_pointer_pos {
                let delta = screen_pos - drag.start_screen_pos;
                if delta.length() >= 14.0 {
                    // Calcula as direções ortogonais na face clicada
                    let normal = drag.hit_normal;
                    let (tangent_a, tangent_b) = if normal.x.abs() > 0.8 {
                        (Vec3::Y, Vec3::Z)
                    } else if normal.y.abs() > 0.8 {
                        (Vec3::X, Vec3::Z)
                    } else {
                        (Vec3::X, Vec3::Y)
                    };

                    let candidate_directions = [
                        tangent_a,
                        -tangent_a,
                        tangent_b,
                        -tangent_b,
                    ];

                    let mut best_score = 0.0_f32;
                    let mut best_dir = tangent_a;

                    let p0 = drag.hit_world_pos;
                    if let Some(s0) = camera.world_to_viewport(camera_global, p0) {
                        for d in candidate_directions {
                            let p_test = p0 + d * 0.5;
                            if let Some(s_test) = camera.world_to_viewport(camera_global, p_test) {
                                let screen_dir = s_test - s0;
                                if screen_dir.length_squared() > 0.001 {
                                    let norm_dir = screen_dir.normalize();
                                    let score = delta.dot(norm_dir);
                                    if score > best_score {
                                        best_score = score;
                                        best_dir = d;
                                    }
                                }
                            }
                        }
                    }

                    if best_score > 10.0 {
                        // Calcula o eixo de rotação: Ω = N × D
                        let omega = normal.cross(best_dir);
                        let (axis, layer, sign) = if omega.x.abs() > 0.8 {
                            (Vec3::X, drag.logical_pos.x, omega.x.signum())
                        } else if omega.y.abs() > 0.8 {
                            (Vec3::Y, drag.logical_pos.y, omega.y.signum())
                        } else {
                            (Vec3::Z, drag.logical_pos.z, omega.z.signum())
                        };

                        let angle = std::f32::consts::FRAC_PI_2 * sign;
                        queue.queue.push_back(MoveCommand::slice(axis, layer, angle, false));
                        drag.is_resolved = true;
                    }
                }
            }
        }
    }

    // 3. ÓRBITA E PAN DA CÂMERA (Mouse ou Touch no Fundo)
    let should_orbit = (interaction.is_orbiting && pointer_pressed) || right_pressed;
    let should_pan = middle_pressed;

    if should_orbit || should_pan {
        for motion in motion_events.read() {
            if should_orbit {
                orbit.yaw -= motion.delta.x * 0.006;
                orbit.pitch = (orbit.pitch + motion.delta.y * 0.006).clamp(-1.45, 1.45);
            } else if should_pan {
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

    // 4. SUPORTE A PINCH-TO-ZOOM E 2 DEDOS (TOUCH)
    if touches.iter().count() >= 2 {
        let mut touches_iter = touches.iter();
        let t1 = touches_iter.next().unwrap();
        let t2 = touches_iter.next().unwrap();
        let dist = (t1.position() - t2.position()).length();
        let prev_dist = (t1.previous_position() - t2.previous_position()).length();
        let pinch_delta = dist - prev_dist;
        orbit.radius = (orbit.radius - pinch_delta * 0.02).clamp(3.5, 20.0);
    }

    // 5. ZOOM VIA SCROLL DO MOUSE
    for scroll in scroll_events.read() {
        orbit.radius = (orbit.radius - scroll.y * 0.6).clamp(3.5, 20.0);
    }

    // 6. AUTO ROTAÇÃO DA CÂMERA
    if orbit.auto_rotate {
        orbit.yaw += 0.35 * time.delta_seconds();
    }

    // 7. FINALIZAÇÃO DO CLIQUE / TOQUE
    if pointer_just_released {
        interaction.drag = None;
        interaction.is_orbiting = false;
    }

    // Atualiza posição da câmera esférica
    let x = orbit.radius * orbit.pitch.cos() * orbit.yaw.sin();
    let y = orbit.radius * orbit.pitch.sin();
    let z = orbit.radius * orbit.pitch.cos() * orbit.yaw.cos();

    camera_transform.translation = orbit.focus + Vec3::new(x, y, z);
    camera_transform.look_at(orbit.focus, Vec3::Y);
}
