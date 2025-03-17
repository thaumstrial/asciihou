use bevy::color::palettes::basic::{GRAY, WHITE};
use bevy::prelude::*;
use crate::{AppState, AsciiFont, GameState, PlayerBombsText, PlayerGrazeText, PlayerLivesText, PlayerPointsText, PlayerPowersText, WindowSize};

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::MainMenu)]
enum MainMenuState {
    #[default]
    Start,
    ExtraStart,
    PracticeStart,
    Replay,
    Score,
    MusicRoom,
    Option,
    Quit,
}
#[derive(Component)]
struct MainMenuEntry(MainMenuState);

#[derive(Component)]
struct StartText;
#[derive(Component)]
struct ExtraStartText;
#[derive(Component)]
struct PracticeStartText;
#[derive(Component)]
struct ReplayText;
#[derive(Component)]
struct ScoreText;
#[derive(Component)]
struct MusicRoomText;
#[derive(Component)]
struct OptionText;
#[derive(Component)]
struct QuitText;

fn setup_main_menu(
    mut commands: Commands,
    font: Res<AsciiFont>,
) {
    let font_size = 40.0;
    let text_font = TextFont {
        font: font.0.clone(),
        font_size: font_size.clone(),
        ..default()
    };

    commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|parent| {
            let menu_items = vec![
                (MainMenuState::Start, "Start"),
                (MainMenuState::ExtraStart, "Extra Start"),
                (MainMenuState::PracticeStart, "Practice Start"),
                (MainMenuState::Replay, "Replay"),
                (MainMenuState::Score, "Score"),
                (MainMenuState::MusicRoom, "Music Room"),
                (MainMenuState::Option, "Option"),
                (MainMenuState::Quit, "Quit"),
            ];

            for (i, (state, label)) in menu_items.iter().enumerate() {
                parent.spawn((
                    Node {
                        left: Val::Px((menu_items.len() - i) as f32 * font_size),
                        ..default()
                    }
                )).with_children(|parent| {
                    parent.spawn((
                        MainMenuEntry(*state),
                        Text::new(*label),
                        text_font.clone(),
                        TextLayout::new_with_justify(JustifyText::Left),
                        TextColor(Color::Srgba(WHITE)),
                    ));
                });
            }
        });
}

fn setup_in_game(
    mut commands: Commands,
    font: Res<AsciiFont>,
    window: Res<WindowSize>,
) {
    let font_size = 40.0;
    let text_font = TextFont {
        font: font.0.clone(),
        font_size: font_size.clone(),
        ..default()
    };

    let width = window.width;
    let height = window.height;

    let horizontal_line = format!(
        "+{}+{}+",
        "-".repeat((width / font_size * 1.9 * 0.65 - 1.0).floor() as usize),
        "-".repeat((width / font_size * 1.9 * 0.35).floor() as usize)
    );
    let vertical_margin = 20.0;

    commands.spawn((
        Text2d::new(horizontal_line.clone()),
        text_font.clone(),
        TextLayout::default(),
        TextColor(Color::Srgba(GRAY)),
        Transform::from_translation(Vec3::new(0.0, height / 2.0 - vertical_margin, 1.0)),
    ));
    commands.spawn((
        Text2d::new(horizontal_line.clone()),
        text_font.clone(),
        TextLayout::default(),
        TextColor(Color::Srgba(GRAY)),
        Transform::from_translation(Vec3::new(0.0, -height / 2.0 + vertical_margin, 1.0)),
    ));

    let vertical_line = "|\n".repeat((height / font_size / 1.2).floor() as usize);
    let horizontal_margin = 30.0;

    commands.spawn((
        Text2d::new(vertical_line.clone()),
        text_font.clone(),
        TextLayout::default(),
        TextColor(Color::Srgba(GRAY)),
        Transform::from_translation(Vec3::new(width / 2.0 - horizontal_margin, 0.0, 1.0)),
    ));
    commands.spawn((
        Text2d::new(vertical_line.clone()),
        text_font.clone(),
        TextLayout::default(),
        TextColor(Color::Srgba(GRAY)),
        Transform::from_translation(Vec3::new(-width / 2.0 + horizontal_margin, 0.0, 1.0)),
    ));
    commands.spawn((
        Text2d::new(vertical_line.clone()),
        text_font.clone(),
        TextLayout::default(),
        TextColor(Color::Srgba(GRAY)),
        Transform::from_translation(Vec3::new(width / 2.0 * 0.219 + horizontal_margin, 0.0, 1.0)),
    ));

    let info_margin = width / 2.0 * 0.4;
    commands.spawn((
        Text2d::new(""),
        text_font.clone(),
        TextLayout::default(),
        TextColor(Color::Srgba(WHITE)),

        Transform::from_translation(Vec3::new(info_margin, height / 2.0 * 0.25, 1.0)),
        PlayerLivesText,
    ));
    commands.spawn((
        Text2d::new(""),
        text_font.clone(),
        TextLayout::default(),
        TextColor(Color::Srgba(WHITE)),

        Transform::from_translation(Vec3::new(info_margin, height / 2.0 * 0.25 - font_size * 1.5, 1.0)),
        PlayerBombsText,
    ));

    commands.spawn((
        Text2d::new(""),
        text_font.clone(),
        TextLayout::default(),
        TextColor(Color::Srgba(WHITE)),

        Transform::from_translation(Vec3::new(info_margin, height / 2.0 * 0.25 - font_size * 3.5, 1.0)),
        PlayerPowersText,
    ));
    commands.spawn((
        Text2d::new(""),
        text_font.clone(),
        TextLayout::default(),
        TextColor(Color::Srgba(WHITE)),

        Transform::from_translation(Vec3::new(info_margin, height / 2.0 * 0.25 - font_size * 5.0, 1.0)),
        PlayerGrazeText,
    ));
    commands.spawn((
        Text2d::new(""),
        text_font.clone(),
        TextLayout::default(),
        TextColor(Color::Srgba(WHITE)),

        Transform::from_translation(Vec3::new(info_margin, height / 2.0 * 0.25 - font_size * 6.5, 1.0)),
        PlayerPointsText,
    ));
}

pub struct GameUiPlugin;
impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_sub_state::<MainMenuState>()
            .add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
            .add_systems(OnEnter(AppState::InGame), setup_in_game);
    }
}