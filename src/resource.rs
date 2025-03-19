use bevy::asset::Handle;
use bevy::prelude::{Font, Resource};

#[derive(Resource)]
pub struct AsciiFont(pub Handle<Font>);
#[derive(Resource)]
pub struct AsciiBoldFont(pub Handle<Font>);
#[derive(Resource, Clone, Copy)]
pub struct WindowSize {
    pub width: f32,
    pub height: f32,
}