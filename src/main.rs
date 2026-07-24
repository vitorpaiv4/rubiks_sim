use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins) // Adiciona a janela, teclado, renderizador 3D, etc.
        .add_systems(Startup, configurar_cena) // Roda uma vez no início
        .add_systems(Update, girar_cubo) // Roda em loop infinito (60 FPS)
        .run();
}

// Criamos um Componente (uma etiqueta) para identificar quem é o nosso cubo
#[derive(Component)]
struct MeuCubo;

// A função de setup recebe "Comandos" para criar coisas no mundo, 
// e acesso às "Malhas" (formatos 3D) e "Materiais" (cores/texturas).
fn configurar_cena(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 1. Criando o Cubo
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::rgb(0.8, 0.1, 0.1)), // Vermelho meio retrô
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        MeuCubo, // Colocamos nossa etiqueta nele!
    ));

    // 2. Criando uma Luz para enxergarmos a cor e as sombras
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // 3. Criando a Câmera 3D
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

// Um Sistema que pega o tempo que passou (Time) e a Posição (Transform) de tudo que tem a etiqueta 'MeuCubo'
fn girar_cubo(time: Res<Time>, mut query: Query<&mut Transform, With<MeuCubo>>) {
    for mut transform in &mut query {
        // Gira o cubo nos eixos Y e X suavemente
        transform.rotate_y(1.0 * time.delta_seconds());
        transform.rotate_x(0.5 * time.delta_seconds());
    }
}