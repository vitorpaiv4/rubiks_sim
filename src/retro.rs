use bevy::prelude::*;
use bevy::render::render_resource::*;
use bevy::render::render_asset::RenderAssetUsages;

pub struct RetroPlugin;

impl Plugin for RetroPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_retro_overlay);
    }
}

fn gen_scanline_tex() -> Image {
    let w = 640u32;
    let h = 480u32;
    let mut pixels = vec![0u8; (w * h * 4) as usize];

    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 4) as usize;
            pixels[i + 3] = if y % 2 == 0 { 55 } else { 0 };
        }
    }

    Image::new(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD,
    )
}

fn setup_retro_overlay(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let tex = gen_scanline_tex();
    let handle = images.add(tex);

    commands.spawn(ImageBundle {
        image: UiImage::new(handle),
        style: Style {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        z_index: ZIndex::Global(i32::MAX),
        ..default()
    });
}
