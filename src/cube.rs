use bevy::prelude::*;
use rand::Rng;
use std::collections::VecDeque;

pub const CUBIE_SIZE: f32 = 0.9;
pub const ADESIVO_ESPESSURA: f32 = 0.02;
pub const ADESIVO_TAMANHO: f32 = 0.8;
pub const ADESIVO_OFFSET: f32 = 0.46;
pub const ANIM_DURATION_USER: f32 = 0.12;
pub const ANIM_DURATION_SCRAMBLE: f32 = 0.045;

pub struct CubePlugin;

impl Plugin for CubePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<RotationState>()
            .init_resource::<MoveQueue>()
            .init_resource::<MoveHistory>()
            .init_resource::<GameTimerState>()
            .init_resource::<ScrambleInfo>()
            .add_systems(Startup, spawn_cube)
            .add_systems(Update, (cube_system, keyboard_input_system));
    }
}

#[derive(Resource, Default)]
pub struct ScrambleInfo {
    pub sequence: String,
}

#[derive(Resource, Default)]
pub struct GameTimerState {
    pub is_scrambled: bool,
    pub is_running: bool,
    pub elapsed: f32,
    pub move_count: u32,
    pub is_solved: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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

#[derive(Clone, Copy, Debug)]
pub struct MoveCommand {
    pub axis: Vec3,
    pub layer: i32,
    pub angle: f32,
    pub is_scramble: bool,
    pub face: Option<Face>,
}

impl MoveCommand {
    pub fn from_face(face: Face, inverse: bool, is_scramble: bool) -> Self {
        Self {
            axis: face.axis(),
            layer: face.layer(),
            angle: face.target_angle(inverse),
            is_scramble,
            face: Some(face),
        }
    }

    pub fn slice(axis: Vec3, layer: i32, angle: f32, is_scramble: bool) -> Self {
        let face = if axis == Vec3::Y && layer == 1 {
            Some(Face::U)
        } else if axis == Vec3::Y && layer == -1 {
            Some(Face::D)
        } else if axis == Vec3::X && layer == 1 {
            Some(Face::R)
        } else if axis == Vec3::X && layer == -1 {
            Some(Face::L)
        } else if axis == Vec3::Z && layer == 1 {
            Some(Face::F)
        } else if axis == Vec3::Z && layer == -1 {
            Some(Face::B)
        } else {
            None
        };

        Self {
            axis,
            layer,
            angle,
            is_scramble,
            face,
        }
    }

    pub fn invert(&self) -> Self {
        Self {
            axis: self.axis,
            layer: self.layer,
            angle: -self.angle,
            is_scramble: false,
            face: self.face,
        }
    }
}

#[derive(Resource, Default)]
pub struct MoveQueue {
    pub queue: VecDeque<MoveCommand>,
}

#[derive(Resource, Default)]
pub struct MoveHistory {
    pub history: Vec<MoveCommand>,
}

#[derive(Component)]
pub struct Cubie {
    pub logical_pos: IVec3,
    pub initial_pos: IVec3,
}

#[derive(Component)]
pub struct CubeRoot;

#[derive(Resource)]
pub struct CubeAssets {
    pub base_mesh: Handle<Mesh>,
    pub sticker_meshes: StickerMeshes,
    pub materials: CubeMaterials,
}

pub struct StickerMeshes {
    pub x: Handle<Mesh>,
    pub y: Handle<Mesh>,
    pub z: Handle<Mesh>,
}

#[derive(Resource)]
pub struct CubeMaterials {
    pub base: Handle<StandardMaterial>,
    pub direita: Handle<StandardMaterial>,
    pub esquerda: Handle<StandardMaterial>,
    pub cima: Handle<StandardMaterial>,
    pub baixo: Handle<StandardMaterial>,
    pub frente: Handle<StandardMaterial>,
    pub tras: Handle<StandardMaterial>,
}

#[derive(Resource, Default)]
pub struct RotationState {
    pub anim: Option<RotationAnim>,
}

pub struct RotationAnim {
    pub axis: Vec3,
    pub target_angle: f32,
    pub elapsed: f32,
    pub duration: f32,
    pub entries: Vec<AnimEntry>,
    pub current_cmd: MoveCommand,
}

pub struct AnimEntry {
    pub entity: Entity,
    pub start_pos: Vec3,
    pub start_rot: Quat,
    pub logical_pos: IVec3,
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
        StandardMaterial {
            base_color: color,
            unlit: true,
            ..default()
        }
    }

    let base_mat  = materials.add(mat(Color::rgb(0.08, 0.08, 0.08)));
    let mat_direita  = materials.add(mat(Color::rgb(0.85, 0.12, 0.12))); // Vermelho
    let mat_esquerda = materials.add(mat(Color::rgb(1.0, 0.45, 0.0)));  // Laranja
    let mat_cima     = materials.add(mat(Color::rgb(0.95, 0.95, 0.95))); // Branco
    let mat_baixo    = materials.add(mat(Color::rgb(1.0, 0.85, 0.0)));  // Amarelo
    let mat_frente   = materials.add(mat(Color::rgb(0.0, 0.78, 0.28))); // Verde
    let mat_tras     = materials.add(mat(Color::rgb(0.0, 0.38, 0.95))); // Azul

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

pub fn trigger_scramble(
    queue: &mut MoveQueue,
    timer_state: &mut GameTimerState,
    scramble_info: &mut ScrambleInfo,
) {
    queue.queue.clear();
    timer_state.is_scrambled = true;
    timer_state.is_running = false;
    timer_state.is_solved = false;
    timer_state.elapsed = 0.0;
    timer_state.move_count = 0;
    queue_wca_scramble(queue, scramble_info);
}

pub fn trigger_reset(
    queue: &mut MoveQueue,
    history: &mut MoveHistory,
    state: &mut RotationState,
    timer_state: &mut GameTimerState,
    scramble_info: &mut ScrambleInfo,
    cubies: &mut Query<(Entity, &mut Transform, &mut Cubie)>,
) {
    queue.queue.clear();
    history.history.clear();
    state.anim = None;
    timer_state.is_scrambled = false;
    timer_state.is_running = false;
    timer_state.is_solved = false;
    timer_state.elapsed = 0.0;
    timer_state.move_count = 0;
    scramble_info.sequence.clear();
    for (_, mut transform, mut cubie) in cubies.iter_mut() {
        cubie.logical_pos = cubie.initial_pos;
        transform.translation = cubie.initial_pos.as_vec3();
        transform.rotation = Quat::IDENTITY;
    }
}

pub fn trigger_undo(
    queue: &mut MoveQueue,
    history: &mut MoveHistory,
    state: &RotationState,
) {
    if state.anim.is_some() || !queue.queue.is_empty() {
        return;
    }
    if let Some(last_move) = history.history.pop() {
        queue.queue.push_back(last_move.invert());
    }
}

fn keyboard_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut queue: ResMut<MoveQueue>,
    mut history: ResMut<MoveHistory>,
    mut state: ResMut<RotationState>,
    mut timer_state: ResMut<GameTimerState>,
    mut scramble_info: ResMut<ScrambleInfo>,
    mut cubies: Query<(Entity, &mut Transform, &mut Cubie)>,
) {
    let inverse = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if let Some(face) = check_face_keys(&keys) {
        queue.queue.push_back(MoveCommand::from_face(face, inverse, false));
    }

    if keys.just_pressed(KeyCode::KeyS) {
        trigger_scramble(&mut queue, &mut timer_state, &mut scramble_info);
    }

    if keys.just_pressed(KeyCode::KeyX) {
        trigger_reset(&mut queue, &mut history, &mut state, &mut timer_state, &mut scramble_info, &mut cubies);
    }

    if keys.just_pressed(KeyCode::KeyZ) || (keys.pressed(KeyCode::ControlLeft) && keys.just_pressed(KeyCode::KeyZ)) {
        trigger_undo(&mut queue, &mut history, &state);
    }
}

fn cube_system(
    time: Res<Time>,
    mut queue: ResMut<MoveQueue>,
    mut history: ResMut<MoveHistory>,
    mut state: ResMut<RotationState>,
    mut timer_state: ResMut<GameTimerState>,
    mut cubies: Query<(Entity, &mut Transform, &mut Cubie)>,
) {
    if timer_state.is_running {
        timer_state.elapsed += time.delta_seconds();
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

    // Processa a interpolação suave da rotação ativa
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
            let cmd = anim.current_cmd;
            for entry in &anim.entries {
                if let Ok((_, mut transform, mut cubie)) = cubies.get_mut(entry.entity) {
                    let new_pos = rotate_logical(entry.logical_pos, anim.axis, anim.target_angle);
                    cubie.logical_pos = new_pos;
                    transform.translation = new_pos.as_vec3();
                    transform.rotation = (Quat::from_axis_angle(anim.axis, anim.target_angle) * entry.start_rot).normalize();
                }
            }

            // Registra no histórico para permitir Undo (se não for scramble)
            if !cmd.is_scramble {
                history.history.push(cmd);
            }

            state.anim = None;

            // Se terminou o movimento e o cubo está em jogo cronometrado, verifica se resolveu
            if timer_state.is_running && queue.queue.is_empty() {
                let solved = cubies.iter().all(|(_, transform, cubie)| {
                    let p = cubie.initial_pos;
                    let non_zero_count = (p.x != 0) as i32 + (p.y != 0) as i32 + (p.z != 0) as i32;
                    if non_zero_count >= 2 {
                        // Cantos e arestas: posição lógica e rotação devem ser exatas
                        cubie.logical_pos == cubie.initial_pos && transform.rotation.dot(Quat::IDENTITY).abs() > 0.99
                    } else {
                        // Centros e miolo: posição lógica deve ser exata
                        cubie.logical_pos == cubie.initial_pos
                    }
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
    let axis = cmd.axis;
    let layer = cmd.layer;
    let target_angle = cmd.angle;
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
        current_cmd: cmd,
    });
}

/// Gera um scramble oficial no padrão WCA (20 movimentos sem repetições imediatas na mesma face/eixo)
fn queue_wca_scramble(queue: &mut MoveQueue, scramble_info: &mut ScrambleInfo) {
    let faces = [Face::U, Face::D, Face::R, Face::L, Face::F, Face::B];
    let mut rng = rand::thread_rng();
    let mut last_face: Option<Face> = None;
    let mut second_last_face: Option<Face> = None;
    let mut notation_strings = Vec::new();

    for _ in 0..20 {
        let face = loop {
            let f = faces[rng.gen_range(0..6)];
            // Não repete a mesma face consecutiva
            if Some(f) == last_face {
                continue;
            }
            // Não repete padrões redundantes no mesmo eixo (ex: U D U)
            if let (Some(l), Some(sl)) = (last_face, second_last_face) {
                if are_same_axis(f, l) && are_same_axis(l, sl) {
                    continue;
                }
            }
            break f;
        };

        // Sorteia tipo de movimento: 0 = Normal, 1 = Prime (inverso), 2 = Duplo (180 graus)
        let move_type = rng.gen_range(0..3);
        let (inverse, is_double, suffix) = match move_type {
            0 => (false, false, ""),
            1 => (true, false, "'"),
            _ => (false, true, "2"),
        };

        let notation = format!("{:?}{}", face, suffix);
        notation_strings.push(notation);

        if is_double {
            // Movimento duplo: 2 giros consecutivos de 90 graus
            queue.queue.push_back(MoveCommand::from_face(face, false, true));
            queue.queue.push_back(MoveCommand::from_face(face, false, true));
        } else {
            queue.queue.push_back(MoveCommand::from_face(face, inverse, true));
        }

        second_last_face = last_face;
        last_face = Some(face);
    }

    scramble_info.sequence = notation_strings.join(" ");
}

fn are_same_axis(a: Face, b: Face) -> bool {
    a.axis() == b.axis()
}

fn rotate_logical(pos: IVec3, axis: Vec3, angle: f32) -> IVec3 {
    let v = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
    let r = Quat::from_axis_angle(axis, angle) * v;
    IVec3::new(r.x.round() as i32, r.y.round() as i32, r.z.round() as i32)
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}
