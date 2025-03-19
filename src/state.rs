use bevy::prelude::StateSet;
use bevy::prelude::{States, SubStates};

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    MainMenu,
    // #[default]
    InGame,
}
#[derive(SubStates, Debug, Clone, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::InGame)]
pub enum GameState {
    #[default]
    Running,
    Paused,
}