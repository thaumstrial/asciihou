use std::thread::sleep;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::asset::io::Reader;
use bevy::color::palettes::basic::*;
use bevy::color::palettes::css::ORANGE;
use bevy::prelude::*;
use bevy::text::cosmic_text::ttf_parser::Weight;
use serde::Deserialize;
use thiserror::Error;

#[derive(Component)]
pub struct AsciiChar {
    pub pos: UVec2,
}
#[derive(Component)]
pub struct AsciiAnimation {
    frames: Vec<(char, Color)>,
    current_frame: usize,
    frame_num: usize,
    frame_size: UVec2,
    frame_time: Timer,
}
impl AsciiAnimation {
    pub fn get_ascii_char_at(&self, pos: &UVec2) -> (char, Color) {
        self.frames[(pos.x as usize + pos.y as usize * self.frame_size.x as usize) + (self.current_frame * self.frame_size.x as usize * self.frame_size.y as usize)]
    }
}

#[derive(Default)]
struct AsciiAnimationLoader;
#[non_exhaustive]
#[derive(Error, Debug)]
enum CustomAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}
impl AssetLoader for AsciiAnimationLoader {
    type Asset = AsciiAnimationAsset;
    type Settings = ();
    type Error = CustomAssetLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let custom_asset = ron::de::from_bytes::<AsciiAnimationAsset>(&bytes)?;
        Ok(custom_asset)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

#[derive(Asset, Deserialize, TypePath)]
pub struct AsciiAnimationAsset {
    pub frames: Vec<(char, String)>,
    pub frame_size: UVec2,
    pub frame_num: usize,
    pub frame_time: f32,
}
fn color_from_hex(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
    Color::srgb_u8(r, g, b)
}

#[derive(Resource)]
pub struct MainMenuAnimation(pub Handle<AsciiAnimationAsset>);
impl AsciiAnimationAsset {
    fn get_component(&self) -> AsciiAnimation {
        let frames: Vec<(char, Color)> = self.frames
            .iter()
            .map(|(ch,hex)| (*ch, color_from_hex(hex)))
            .collect();

        AsciiAnimation {
            frames,
            current_frame: 0,
            frame_num: self.frame_num,
            frame_size: self.frame_size,
            frame_time: Timer::from_seconds(self.frame_time, TimerMode::Repeating),
        }
    }
}
fn setup_animation(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let main_menu_animation: Handle<AsciiAnimationAsset> = asset_server.load("ascii/animation/test.ron");
    commands.insert_resource(MainMenuAnimation(main_menu_animation));
}
pub fn spawn_ascii_animation(
    commands: &mut Commands,
    animation_asset: &AsciiAnimationAsset,
    font: &Handle<Font>,
    font_size: f32,
    transform: Transform,
) {
    let char_width = font_size * 0.6;
    let char_height = font_size;
    let frame_size = animation_asset.frame_size;
    let animation_component = animation_asset.get_component();

    let offset_x = -((frame_size.x as f32) * char_width) / 2.0;
    let offset_y = ((frame_size.y as f32) * char_height) / 2.0;

    let mut parent_entity = commands.spawn((
        Transform::from(transform),
        InheritedVisibility::default(),
    ));

    parent_entity.with_children(|parent| {
        for x in 0..frame_size.x {
            for y in 0..frame_size.y {
                let pos = UVec2::new(x, y);
                let (ch, color) = animation_component.get_ascii_char_at(&pos);
                let pos_x = x as f32 * char_width + offset_x;
                let pos_y = -(y as f32 * char_height) + offset_y;

                parent.spawn((
                    Text2d::new(ch.to_string()),
                    TextFont {
                        font: font.clone(),
                        font_size,
                        ..default()
                    },
                    AsciiChar { pos },
                    TextLayout::new_with_justify(JustifyText::Center),
                    Transform::from_translation(Vec3::new(pos_x, pos_y, 0.0)),
                    TextColor(color),
                ));
            }
        }
    });

    parent_entity.insert(animation_component);
}
fn play_ascii_animation(
    time: Res<Time>,
    mut animation_query: Query<(&Children, &mut AsciiAnimation)>,
    mut ascii_chars: Query<(&AsciiChar, &mut Text2d, &mut TextColor)>,
) {
    for (children, mut animation) in animation_query.iter_mut() {

        animation.frame_time.tick(time.delta());

        if animation.frame_time.just_finished() {
            animation.current_frame = (animation.current_frame + 1) % animation.frame_num;
            for &child in children.iter() {
                if let Ok((ascii_char, mut text, mut text_color)) = ascii_chars.get_mut(child) {
                    let (ch, color) = animation.get_ascii_char_at(&ascii_char.pos);
                    text.0 = ch.to_string();
                    text_color.0 = color;
                }
            }
        }
    }
}
pub struct AsciiAnimationPlugin;
impl Plugin for AsciiAnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_asset::<AsciiAnimationAsset>()
            .init_asset_loader::<AsciiAnimationLoader>()
            .add_systems(Startup, setup_animation)
            .add_systems(Update, play_ascii_animation);
    }
}