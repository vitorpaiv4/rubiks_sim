use bevy::prelude::*;
use rand::Rng;
use std::collections::VecDeque;

const CUBIE_SIZE: f32 = 0.9;
const ADESIVO_ESPESSURA: f32 = 0.02;
const ADESIVO_TAMANHO: f32 = 0.8;
const ADESIVO_OFFSET: f32 = 0.46;
const ANIM_DURATION_USER: f32 = 0.14;
const ANIM_DURATION_SCRAMBLE: f32 = 0.055;

pub struct CubePlugin;

impl Plugin for CubePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<RotationState>()
            .init_resource::<MoveQueue>()
            .init_resource::<GameTimerState>()
            .add_systems(Startup, spawn_cube)
            .add_systems(Update, cube_system);
    }
}

#[derive(Resource, Default)]
pub struct GameTimerState {
    pub is_scrambled: bool,
    pub is_running: bool,
    pub elapsed: f32,
    pub move_count: u32,
    pub is_solved: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct MoveCommand {
    pub face: Face,
    pub inverse: bool,
    pub is_scramble: bool,
}

#[derive(Resource, Default)]
pub struct MoveQueue {
    pub queue: VecDeque<MoveCommand>,
}

#[derive(Component)]
pub struct Cubie {
    pub logical_pos: IVec3,
    pub initial_pos: IVec3,
}

#[derive(Component)]
struct CubeRoot;

#[derive(Resource)]
struct CubeAssets {
    base_mesh: Handle<Mesh>,
    sticker_meshes: StickerMeshes,
    materials: CubeMaterials,
}

struct StickerMeshes {
    x: Handle<Mesh>,
    y: Handle<Mesh>,
    z: Handle<Mesh>,
}

#[derive(Resource)]
struct CubeMaterials {
    base: Handle<StandardMaterial>,
    direita: Handle<StandardMaterial>,
    esquerda: Handle<StandardMaterial>,
    cima: Handle<StandardMaterial>,
    baixo: Handle<StandardMaterial>,
    frente: Handle<StandardMaterial>,
    tras: Handle<StandardMaterial>,
}

#[derive(Clone, Copy, Debug)]
pub enum Face {
    U, D, R, L, F, B,
}

impl Face {
    pub fn axis(&self) -> Vec3 {
        match self {
            Face::U | Face::D => Vec3::Y,
            Face::R | Face::L => Vec3::X,
            Face::F | Face::B => Vec3::Z,
        }
    }

    pub fn layer(&self) -> i32 {
        match self {
            Face::U => 1, Face::D => -1,
            Face::R => 1, Face::L => -1,
            Face::F => 1, Face::B => -1,
        }
    }

    pub fn target_angle(&self, inverse: bool) -> f32 {
        let ang = std::f32::consts::FRAC_PI_2;
        match (self, inverse) {
            (Face::U, false) => -ang,  (Face::U, true) => ang,
            (Face::D, false) => ang,   (Face::D, true) => -ang,
            (Face::R, false) => -ang,  (Face::R, true) => ang,
            (Face::L, false) => ang,   (Face::L, true) => -ang,
            (Face::F, false) => -ang,  (Face::F, true) => ang,
            (Face::B, false) => ang,   (Face::B, true) => -ang,
        }
    }
}

#[derive(Resource, Default)]
pub struct RotationState {
    anim: Option<RotationAnim>,
}

pub struct RotationAnim {
    axis: Vec3,
    target_angle: f32,
    elapsed: f32,
    duration: f32,
    entries: Vec<AnimEntry>,
}

pub struct AnimEntry {
    entity: Entity,
    start_pos: Vec3,
    start_rot: Quat,
    logical_pos: IVec3,
}

fn spawn_cube(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let base_mesh = meshes.add(Cuboid::new(CUBIE_SIZE, CUBIE_SIZE, CUBIE_SIZE));

    let sticker_x = meshes.add(Cuboid::new(ADESIVO_ESPESSURA, ADESIVO_TAMANHO, ADESIVO_TAMANHO));
    let sticker_y = meshes.add(Cuboid::new(ADESIVO_TAMANHO, ADESIVO_ESPESSURA, ADESIVO_TAMANHO));
    let sticker_z = meshes.add(Cuboid::new(ADESIVO_TAMANHO, ADESIVO_TAMANHO, ADESIVO_ESPESSURA));

    fn mat(color: Color) -> StandardMaterial {
        StandardMaterial { base_color: color, ..default() }
    }

    let base_mat  = materials.add(mat(Color::rgb(0.15, 0.15, 0.15)));
    let mat_direita  = materials.add(mat(Color::rgb(0.9, 0.1, 0.1)));
    let mat_esquerda = materials.add(mat(Color::rgb(1.0, 0.5, 0.0)));
    let mat_cima     = materials.add(mat(Color::rgb(1.0, 1.0, 1.0)));
    let mat_baixo    = materials.add(mat(Color::rgb(1.0, 0.9, 0.0)));
    let mat_frente   = materials.add(mat(Color::rgb(0.0, 0.8, 0.2)));
    let mat_tras     = materials.add(mat(Color::rgb(0.0, 0.3, 1.0)));

    let assets = CubeAssets {
        base_mesh,
        sticker_meshes: StickerMeshes { x: sticker_x, y: sticker_y, z: sticker_z },
        materials: CubeMaterials {
            base: base_mat,
            direita: mat_direita,
            esquerda: mat_esquerda,
            cima: mat_cima,
            baixo: mat_baixo,
            frente: mat_frente,
            tras: mat_tras,
        },
    };

    let root = commands.spawn((SpatialBundle::default(), CubeRoot)).id();

    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                let pos = IVec3::new(x, y, z);
                let cubie = commands.spawn((
                    PbrBundle {
                        mesh: assets.base_mesh.clone(),
                        material: assets.materials.base.clone(),
                        transform: Transform::from_translation(pos.as_vec3()),
                        ..default()
                    },
                    Cubie { logical_pos: pos, initial_pos: pos },
                )).with_children(|pai| {
                    if x == 1 { spawn_sticker(pai, &assets, "direita", Vec3::X); }
                    else if x == -1 { spawn_sticker(pai, &assets, "esquerda", Vec3::NEG_X); }

                    if y == 1 { spawn_sticker(pai, &assets, "cima", Vec3::Y); }
                    else if y == -1 { spawn_sticker(pai, &assets, "baixo", Vec3::NEG_Y); }

                    if z == 1 { spawn_sticker(pai, &assets, "frente", Vec3::Z); }
                    else if z == -1 { spawn_sticker(pai, &assets, "tras", Vec3::NEG_Z); }
                }).id();

                commands.entity(root).add_child(cubie);
            }
        }
    }

    commands.insert_resource(assets);
}

fn spawn_sticker(
    parent: &mut ChildBuilder,
    assets: &CubeAssets,
    face: &str,
    dir: Vec3,
) {
    let (mesh, material) = match face {
        "direita" | "esquerda" => (assets.sticker_meshes.x.clone(), sticker_material(&assets.materials, face)),
        "cima" | "baixo" => (assets.sticker_meshes.y.clone(), sticker_material(&assets.materials, face)),
        "frente" | "tras" => (assets.sticker_meshes.z.clone(), sticker_material(&assets.materials, face)),
        _ => unreachable!(),
    };
    parent.spawn(PbrBundle {
        mesh,
        material,
        transform: Transform::from_translation(dir * ADESIVO_OFFSET),
        ..default()
    });
}

fn sticker_material(assets: &CubeMaterials, face: &str) -> Handle<StandardMaterial> {
    match face {
        "direita"  => assets.direita.clone(),
        "esquerda" => assets.esquerda.clone(),
        "cima"     => assets.cima.clone(),
        "baixo"    => assets.baixo.clone(),
        "frente"   => assets.frente.clone(),
        "tras"     => assets.tras.clone(),
        _ => unreachable!(),
    }
}

fn cube_system(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut queue: ResMut<MoveQueue>,
    mut state: ResMut<RotationState>,
    mut timer_state: ResMut<GameTimerState>,
    mut cubies: Query<(Entity, &mut Transform, &mut Cubie)>,
) {
    if timer_state.is_running {
        timer_state.elapsed += time.delta_seconds();
    }

    // Escuta comandos a qualquer momento e adiciona na fila
    if let Some(face) = check_face_keys(&keys) {
        let inverse = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        queue.queue.push_back(MoveCommand {
            face,
            inverse,
            is_scramble: false,
        });
    }

    if keys.just_pressed(KeyCode::KeyS) {
        queue.queue.clear();
        timer_state.is_scrambled = true;
        timer_state.is_running = false;
        timer_state.is_solved = false;
        timer_state.elapsed = 0.0;
        timer_state.move_count = 0;
        queue_scramble(&mut queue);
    }

    if keys.just_pressed(KeyCode::KeyX) {
        queue.queue.clear();
        state.anim = None;
        timer_state.is_scrambled = false;
        timer_state.is_running = false;
        timer_state.is_solved = false;
        timer_state.elapsed = 0.0;
        timer_state.move_count = 0;
        for (_, mut transform, mut cubie) in &mut cubies {
            cubie.logical_pos = cubie.initial_pos;
            transform.translation = cubie.initial_pos.as_vec3();
            transform.rotation = Quat::IDENTITY;
        }
    }

    // Se nenhuma animação está ativa, consome o próximo movimento da fila
    if state.anim.is_none() {
        if let Some(cmd) = queue.queue.pop_front() {
            if !cmd.is_scramble {
                if timer_state.is_scrambled && !timer_state.is_running && !timer_state.is_solved {
                    timer_state.is_running = true;
                }
                if timer_state.is_running || timer_state.is_scrambled {
                    timer_state.move_count += 1;
                }
            }
            start_rotation(&mut state, cmd, &cubies);
        }
    }

    // Processa a animação ativa
    if let Some(ref mut anim) = state.anim {
        anim.elapsed += time.delta_seconds();
        let t = (anim.elapsed / anim.duration).min(1.0);
        let eased = smoothstep(t);

        for entry in &anim.entries {
            if let Ok((_, mut transform, _)) = cubies.get_mut(entry.entity) {
                let angle = anim.target_angle * eased;
                let rot = Quat::from_axis_angle(anim.axis, angle);
                transform.translation = rot * entry.start_pos;
                transform.rotation = rot * entry.start_rot;
            }
        }

        if t >= 1.0 {
            for entry in &anim.entries {
                if let Ok((_, mut transform, mut cubie)) = cubies.get_mut(entry.entity) {
                    let new_pos = rotate_logical(entry.logical_pos, anim.axis, anim.target_angle);
                    cubie.logical_pos = new_pos;
                    transform.translation = new_pos.as_vec3();
                    transform.rotation = Quat::from_axis_angle(anim.axis, anim.target_angle) * entry.start_rot;
                }
            }
            state.anim = None;

            // Se terminou o movimento e o cubo está em jogo cronometrado, verifica se resolveu
            if timer_state.is_running && queue.queue.is_empty() {
                let solved = cubies.iter().all(|(_, transform, cubie)| {
                    cubie.logical_pos == cubie.initial_pos && transform.rotation.dot(Quat::IDENTITY).abs() > 0.999
                });
                if solved {
                    timer_state.is_running = false;
                    timer_state.is_solved = true;
                }
            }
        }
    }
}

fn check_face_keys(keys: &Res<ButtonInput<KeyCode>>) -> Option<Face> {
    if keys.just_pressed(KeyCode::KeyU) { Some(Face::U) }
    else if keys.just_pressed(KeyCode::KeyD) { Some(Face::D) }
    else if keys.just_pressed(KeyCode::KeyR) { Some(Face::R) }
    else if keys.just_pressed(KeyCode::KeyL) { Some(Face::L) }
    else if keys.just_pressed(KeyCode::KeyF) { Some(Face::F) }
    else if keys.just_pressed(KeyCode::KeyB) { Some(Face::B) }
    else { None }
}

fn start_rotation(
    state: &mut RotationState,
    cmd: MoveCommand,
    cubies: &Query<(Entity, &mut Transform, &mut Cubie)>,
) {
    let axis = cmd.face.axis();
    let layer = cmd.face.layer();
    let target_angle = cmd.face.target_angle(cmd.inverse);
    let duration = if cmd.is_scramble { ANIM_DURATION_SCRAMBLE } else { ANIM_DURATION_USER };

    let entries: Vec<AnimEntry> = cubies.iter()
        .filter(|(_, _, cubie)| {
            let pos = cubie.logical_pos;
            (axis == Vec3::X && pos.x == layer) ||
            (axis == Vec3::Y && pos.y == layer) ||
            (axis == Vec3::Z && pos.z == layer)
        })
        .map(|(entity, transform, cubie)| {
            AnimEntry {
                entity,
                start_pos: transform.translation,
                start_rot: transform.rotation,
                logical_pos: cubie.logical_pos,
            }
        })
        .collect();

    if entries.is_empty() { return; }

    state.anim = Some(RotationAnim {
        axis,
        target_angle,
        elapsed: 0.0,
        duration,
        entries,
    });
}

fn queue_scramble(queue: &mut MoveQueue) {
    let faces = [Face::U, Face::D, Face::R, Face::L, Face::F, Face::B];
    let mut rng = rand::thread_rng();
    let mut last_face: Option<Face> = None;

    for _ in 0..20 {
        let face = loop {
            let f = faces[rng.gen_range(0..6)];
            if last_face.map_or(true, |last| !are_opposite(last, f)) {
                break f;
            }
        };
        let inverse = rng.gen_bool(0.5);
        queue.queue.push_back(MoveCommand {
            face,
            inverse,
            is_scramble: true,
        });
        last_face = Some(face);
    }
}

fn are_opposite(a: Face, b: Face) -> bool {
    matches!((a, b),
        (Face::U, Face::D) | (Face::D, Face::U) |
        (Face::R, Face::L) | (Face::L, Face::R) |
        (Face::F, Face::B) | (Face::B, Face::F)
    )
}

fn rotate_logical(pos: IVec3, axis: Vec3, angle: f32) -> IVec3 {
    let v = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
    let r = Quat::from_axis_angle(axis, angle) * v;
    IVec3::new(r.x.round() as i32, r.y.round() as i32, r.z.round() as i32)
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}
