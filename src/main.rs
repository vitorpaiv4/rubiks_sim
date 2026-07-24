use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, configurar_cena)
        .add_systems(Update, girar_camera)
        .run();
}

#[derive(Component)]
struct CameraOrbit;

fn configurar_cena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 1. O ESQUELETO (O bloco principal cinza escuro)
    let malha_base = meshes.add(Cuboid::new(0.9, 0.9, 0.9));
    let material_base = materials.add(Color::rgb(0.1, 0.1, 0.1));

    // 2. AS CORES CLÁSSICAS DO CUBO MÁGICO
    let mat_direita = materials.add(Color::rgb(0.8, 0.1, 0.1)); // Vermelho
    let mat_esquerda = materials.add(Color::rgb(1.0, 0.4, 0.0)); // Laranja
    let mat_cima = materials.add(Color::rgb(1.0, 1.0, 1.0));    // Branco
    let mat_baixo = materials.add(Color::rgb(0.9, 0.9, 0.0));   // Amarelo
    let mat_frente = materials.add(Color::rgb(0.0, 0.7, 0.2));  // Verde
    let mat_tras = materials.add(Color::rgb(0.0, 0.2, 0.8));    // Azul

    // 3. AS MALHAS DOS ADESIVOS (Blocos muito fininhos, com 0.02 de espessura)
    let adesivo_x = meshes.add(Cuboid::new(0.02, 0.8, 0.8)); // Adesivo para as laterais
    let adesivo_y = meshes.add(Cuboid::new(0.8, 0.02, 0.8)); // Adesivo para cima/baixo
    let adesivo_z = meshes.add(Cuboid::new(0.8, 0.8, 0.02)); // Adesivo para frente/trás

    for x in -1..=1 {
        for y in -1..=1 {
            for z in -1..=1 {
                // Criamos o bloco base (Pai)
                commands.spawn(PbrBundle {
                    mesh: malha_base.clone(),
                    material: material_base.clone(),
                    transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                    ..default()
                })
                .with_children(|pai| {
                    // Aqui dentro nascem os filhos (adesivos)! 
                    // Só colamos o adesivo se a face do bloco estiver na borda do cubo mágico.

                    // Faces no eixo X (Direita e Esquerda)
                    if x == 1 {
                        pai.spawn(PbrBundle {
                            mesh: adesivo_x.clone(),
                            material: mat_direita.clone(),
                            transform: Transform::from_xyz(0.46, 0.0, 0.0), // 0.46 joga o adesivo um pouquinho pra fora da face do bloco
                            ..default()
                        });
                    } else if x == -1 {
                        pai.spawn(PbrBundle {
                            mesh: adesivo_x.clone(),
                            material: mat_esquerda.clone(),
                            transform: Transform::from_xyz(-0.46, 0.0, 0.0),
                            ..default()
                        });
                    }

                    // Faces no eixo Y (Cima e Baixo)
                    if y == 1 {
                        pai.spawn(PbrBundle {
                            mesh: adesivo_y.clone(),
                            material: mat_cima.clone(),
                            transform: Transform::from_xyz(0.0, 0.46, 0.0),
                            ..default()
                        });
                    } else if y == -1 {
                        pai.spawn(PbrBundle {
                            mesh: adesivo_y.clone(),
                            material: mat_baixo.clone(),
                            transform: Transform::from_xyz(0.0, -0.46, 0.0),
                            ..default()
                        });
                    }

                    // Faces no eixo Z (Frente e Trás)
                    if z == 1 {
                        pai.spawn(PbrBundle {
                            mesh: adesivo_z.clone(),
                            material: mat_frente.clone(),
                            transform: Transform::from_xyz(0.0, 0.0, 0.46),
                            ..default()
                        });
                    } else if z == -1 {
                        pai.spawn(PbrBundle {
                            mesh: adesivo_z.clone(),
                            material: mat_tras.clone(),
                            transform: Transform::from_xyz(0.0, 0.0, -0.46),
                            ..default()
                        });
                    }
                });
            }
        }
    }

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(5.0, 8.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        directional_light: DirectionalLight {
            illuminance: 3000.0, 
            shadows_enabled: true,
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
}

fn girar_camera(time: Res<Time>, mut query: Query<&mut Transform, With<CameraOrbit>>) {
    for mut transform in &mut query {
        let rotacao = Quat::from_rotation_y(0.5 * time.delta_seconds());
        transform.translation = rotacao * transform.translation;
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}