use bevy::color::palettes::basic::{GRAY, WHITE};
use bevy::input::common_conditions::{input_just_pressed};
use bevy::prelude::*;
use crate::{AppState, AsciiFont, PlayerBombsText, PlayerGrazeText, PlayerLivesText, PlayerPointsText, PlayerPowersText, WindowSize};

#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(AppState = AppState::MainMenu)]
enum MainMenuState {
    #[default]
    Choosing,
    Start,
    ExtraStart,
    PracticeStart,
    Replay,
    Score,
    MusicRoom,
    Option,
    Quit,
}
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(MainMenuState = MainMenuState::Start)]
enum StartState {
    #[default]
    Difficulty,
    Character,
    SpellCard,
}
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(StartState = StartState::Difficulty)]
enum DifficultyState {
    #[default]
    Easy,
    Normal,
    Hard,
    Lunatic
}
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(StartState = StartState::Character)]
enum CharacterState {
    #[default]
    ReimuHakurei,
    MarisaKirisame,
}
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(CharacterState = CharacterState::ReimuHakurei)]
enum ReimuSpellCardState {
    #[default]
    SpellA,
    SpellB,
}
#[derive(SubStates, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[source(CharacterState = CharacterState::MarisaKirisame)]
enum MarisaSpellCardState {
    #[default]
    SpellC,
    SpellD,
}
#[derive(Resource)]
struct SelectedSpellCard {
    selected_index: usize,
    repeat_timer: Timer,
}
#[derive(Resource)]
struct SelectedCharacter {
    selected: CharacterState,
    repeat_timer: Timer,
}
#[derive(Resource)]
struct SelectedMenuEntry {
    selected: MainMenuState,
    repeat_timer: Timer,
}
#[derive(Resource)]
struct SelectedDifficulty {
    selected: DifficultyState,
    repeat_timer: Timer,
}
#[derive(Component)]
struct ReimuSpellCardEntry;
#[derive(Component)]
struct MarisaSpellCardEntry;
#[derive(Component)]
struct SpellCardEntryIndex(usize);
#[derive(Component)]
struct SpellCardContainer;
#[derive(Component)]
struct CharacterContainer;
#[derive(Component)]
struct DifficultyContainer;
#[derive(Component)]
struct MainMenuEntry(MainMenuState);
#[derive(Component)]
struct DifficultyEntry(DifficultyState);
#[derive(Component)]
struct CharacterEntry(CharacterState);
fn setup_start(
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
        .spawn((
            StateScoped(MainMenuState::Start),
            Node {
                justify_content: JustifyContent::SpaceEvenly,

                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            }))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        // justify_content: JustifyContent::FlexStart,
                        // align_items: AlignItems::FlexStart,
                        width: Val::Percent(100.0),
                        margin: UiRect {
                            left: Val::Px(font_size),
                            top: Val::Px(font_size),
                            ..default()
                        },
                        ..default()
                    },
                    DifficultyContainer
                ));

            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        // justify_content: JustifyContent::FlexStart,
                        // align_items: AlignItems::Center,
                        width: Val::Percent(100.0),
                        margin: UiRect {
                            top: Val::Px(font_size),
                            ..default()
                        },
                        ..default()
                    },
                    CharacterContainer,
                ));

            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        // justify_content: JustifyContent::FlexStart,
                        // align_items: AlignItems::Center,
                        width: Val::Percent(100.0),
                        margin: UiRect {
                            top: Val::Px(font_size),
                            ..default()
                        },
                        ..default()
                    },
                    SpellCardContainer
                ));
        });
}
fn setup_character(
    mut commands: Commands,
    font: Res<AsciiFont>,
    character_container: Query<Entity, With<CharacterContainer>>,
    spell_card_container: Query<Entity, With<SpellCardContainer>>,
) {
    if let Ok(entity) = character_container.get_single() {
        commands.entity(entity).despawn_descendants();
    }
    if let Ok(entity) = spell_card_container.get_single() {
        commands.entity(entity).despawn_descendants();
    }

    let font_size = 40.0;
    let text_font = TextFont {
        font: font.0.clone(),
        font_size,
        ..default()
    };

    if let Ok(container_entity) = character_container.get_single() {
        commands.entity(container_entity).with_children(|parent| {
            parent.spawn((
                Text::new("Character:"),
                text_font.clone(),
                TextLayout::new_with_justify(JustifyText::Left),
                TextColor(Color::Srgba(WHITE)),
            ));

            parent.spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            }).with_children(|parent| {
                let characters = vec![
                    (CharacterState::ReimuHakurei, "[ ] Reimu Hakurei"),
                    (CharacterState::MarisaKirisame, "[ ] Marisa Kirisame"),
                ];

                for (entry, label) in characters {
                    parent.spawn(Node { ..default() }).with_children(|parent| {
                        parent.spawn((
                            CharacterEntry(entry),
                            Text::new(label),
                            text_font.clone(),
                            TextLayout::new_with_justify(JustifyText::Left),
                            TextColor(Color::Srgba(WHITE)),
                        ));
                    });
                }
            });
        });
    }
}
fn setup_difficulty(
    mut commands: Commands,
    font: Res<AsciiFont>,
    difficulty_container: Query<Entity, With<DifficultyContainer>>,
    character_container: Query<Entity, With<CharacterContainer>>,
) {
    if let Ok(entity) = difficulty_container.get_single() {
        commands.entity(entity).despawn_descendants();
    }
    if let Ok(entity) = character_container.get_single() {
        commands.entity(entity).despawn_descendants();
    }

    let font_size = 40.0;
    let text_font = TextFont {
        font: font.0.clone(),
        font_size,
        ..default()
    };

    if let Ok(container_entity) = difficulty_container.get_single() {
        commands.entity(container_entity).with_children(|parent| {
            parent.spawn((
                Text::new("Difficulty:"),
                text_font.clone(),
                TextLayout::new_with_justify(JustifyText::Left),
                TextColor(Color::Srgba(WHITE)),
            ));

            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|parent| {
                    let difficulties = vec![
                        (DifficultyState::Easy, "[ ] Easy"),
                        (DifficultyState::Normal, "[ ] Normal"),
                        (DifficultyState::Hard, "[ ] Hard"),
                        (DifficultyState::Lunatic, "[ ] Lunatic"),
                    ];

                    for (entry, label) in difficulties {
                        parent.spawn(Node { ..default() }).with_children(|parent| {
                            parent.spawn((
                                DifficultyEntry(entry),
                                Text::new(label),
                                text_font.clone(),
                                TextLayout::new_with_justify(JustifyText::Left),
                                TextColor(Color::Srgba(WHITE)),
                            ));
                        });
                    }
                });
        });
    }
}
fn setup_spell_cards(
    mut commands: Commands,
    font: Res<AsciiFont>,
    selected_character: Res<SelectedCharacter>,
    container: Query<Entity, With<SpellCardContainer>>,
) {
    let font_size = 40.0;
    let text_font = TextFont {
        font: font.0.clone(),
        font_size,
        ..default()
    };

    if let Ok(container_entity) = container.get_single() {
        commands.entity(container_entity).with_children(|parent| {
            parent.spawn((
                StateScoped(StartState::SpellCard),
                Text::new("Spell Card:"),
                text_font.clone(),
                TextLayout::new_with_justify(JustifyText::Left),
                TextColor(Color::Srgba(WHITE)),
            ));

            match selected_character.selected {
                CharacterState::ReimuHakurei => {
                    let entries = vec![
                        "[ ] Fantasy Orb",
                        "[ ] Homing Amulet",
                    ];
                    for (index, label) in entries.into_iter().enumerate() {
                        parent.spawn((
                            StateScoped(StartState::SpellCard),
                            ReimuSpellCardEntry,
                            Text::new(label),
                            text_font.clone(),
                            TextLayout::default(),
                            TextColor(Color::Srgba(WHITE)),
                            SpellCardEntryIndex(index),
                        ));
                    }
                }
                CharacterState::MarisaKirisame => {
                    let entries = vec![
                        "[ ] Master Spark",
                        "[ ] Stardust Reverie",
                    ];
                    for (index, label) in entries.into_iter().enumerate() {
                        parent.spawn((
                            StateScoped(StartState::SpellCard),
                            MarisaSpellCardEntry,
                            Text::new(label),
                            text_font.clone(),
                            TextLayout::default(),
                            TextColor(Color::Srgba(WHITE)),
                            SpellCardEntryIndex(index),
                        ));
                    }
                }
            }
        });
    }
}

fn spell_card_update_texts(
    selected: Res<SelectedSpellCard>,
    selected_character: Res<SelectedCharacter>,
    mut reimu_texts: Query<(&SpellCardEntryIndex, &mut Text), (With<ReimuSpellCardEntry>, Without<MarisaSpellCardEntry>)>,
    mut marisa_texts: Query<(&SpellCardEntryIndex, &mut Text), (With<MarisaSpellCardEntry>, Without<ReimuSpellCardEntry>)>,
) {
    if !selected.is_changed() {
        return;
    }

    match selected_character.selected {
        CharacterState::ReimuHakurei => {
            for (index, mut text) in reimu_texts.iter_mut() {
                let label = text.0.trim_start_matches(['[', 'X', ']', ' ']);
                if index.0 == selected.selected_index {
                    text.0 = format!("[X] {}", label);
                } else {
                    text.0 = format!("[ ] {}", label);
                }
            }
        }
        CharacterState::MarisaKirisame => {
            for (index, mut text) in marisa_texts.iter_mut() {
                let label = text.0.trim_start_matches(['[', 'X', ']', ' ']);
                if index.0 == selected.selected_index {
                    text.0 = format!("[X] {}", label);
                } else {
                    text.0 = format!("[ ] {}", label);
                }
            }
        }
    }
}

fn spell_card_selection(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedSpellCard>,
    selected_character: Res<SelectedCharacter>,
) {
    let total = match selected_character.selected {
        CharacterState::ReimuHakurei => 2,
        CharacterState::MarisaKirisame => 2,
    };

    let direction = navigation_direction(&keyboard_input, &mut selected.repeat_timer, &time.delta());

    if direction == 0 {
        return;
    }

    selected.selected_index = ((selected.selected_index as isize + direction + total as isize) % total as isize) as usize;
}
fn setup_main_menu(
    mut commands: Commands,
    font: Res<AsciiFont>,
) {
    commands.insert_resource(SelectedMenuEntry {
        selected: MainMenuState::Start,
        repeat_timer: Timer::from_seconds(0.15, TimerMode::Repeating),
    });

    commands.insert_resource(SelectedDifficulty {
        selected: DifficultyState::Easy,
        repeat_timer: Timer::from_seconds(0.15, TimerMode::Repeating),
    });

    commands.insert_resource(SelectedCharacter {
        selected: CharacterState::ReimuHakurei,
        repeat_timer: Timer::from_seconds(0.15, TimerMode::Repeating),
    });

    commands.insert_resource(SelectedSpellCard {
        selected_index: 0,
        repeat_timer: Timer::from_seconds(0.15, TimerMode::Repeating),
    });

    let font_size = 40.0;
    let text_font = TextFont {
        font: font.0.clone(),
        font_size: font_size.clone(),
        ..default()
    };

    commands
        .spawn((
            StateScoped(MainMenuState::Choosing),
            Node {
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::FlexEnd,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
        }))
        .with_children(|parent| {
           parent.spawn(Node {
               flex_direction: FlexDirection::Column,
               margin: UiRect {
                   right: Val::Px(200.0),
                   bottom: Val::Px(font_size),
                   ..default()
               },
               ..default()
           })
               .with_children(|parent| {
                   let menu_items = vec![
                       (MainMenuState::Start, "> Start"),
                       (MainMenuState::ExtraStart, "  Extra Start"),
                       (MainMenuState::PracticeStart, "  Practice Start"),
                       (MainMenuState::Replay, "  Replay"),
                       (MainMenuState::Score, "  Score"),
                       (MainMenuState::MusicRoom, "  Music Room"),
                       (MainMenuState::Option, "  Option"),
                       (MainMenuState::Quit, "  Quit"),
                   ];

                   for (i, (state, label)) in menu_items.iter().enumerate() {
                       parent.spawn(
                           Node {
                               left: Val::Px((menu_items.len() - i) as f32 * font_size * 0.5),
                               ..default()
                           }
                       ).with_children(|parent| {
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


fn difficulty_update_texts(
    selected: Res<SelectedDifficulty>,
    mut texts: Query<(&DifficultyEntry, &mut Text)>,
) {
    if !selected.is_changed() {
        return;
    }

    for (entry, mut text) in texts.iter_mut() {
        let label = text.0.trim_start_matches(['[', 'X', ']', ' ']);
        if entry.0 == selected.selected {
            text.0 = format!("[X] {}", label);
        } else {
            text.0 = format!("[ ] {}", label);
        }
    }
}

fn character_update_texts(
    selected: Res<SelectedCharacter>,
    mut texts: Query<(&CharacterEntry, &mut Text)>,
) {
    if !selected.is_changed() {
        return;
    }

    for (entry, mut text) in texts.iter_mut() {
        let label = text.0.trim_start_matches(['[', 'X', ']', ' ']);
        if entry.0 == selected.selected {
            text.0 = format!("[X] {}", label);
        } else {
            text.0 = format!("[ ] {}", label);
        }
    }
}

fn main_menu_update_texts(
    selected: Res<SelectedMenuEntry>,
    mut texts: Query<(&MainMenuEntry, &mut Text)>,
) {
    if !selected.is_changed() {
        return;
    }

    for (entry, mut text) in texts.iter_mut() {
        let label = text.0.trim_start_matches(['>', ' ']);
        if entry.0 == selected.selected {
            text.0 = format!("> {}", label);
        } else {
            text.0 = format!("  {}", label);
        }
    }
}

fn navigation_direction(
    keyboard_input: &ButtonInput<KeyCode>,
    timer: &mut Timer,
    delta: &std::time::Duration,
) -> isize {
    let mut direction = 0;

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        direction = -1;
        timer.reset();
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        direction = 1;
        timer.reset();
    } else if keyboard_input.pressed(KeyCode::ArrowUp) {
        timer.tick(*delta);
        if timer.finished() {
            direction = -1;
        }
    } else if keyboard_input.pressed(KeyCode::ArrowDown) {
        timer.tick(*delta);
        if timer.finished() {
            direction = 1;
        }
    } else {
        timer.reset();
    }

    direction
}

fn difficulty_selection(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedDifficulty>,
) {
    use DifficultyState::*;

    let order = [Easy, Normal, Hard, Lunatic];

    let direction = navigation_direction(&keyboard_input, &mut selected.repeat_timer, &time.delta());

    if direction == 0 {
        return;
    }

    let current = selected.selected;
    let current_index = order.iter().position(|s| *s == current).unwrap_or(0);
    let new_index = (current_index as isize + direction + order.len() as isize) % order.len() as isize;
    selected.selected = order[new_index as usize];
}

fn character_selection(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedCharacter>,
) {
    use CharacterState::*;

    let order = [ReimuHakurei, MarisaKirisame];
    let direction = navigation_direction(&keyboard_input, &mut selected.repeat_timer, &time.delta());

    if direction == 0 {
        return;
    }

    let current = selected.selected;
    let current_index = order.iter().position(|s| *s == current).unwrap_or(0);
    let new_index = (current_index as isize + direction + order.len() as isize) % order.len() as isize;
    selected.selected = order[new_index as usize];
}

fn main_menu_selection(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedMenuEntry>,
) {
    use MainMenuState::*;

    let order = [
        Start,
        ExtraStart,
        PracticeStart,
        Replay,
        Score,
        MusicRoom,
        Option,
        Quit,
    ];

    let direction = navigation_direction(&keyboard_input, &mut selected.repeat_timer, &time.delta());

    if direction == 0 {
        return;
    }

    let current = selected.selected;
    let current_index = order.iter().position(|s| *s == current).unwrap_or(0);
    let new_index = (current_index as isize + direction + order.len() as isize) % order.len() as isize;
    selected.selected = order[new_index as usize];
}

fn character_confirm_selection(
    selected: Res<SelectedCharacter>,
    mut next_state: ResMut<NextState<StartState>>,
    mut next_state_reimu: ResMut<NextState<ReimuSpellCardState>>,
    mut next_state_marisa: ResMut<NextState<MarisaSpellCardState>>,
) {
    next_state.set(StartState::SpellCard);
    match selected.selected {
        CharacterState::ReimuHakurei => next_state_reimu.set(ReimuSpellCardState::SpellA),
        CharacterState::MarisaKirisame => next_state_marisa.set(MarisaSpellCardState::SpellC),
    }
}

fn spell_card_confirm_selection(
    mut next_state: ResMut<NextState<AppState>>,
) {
    next_state.set(AppState::InGame);
}

fn spell_card_quit(
    mut next_state: ResMut<NextState<StartState>>,
) {
    next_state.set(StartState::Character);
}


fn difficulty_confirm_selection(
    mut next_state: ResMut<NextState<StartState>>,
) {
    next_state.set(StartState::Character);
}

fn main_menu_confirm_selection(
    selected: Res<SelectedMenuEntry>,
    mut next_state: ResMut<NextState<MainMenuState>>,
) {
    next_state.set(selected.selected);
}

fn main_menu_reset_selection(
    mut selected: ResMut<SelectedMenuEntry>,
) {
    selected.selected = MainMenuState::Quit;
}

fn difficulty_quit(
    mut next_state: ResMut<NextState<MainMenuState>>,
) {
    next_state.set(MainMenuState::Choosing);
}

fn character_quit(
    mut next_state: ResMut<NextState<StartState>>
) {
    next_state.set(StartState::Difficulty);
}

fn main_menu_quit(
    mut next_state: ResMut<NextState<MainMenuState>>,
) {
    next_state.set(MainMenuState::Quit);
}

use bevy::app::AppExit;

fn main_menu_handle_quit(
    mut exit_writer: EventWriter<AppExit>,
) {
    exit_writer.send(AppExit::Success);
}

fn back_key_just_pressed(input: Res<ButtonInput<KeyCode>>) -> bool {
    input.just_pressed(KeyCode::KeyX) || input.just_pressed(KeyCode::Escape)
}

fn confirm_key_just_pressed(input: Res<ButtonInput<KeyCode>>) -> bool {
    input.just_pressed(KeyCode::KeyZ) || input.just_pressed(KeyCode::Enter)
}

fn update_start_ui_visibility(
    start_state: Res<State<StartState>>,
    mut difficulty_q: Query<&mut Visibility, With<DifficultyContainer>>,
    mut character_q: Query<&mut Visibility, With<CharacterContainer>>,
    mut spell_q: Query<&mut Visibility, With<SpellCardContainer>>,
) {
    let is_difficulty = *start_state == StartState::Difficulty;
    let is_character = *start_state == StartState::Character;
    let is_spell = *start_state == StartState::SpellCard;

    if let Ok(mut vis) = difficulty_q.get_single_mut() {
        *vis = if is_difficulty { Visibility::Visible } else { Visibility::Hidden };
    }
    if let Ok(mut vis) = character_q.get_single_mut() {
        *vis = if is_character { Visibility::Visible } else { Visibility::Hidden };
    }
    if let Ok(mut vis) = spell_q.get_single_mut() {
        *vis = if is_spell { Visibility::Visible } else { Visibility::Hidden };
    }
}

pub struct GameUiPlugin;
impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_sub_state::<MainMenuState>()
            .add_sub_state::<StartState>()
            .add_sub_state::<DifficultyState>()
            .add_sub_state::<CharacterState>()
            .add_sub_state::<ReimuSpellCardState>()
            .add_sub_state::<MarisaSpellCardState>()
            .enable_state_scoped_entities::<MainMenuState>()
            .enable_state_scoped_entities::<StartState>()
            .add_systems(OnEnter(MainMenuState::Choosing), setup_main_menu)
            .add_systems(OnEnter(AppState::InGame), setup_in_game)
            .add_systems(OnEnter(StartState::SpellCard), setup_spell_cards)
            .add_systems(OnEnter(StartState::Difficulty), setup_difficulty)
            .add_systems(OnEnter(StartState::Character), setup_character)
            .add_systems(OnEnter(MainMenuState::Start), setup_start)
            .add_systems(Update, (
                main_menu_selection,
                main_menu_confirm_selection.run_if(confirm_key_just_pressed),
                main_menu_update_texts.run_if(resource_changed::<SelectedMenuEntry>),
                main_menu_reset_selection.run_if(back_key_just_pressed),
                main_menu_quit.run_if(input_just_pressed(KeyCode::KeyQ)),
            ).run_if(in_state(MainMenuState::Choosing)))
            .add_systems(Update, (
                difficulty_selection,
                difficulty_update_texts.run_if(resource_changed::<SelectedDifficulty>),
                difficulty_confirm_selection.run_if(confirm_key_just_pressed),
                difficulty_quit.run_if(back_key_just_pressed),
            ).run_if(in_state(StartState::Difficulty)))
            .add_systems(Update, (
                character_selection,
                character_update_texts.run_if(resource_changed::<SelectedCharacter>),
                character_confirm_selection.run_if(confirm_key_just_pressed),
                character_quit.run_if(back_key_just_pressed),
            ).run_if(in_state(StartState::Character)))
            .add_systems(Update, (
                spell_card_selection,
                spell_card_update_texts.run_if(resource_changed::<SelectedSpellCard>),
                spell_card_confirm_selection.run_if(confirm_key_just_pressed),
                spell_card_quit.run_if(back_key_just_pressed),
            ).run_if(in_state(StartState::SpellCard)))
            .add_systems(OnEnter(MainMenuState::Quit), main_menu_handle_quit)   ;
    }
}