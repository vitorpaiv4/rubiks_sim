use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, configurar_cena)
        .add_systems(Update, girar_camera) // Agora giramos a câmera em vez do cubo
        .run();
}

// 1. ADICIONADO: Uma etiqueta para identificarmos a câmera depois
#[derive(Component)]
struct CameraOrbit;

fn configurar_cena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 2. OTIMIZAÇÃO: Criamos o formato e a cor apenas UMA VEZ na memória.
    // Usar tamanho 0.9 cria um espaçamento visual (gap) entre as peças.
    let mesh_cubie = meshes.add(Cuboid::new(0.9, 0.9, 0.9));
    let material_cubie = materials.add(Color::rgb(0.15, 0.15, 0.15)); // Cinza escuro (o esqueleto do cubo)

    // 3. A MÁGICA DOS 3 EIXOS: -1 (Esquerda/Baixo/Trás), 0 (Meio), 1 (Direita/Cima/Frente)
    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                commands.spawn(PbrBundle {
                    // O clone() aqui não copia o objeto 3D inteiro, apenas a referência para a memória do Bevy
                    mesh: mesh_cubie.clone(),
                    material: material_cubie.clone(),
                    transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                    ..default()
                });
            }
        }
    }

    // 4. LUZ: Mudamos para luz direcional (como a luz do Sol) para iluminar todo o objeto de forma uniforme
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(5.0, 8.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            illuminance: 3000.0, 
            shadows_enabled: true,
            ..default()
        },
        ..default()
    });

    // 5. CÂMERA: Mais afastada para caber tudo, e com a nossa etiqueta
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 5.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraOrbit, // Colando a etiqueta
    ));
}

// 6. ADICIONADO: O sistema para girar a câmera suavemente ao redor do centro
fn girar_camera(time: Res<Time>, mut query: Query<&mut Transform, With<CameraOrbit>>) {
    for mut transform in &mut query {
        // Criamos uma rotação no eixo Y com base no tempo
        let rotacao = Quat::from_rotation_y(0.5 * time.delta_seconds());
        
        // Multiplicar a rotação pela posição faz ela "orbitar" em vez de girar no próprio eixo
        transform.translation = rotacao * transform.translation;
        
        // Mantém a câmera sempre olhando para o centro do mundo (onde está o cubo)
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}