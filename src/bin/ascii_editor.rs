use bevy::color::palettes::basic::WHITE;
use bevy::ecs::event::EventCursor;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use asciihou::ascii_animation::AsciiAnimationPlugin;
use asciihou::resource::AsciiFont;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);

#[derive(Resource, Default)]
struct ColorInput {
    value: String,
    active: bool,
}
#[derive(Resource, Default)]
struct CharInput {
    value: String,
    active: bool,
}
#[derive(Resource, Default)]
struct Brush {
    ch: char,
    color: Color,
}
#[derive(Component)]
struct ColorInputBox;
#[derive(Component)]
struct ColorInputText;
#[derive(Component)]
struct ColorContainer;
#[derive(Component)]
struct ColorSample(Color);
#[derive(Component)]
struct BrushSample;
#[derive(Component)]
struct CharInputBox;
#[derive(Component)]
struct CharInputText;
#[derive(Component)]
struct CharContainer;
#[derive(Component)]
struct CharSample{
    ch: char,
    color: Color
}

struct Cell {
    ch: char,
    color: String, // "#rrggbb"
}
struct AsciiFrame {
    cells: Vec<Cell>, // width * height
}

fn parse_hex_color(hex: &str) -> Result<Color, ()> {
    if hex.len() != 6 {
        return Err(());
    }
    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| ())?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| ())?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| ())?;
    Ok(Color::srgb_u8(r, g, b))
}
fn button_system(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor, Entity), (Changed<Interaction>, With<Button>)>,
    mut color_input: ResMut<ColorInput>,
    mut char_input: ResMut<CharInput>,
    color_input_box: Query<Entity, With<ColorInputBox>>,
    char_input_box: Query<Entity, With<CharInputBox>>,
    color_sample_query: Query<&ColorSample>,
    char_sample_query: Query<&CharSample>,
    mut commands: Commands,
    color_container_query: Query<Entity, With<ColorContainer>>,
    char_container_query: Query<Entity, With<CharContainer>>,
    mut brush: ResMut<Brush>,
    font: Res<AsciiFont>,
) {
    for (interaction, mut background, entity) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if color_input_box.get(entity).is_ok() {
                    *background = PRESSED_BUTTON.into();
                }
                if char_input_box.get(entity).is_ok() {
                    *background = PRESSED_BUTTON.into();
                }


                if color_input_box.get(entity).is_ok() {
                    color_input.active = true;
                    char_input.active = false;

                    if let Ok(parsed_color) = parse_hex_color(&color_input.value) {
                        if let Ok(container) = color_container_query.get_single() {
                            commands.entity(container).with_children(|parent| {
                                parent.spawn((
                                    ColorSample(parsed_color),
                                    Node {
                                        width: Val::Px(30.0),
                                        height: Val::Px(30.0),
                                        margin: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    Button,
                                    BackgroundColor(parsed_color),
                                ));
                            });
                        }
                    }
                    continue;
                }

                if char_input_box.get(entity).is_ok() {
                    char_input.active = true;
                    color_input.active = false;

                    if let Ok(container) = char_container_query.get_single() {
                        commands.entity(container).with_children(|parent| {
                            parent.spawn((
                                CharSample {
                                    ch: brush.ch,
                                    color: brush.color
                                },
                                Button,
                                Node {
                                    width: Val::Px(30.0),
                                    height: Val::Px(30.0),
                                    margin: UiRect::all(Val::Px(2.0)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                            )).with_children(|parent| {
                                parent.spawn((
                                    Text::new(brush.ch.to_string()),
                                    TextFont {
                                        font: font.0.clone(),
                                        font_size: 40.0,
                                        ..default()
                                    },
                                    TextLayout::default(),
                                    TextColor(brush.color)
                                ));
                            });
                        });
                    }
                    continue;
                }

                if let Ok(sample) = color_sample_query.get(entity) {
                    brush.color = sample.0;
                    continue;
                }

                if let Ok(sample) = char_sample_query.get(entity) {
                    brush.ch = sample.ch;
                    brush.color = sample.color;
                    continue;
                }
            }
            Interaction::Hovered => {
                if color_sample_query.get(entity).is_ok() {
                    continue;
                }

                if char_sample_query.get(entity).is_ok() {
                    continue;
                }

                if color_input_box.get(entity).is_ok() {
                    color_input.active = true;
                    char_input.active = false;
                }
                if char_input_box.get(entity).is_ok() {
                    char_input.active = true;
                    color_input.active = false;
                }
                *background = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                if color_sample_query.get(entity).is_ok() {
                    continue;
                }
                if char_sample_query.get(entity).is_ok() {
                    continue;
                }
                color_input.active = false;
                char_input.active = false;
                *background = NORMAL_BUTTON.into();
            }
        }
    }
}

fn char_input(
    mut char_input: ResMut<CharInput>,
    mut brush: ResMut<Brush>,
    mut text_query: Query<&mut Text, With<CharInputText>>,
    mut reader: Local<EventCursor<KeyboardInput>>,
    events: Res<Events<KeyboardInput>>,
) {
    if !char_input.active {
        return;
    }

    let mut changed = false;

    for event in reader.read(&events) {
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            Key::Character(s) => {
                if let Some(c) = s.chars().next() {
                    char_input.value = c.to_string();
                    brush.ch = c;
                    changed = true;
                }
            }
            Key::Backspace => {
                char_input.value.clear();
                brush.ch = ' ';
                changed = true;
            }
            _ => {}
        }
    }

    if changed {
        for mut text in &mut text_query {
            text.0 = char_input.value.clone();
        }
    }
}
fn setup(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2d::default());
    let font = asset_server.load("font/UbuntuMono-R.ttf");
    commands.insert_resource(AsciiFont(font.clone()));
    let font_size = 40.0;

    commands.spawn(Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::FlexStart,
        flex_direction: FlexDirection::Row,
        ..default()
    }).with_children(|parent| {
        parent.spawn(Node {
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Column,
            ..default()
        }).with_children(|parent| {
            parent.spawn((
                Text::new("Brush: "),
                TextFont {
                    font: font.clone(),
                    font_size,
                    ..default()
                },
                TextLayout::default(),
                TextColor(Color::Srgba(WHITE))
            ));
            parent.spawn((
                Node::default(),
            )).with_children(|parent| {
                parent.spawn((
                    Text::new(""),
                    TextFont {
                        font: font.clone(),
                        font_size,
                        ..default()
                    },
                    TextLayout::default(),
                    TextColor(Color::Srgba(WHITE)),
                    BrushSample
                ));
            });

            parent.spawn((
                Button,
                BorderColor(Color::BLACK),
                BorderRadius::MAX,
                BackgroundColor(NORMAL_BUTTON),
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(65.0),
                    border: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ColorInputBox
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("Color: "),
                    TextFont {
                        font: font.clone(),
                        font_size,
                        ..default()
                    },
                    TextLayout::default(),
                    TextColor(Color::Srgba(WHITE))
                ));
            });

            parent.spawn((
                Text::new("ffffff"),
                TextFont {
                    font: font.clone(),
                    font_size,
                    ..default()
                },
                TextLayout::default(),
                TextColor(Color::Srgba(WHITE)),
                ColorInputText,
            ));

            parent.spawn((
                Node {
                    width: Val::Percent(25.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                ColorContainer
            ));

            parent.spawn((
                Button,
                BorderColor(Color::BLACK),
                BorderRadius::MAX,
                BackgroundColor(NORMAL_BUTTON),
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(65.0),
                    border: UiRect::all(Val::Px(5.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                CharInputBox
            )).with_children(|parent| {
                parent.spawn((
                    Text::new("Char: "),
                    TextFont {
                        font: font.clone(),
                        font_size,
                        ..default()
                    },
                    TextLayout::default(),
                    TextColor(Color::Srgba(WHITE)),
                ));
            });

            parent.spawn((
                Text::new(" "),
                TextFont {
                    font: font.clone(),
                    font_size,
                    ..default()
                },
                TextLayout::default(),
                TextColor(Color::Srgba(WHITE)),
                CharInputText,
            ));

            parent.spawn((
                Node {
                    width: Val::Percent(25.0),
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    ..default()
                },
                CharContainer,
            ));
        });
    });
}

fn color_input(
    mut input: ResMut<ColorInput>,
    mut text_query: Query<&mut Text, With<ColorInputText>>,
    mut brush: ResMut<Brush>,
    mut reader: Local<EventCursor<KeyboardInput>>,
    events: Res<Events<KeyboardInput>>,
) {
    if !input.active {
        return;
    }

    let mut changed = false;

    for event in reader.read(&events) {
        if !event.state.is_pressed() {
            continue;
        }

        match &event.logical_key {
            Key::Character(s) => {
                if input.value.len() < 6 && s.chars().all(|c| c.is_ascii_hexdigit()) {
                    input.value.push_str(s);
                    changed = true;
                }
            }
            Key::Backspace => {
                input.value.pop();
                changed = true;
            }
            _ => {}
        }
    }

    if changed {
        for mut text in &mut text_query {
            text.0 = input.value.clone();
        }

        if let Ok(parsed) = parse_hex_color(&input.value) {
            brush.color = parsed;
        }
    }
}
fn update_brush_sample(
    brush: Res<Brush>,
    mut sample_query: Query<(&mut TextColor, &mut Text), With<BrushSample>>,
) {
    if brush.is_changed() {
        for (mut color, mut text) in &mut sample_query {
            *color = brush.color.into();
            text.0 = brush.ch.to_string();
        }
    }
}
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(AsciiAnimationPlugin)
        .init_resource::<ColorInput>()
        .init_resource::<Brush>()
        .init_resource::<CharInput>()
        .add_systems(Startup, setup)
        .add_systems(Update, (
            button_system,
            color_input,
            char_input,
            update_brush_sample.run_if(resource_changed::<Brush>),
        ))
        .run();
}