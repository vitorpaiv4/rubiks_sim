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
    pub floating: bool,
    pub focus: Vec3,
}

impl Default for CameraOrbit {
    fn default() -> Self {
        Self {
            radius: 9.5,
            yaw: 0.0,
            pitch: 0.55,
            auto_rotate: false,
            floating: false,
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

fn raycast_cubies(
    ray: Ray3d,
    cubies: &Query<(&Cubie, &Transform), Without<CameraOrbit>>,
) -> Option<(f32, Vec3, IVec3, Vec3)> {
    let ray_origin = ray.origin;
    let ray_dir = ray.direction.normalize();
    let mut closest_hit: Option<(f32, Vec3, IVec3, Vec3)> = None;

    for (cubie, transform) in cubies {
        let half = Vec3::splat(CUBIE_SIZE * 0.52);
        let min = transform.translation - half;
        let max = transform.translation + half;

        if let Some((t, normal)) = intersect_ray_aabb(ray_origin, ray_dir, min, max) {
            if closest_hit.map_or(true, |(closest_t, _, _, _)| t < closest_t) {
                let hit_world = ray_origin + ray_dir * t;
                closest_hit = Some((t, normal, cubie.logical_pos, hit_world));
            }
        }
    }
    closest_hit
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

    let scale = window.scale_factor().max(1.0);

    // Atalho teclado opcional para PC
    if keys.just_pressed(KeyCode::Space) {
        orbit.floating = !orbit.floating;
    }

    let touch_count = touches.iter().count();

    // =========================================================================
    // 1. GESTO DE 2 DEDOS (TOUCH MOBILE): ÓRBITA DA CÂMERA + PINCH TO ZOOM
    // =========================================================================
    if touch_count >= 2 {
        // Cancela qualquer arraste de peça de 1 dedo ativo
        interaction.drag = None;
        interaction.is_orbiting = true;

        let mut touches_iter = touches.iter();
        let t1 = touches_iter.next().unwrap();
        let t2 = touches_iter.next().unwrap();

        // Rotação/Órbita com 2 dedos movendo juntos
        let avg_delta = (t1.delta() + t2.delta()) * 0.5;
        let delta_logical = avg_delta / scale;
        orbit.yaw -= delta_logical.x * 0.008;
        orbit.pitch = (orbit.pitch + delta_logical.y * 0.008).clamp(-1.45, 1.45);

        // Zoom por pinça (Pinch-to-zoom)
        let dist = (t1.position() - t2.position()).length();
        let prev_dist = (t1.previous_position() - t2.previous_position()).length();
        let pinch_delta = (dist - prev_dist) / scale;
        orbit.radius = (orbit.radius - pinch_delta * 0.025).clamp(3.5, 22.0);
    }

    // =========================================================================
    // 2. GESTO DE 1 DEDO / MOUSE: ARRASTAR PEÇA DO CUBO OU ÓRBITA NO FUNDO
    // =========================================================================
    let (current_pointer_pos, pointer_just_pressed, pointer_pressed, pointer_just_released) = if touch_count == 1 {
        let touch = touches.iter().next().unwrap();
        let pos_logical = touch.position() / scale;
        (
            Some(pos_logical),
            touches.any_just_pressed(),
            true,
            touches.any_just_released(),
        )
    } else if touch_count == 0 {
        (
            window.cursor_position(),
            mouse_buttons.just_pressed(MouseButton::Left),
            mouse_buttons.pressed(MouseButton::Left),
            mouse_buttons.just_released(MouseButton::Left),
        )
    } else {
        (None, false, false, false)
    };

    let right_pressed = mouse_buttons.pressed(MouseButton::Right);
    let middle_pressed = mouse_buttons.pressed(MouseButton::Middle);

    // INÍCIO DO TOQUE / CLIQUE (1 dedo ou botão esquerdo do mouse)
    if pointer_just_pressed && !ui_hover.is_hovering_ui && touch_count <= 1 {
        if let Some(screen_pos) = current_pointer_pos {
            // Tenta obter raio de projeção 3D
            let ray_opt = camera.viewport_to_world(camera_global, screen_pos)
                .or_else(|| camera.viewport_to_world(camera_global, screen_pos * scale));

            if let Some(ray) = ray_opt {
                if let Some((_, hit_normal, logical_pos, hit_world)) = raycast_cubies(ray, &cubies) {
                    // Tocou diretamente em uma peça do cubo
                    interaction.drag = Some(ActiveDrag {
                        start_screen_pos: screen_pos,
                        hit_world_pos: hit_world,
                        hit_normal,
                        logical_pos,
                        is_resolved: false,
                    });
                    interaction.is_orbiting = false;
                } else {
                    // Tocou no fundo / vazio
                    interaction.drag = None;
                    interaction.is_orbiting = true;
                }
            }
        }
    }

    // PROCESSAMENTO DO ARRASTE NA PEÇA DO CUBO
    if let Some(ref mut drag) = interaction.drag {
        if !drag.is_resolved && pointer_pressed && rotation_state.anim.is_none() {
            if let Some(screen_pos) = current_pointer_pos {
                let delta = screen_pos - drag.start_screen_pos;
                let min_drag_dist = 16.0; // Distância confortável para touch e mouse

                if delta.length() >= min_drag_dist {
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
                    let s0_opt = camera.world_to_viewport(camera_global, p0)
                        .map(|s| s / scale);

                    if let Some(s0) = s0_opt {
                        for d in candidate_directions {
                            let p_test = p0 + d * 0.5;
                            if let Some(s_test_raw) = camera.world_to_viewport(camera_global, p_test) {
                                let s_test = s_test_raw / scale;
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
                        // Calcula eixo de rotação: Ω = N × D
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

    // ÓRBITA COM 1 DEDO NO FUNDO OU MOUSE
    if interaction.is_orbiting && touch_count == 1 {
        if let Some(touch) = touches.iter().next() {
            let delta = touch.delta() / scale;
            orbit.yaw -= delta.x * 0.007;
            orbit.pitch = (orbit.pitch + delta.y * 0.007).clamp(-1.45, 1.45);
        }
    } else if (interaction.is_orbiting && pointer_pressed) || right_pressed {
        for motion in motion_events.read() {
            orbit.yaw -= (motion.delta.x / scale) * 0.006;
            orbit.pitch = (orbit.pitch + (motion.delta.y / scale) * 0.006).clamp(-1.45, 1.45);
        }
    } else if middle_pressed {
        for motion in motion_events.read() {
            let forward = -Vec3::new(
                orbit.pitch.cos() * orbit.yaw.sin(),
                orbit.pitch.sin(),
                orbit.pitch.cos() * orbit.yaw.cos(),
            ).normalize();
            let right = forward.cross(Vec3::Y).normalize();
            let up = right.cross(forward).normalize();
            orbit.focus += (-right * (motion.delta.x / scale) + up * (motion.delta.y / scale)) * 0.008;
        }
    } else {
        motion_events.clear();
    }

    // ZOOM VIA SCROLL (Desktop)
    for scroll in scroll_events.read() {
        orbit.radius = (orbit.radius - scroll.y * 0.6).clamp(3.5, 22.0);
    }

    // =========================================================================
    // 3. MODO FLUTUAR / ZERO GRAVITY (AUTO-ROTAÇÃO SUAVE + ONDA SENOIDAL)
    // =========================================================================
    if orbit.floating || orbit.auto_rotate {
        let elapsed = time.elapsed_seconds();
        orbit.yaw += 0.35 * time.delta_seconds();
        orbit.focus.y = (elapsed * 1.6).sin() * 0.22;
    } else {
        // Retorna suavemente para o centro se desligado
        orbit.focus.y = orbit.focus.y * 0.95;
    }

    // FINALIZAÇÃO DO CLIQUE / TOQUE
    if pointer_just_released || touch_count == 0 && !pointer_pressed {
        interaction.drag = None;
        interaction.is_orbiting = false;
    }

    // ATUALIZAÇÃO DA POSIÇÃO DA CÂMERA ESFÉRICA
    let x = orbit.radius * orbit.pitch.cos() * orbit.yaw.sin();
    let y = orbit.radius * orbit.pitch.sin();
    let z = orbit.radius * orbit.pitch.cos() * orbit.yaw.cos();

    camera_transform.translation = orbit.focus + Vec3::new(x, y, z);
    camera_transform.look_at(orbit.focus, Vec3::Y);
}
