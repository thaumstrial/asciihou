use bevy::asset::{AssetMetaCheck, AssetServer};
use bevy::color::palettes::css::*;
use bevy::color::palettes::tailwind::{BLUE_400, RED_400};
use bevy::DefaultPlugins;
use bevy::input::common_conditions::*;
use bevy::text::{JustifyText, Text2d, TextFont, TextLayout};
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResized};
use bevy_rapier2d::prelude::*;
use bevy_rapier2d::prelude::ColliderMassProperties::MassProperties;

#[derive(Component)]
struct Player;
#[derive(Component)]
struct ShootCooldown(Timer);
#[derive(Component)]
struct PlayerBullet;
#[derive(Component)]
struct PlayerHomingBullet {
    speed: f32,
    rotate_speed: f32,
}
#[derive(Component)]
struct Enemy;
#[derive(Component)]
struct EnemyBullet;
#[derive(Component)]
struct Health(i32);
#[derive(Component)]
struct LinearMovement(Vec2);
#[derive(Component)]
struct EnemySingleShoot {
    direction: Vec2,
    cooldown: Timer,
}
#[derive(Component)]
struct PlayerLivesText;
#[derive(Component)]
struct PlayerBombsText;
#[derive(Component)]
struct PlayerPowersText;
#[derive(Component)]
struct PlayerPointsText;
#[derive(Component)]
struct PowerItem;
#[derive(Component)]
struct PointItem;
#[derive(Component)]
struct SupportUnit;
#[derive(Component)]
struct SupportHomingShoot {
    speed: f32,
    rotate_speed: f32,
    cooldown: Timer,
}

#[derive(Resource)]
struct AsciiFont(Handle<Font>);
#[derive(Resource)]
struct ShowColliderDebug(bool);
#[derive(Resource)]
struct EnemySpawnTimer {
    timer: Timer,
}
#[derive(Resource, Clone, Copy)]
struct WindowSize {
    width: f32,
    height: f32,
}
#[derive(Resource)]
struct PlayerLives(pub i32);
#[derive(Resource)]
struct PlayerBombs(pub i32);
#[derive(Resource)]
struct PlayerPowers(pub i32);
#[derive(Resource)]
struct PlayerPoints(pub i32);

fn update_lives_text(
    lives: Res<PlayerLives>,
    mut query: Query<&mut Text2d, With<PlayerLivesText>>,
) {
    let num = "@".repeat(lives.0.max(0) as usize);
    let margins = " ".repeat(lives.0.max(0) as usize);
    for mut text in query.iter_mut() {
        text.0 = format!("  {}Player: {}", margins, num);
    }
}
fn update_bombs_text(
    bombs: Res<PlayerBombs>,
    mut query: Query<&mut Text2d, With<PlayerBombsText>>,
) {
    let num = "$".repeat(bombs.0.max(0) as usize);
    let margins = " ".repeat(bombs.0.max(0) as usize);
    for mut text in query.iter_mut() {
        text.0 = format!("{}Bomb: {}", margins, num);
    }
}
fn update_powers_text(
    powers: Res<PlayerPowers>,
    mut query: Query<&mut Text2d, With<PlayerPowersText>>,
) {
    let num = powers.0.to_string();
    let margins = " ".repeat(powers.0.to_string().len().max(0) as usize);
    for mut text in query.iter_mut() {
        text.0 = format!(" {}Power: {}", margins, num);
    }
}
fn update_points_text(
    points: Res<PlayerPoints>,
    mut query: Query<&mut Text2d, With<PlayerPointsText>>,
) {
    let num = points.0.to_string();
    let margins = " ".repeat(points.0.to_string().len().max(0) as usize);
    for mut text in query.iter_mut() {
        text.0 = format!(" {}Point: {}", margins, num);
    }
}

fn linear_movement(
    mut commands: Commands,
    mut query: Query<(Entity, &LinearMovement, &mut Velocity)>,
) {
    for (entity, movement, mut velocity) in query.iter_mut() {
        velocity.linvel += movement.0;
        commands.entity(entity).remove::<LinearMovement>();
    }
}

fn player_homing_bullet(
    mut query: Query<(&mut Velocity, &Transform, &PlayerHomingBullet)>,
    enemies: Query<&Transform, With<Enemy>>,
    time: Res<Time>,
) {
    for (mut velocity, bullet_transform, homing) in query.iter_mut() {
        let current_dir = velocity.linvel.normalize_or_zero();

        if let Some(target) = enemies.iter()
            .min_by(|a, b| {
                let da = bullet_transform.translation.distance_squared(a.translation);
                let db = bullet_transform.translation.distance_squared(b.translation);
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            }) {

            let desired_dir = (target.translation.truncate() - bullet_transform.translation.truncate())
                .normalize_or_zero();

            let angle_between = current_dir.angle_to(desired_dir);
            let max_rotate = homing.rotate_speed * time.delta_secs();

            let clamped_angle = angle_between.clamp(-max_rotate, max_rotate);
            let new_dir = current_dir.rotate(Vec2::from_angle(clamped_angle));

            velocity.linvel = new_dir.normalize_or_zero() * homing.speed;
        }
    }
}


fn support_homing_shoot(
    mut commands: Commands,
    mut query: Query<(&GlobalTransform, &mut SupportHomingShoot)>,
    enemies: Query<&GlobalTransform, With<Enemy>>,
    time: Res<Time>,
    font: Res<AsciiFont>,
) {
    for (support_transform, mut homing) in query.iter_mut() {
        homing.cooldown.tick(time.delta());
        if !homing.cooldown.finished() {
            continue;
        }

        if let Some(target) = enemies.iter()
            .min_by(|a, b| {
                let da = support_transform.translation().truncate().distance_squared(a.translation().truncate());
                let db = support_transform.translation().truncate().distance_squared(b.translation().truncate());
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            }) {

            let direction = (target.translation().truncate() - support_transform.translation().truncate())
                .normalize_or_zero();

            commands.spawn((
                PlayerBullet,
                PlayerHomingBullet {
                    speed: homing.speed,
                    rotate_speed: homing.rotate_speed,
                },
                Transform::from_translation(support_transform.translation()),
                Text2d::new("*"),
                TextFont {
                    font: font.0.clone(),
                    font_size: 30.0,
                    ..default()
                },
                TextLayout::default(),
                TextColor(Color::Srgba(PURPLE)),
                Collider::ball(5.0),
                RigidBody::KinematicVelocityBased,
                Velocity::linear(direction * homing.speed),
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(Group::GROUP_2, Group::GROUP_4),
            ));
        }

        homing.cooldown.reset();
    }
}

fn enemy_single_shoot(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut EnemySingleShoot)>,
    time: Res<Time>,
    font: Res<AsciiFont>,
) {
    for (transform, mut single_shot) in query.iter_mut() {
        single_shot.cooldown.tick(time.delta());

        if single_shot.cooldown.finished() {
            let spawn_pos = transform.translation;

            commands.spawn((
                EnemyBullet,

                Transform::from_translation(spawn_pos),
                Text2d::new("x"),
                TextFont {
                    font: font.0.clone(),
                    font_size: 30.0,
                    ..default()
                },
                TextLayout::default(),
                TextColor(Color::Srgba(WHITE)),

                Collider::ball(5.0),
                RigidBody::KinematicVelocityBased,
                Velocity::linear(single_shot.direction),
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(Group::GROUP_8, Group::GROUP_1),
            ));

            single_shot.cooldown.reset();
        }
    }
}

fn spawn_support_units(
    mut commands: Commands,
    font: Res<AsciiFont>,
    powers: Res<PlayerPowers>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    support_query: Query<(Entity), With<SupportUnit>>,
) {
    if powers.0 < 1 || support_query.iter().count() >= 2 {
        return;
    }

    if let Ok((player_entity, player_transform)) = player_query.get_single() {
        let offsets = [Vec2::new(30.0, 30.0), Vec2::new(-30.0, 30.0)];

        for offset_pos in offsets {
            commands.spawn((
                SupportUnit,
                SupportHomingShoot {
                    speed: 800.0,
                    rotate_speed: 1.0,
                    cooldown: Timer::from_seconds(0.2, TimerMode::Repeating)
                },
                Text2d::new("N"),
                TextFont {
                    font: font.0.clone(),
                    font_size: 30.0,
                    ..default()
                },
                Transform::from_translation(offset_pos.extend(0.0)),
                RigidBody::KinematicVelocityBased,
                Velocity {
                    angvel: 2.0,
                    ..default()
                },
                TextLayout::default(),
                TextColor(Color::Srgba(PINK)),
            )).set_parent(player_entity);
        }
    }
}

fn despawn_support_units(
    mut commands: Commands,
    powers: Res<PlayerPowers>,
    query: Query<Entity, With<SupportUnit>>,
) {
    if powers.0 <= 20 {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_enemies(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
    enemy_query: Query<Entity, With<Enemy>>,
    player_query: Query<&Transform, With<Player>>,
    window: Res<WindowSize>,
    font: Res<AsciiFont>,
) {
    const MAX_ENEMIES: usize = 10;
    const SPAWN_CHANCE: f32 = 0.5;
    const MAX_DEVIATION_DEG: f32 = 30.0;
    const MAX_SHOOT_DEVIATION_DEG: f32 = 10.0;

    if enemy_query.iter().count() >= MAX_ENEMIES {
        return;
    }

    spawn_timer.timer.tick(time.delta());
    if spawn_timer.timer.finished() {
        if rand::random::<f32>() < SPAWN_CHANCE {
            let Ok(player_transform) = player_query.get_single() else { return; };
            let player_pos = player_transform.translation.truncate();

            let min_x = -window.width / 2.0 + 45.0;
            let max_x = window.width * 0.25 - 5.0;
            let spawn_x = rand::random::<f32>() * (max_x - min_x) + min_x;
            let spawn_y = window.height / 2.0;

            let spawn_pos = Vec2::new(spawn_x, spawn_y);
            let angle = (rand::random::<f32>() * 2.0 - 1.0) * MAX_DEVIATION_DEG.to_radians();
            let direction = (player_pos - spawn_pos).rotate(Vec2::from_angle(angle));

            let speed = rand::random::<f32>() * 100.0 + 100.0;
            let movement_vec = direction.normalize_or_zero() * speed;

            let shoot_angle = (rand::random::<f32>() * 2.0 - 1.0) * MAX_SHOOT_DEVIATION_DEG.to_radians();
            let shoot_direction = (player_pos - spawn_pos).rotate(Vec2::from_angle(shoot_angle)).normalize_or_zero() * speed;

            commands.spawn((
                Text2d::new("&"),
                TextFont {
                    font: font.0.clone(),
                    font_size: 40.0,
                    ..default()
                },
                TextLayout::new_with_justify(JustifyText::Center),
                TextColor(Color::Srgba(ORANGE)),

                Enemy,
                RigidBody::Dynamic,
                Sensor,
                Collider::ball(10.0),
                Velocity::zero(),
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(Group::GROUP_4, Group::GROUP_1 | Group::GROUP_2),

                Transform::from_translation(spawn_pos.extend(0.0)),
                LinearMovement(movement_vec),
                Health((rand::random::<u32>() % 3 + 1) as i32),

                EnemySingleShoot {
                    direction: shoot_direction * (rand::random::<f32>() * 2.0 + 2.0),
                    cooldown: Timer::from_seconds(rand::random::<f32>() * 0.4 + 0.1, TimerMode::Repeating),
                }
            ));
        }
    }
}

fn item_gravity(
    mut query: Query<&mut Velocity, Or<(With<PowerItem>, With<PointItem>)>>,
    time: Res<Time>,
) {
    let gravity_acc = -100.0;
    let max_fall_speed = -100.0;
    let horizontal_decay = 10.0;

    for mut velocity in query.iter_mut() {
        velocity.linvel.y += gravity_acc * time.delta_secs();
        if velocity.linvel.y < max_fall_speed {
            velocity.linvel.y = max_fall_speed;
        }

        if velocity.linvel.x.abs() > 0.0 {
            let decay = horizontal_decay * time.delta_secs();
            if velocity.linvel.x > 0.0 {
                velocity.linvel.x = (velocity.linvel.x - decay).max(0.0);
            } else {
                velocity.linvel.x = (velocity.linvel.x + decay).min(0.0);
            }
        }
    }
}

fn bullet_hit_enemy(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    mut enemies: Query<(Entity, &mut Health, &Transform), With<Enemy>>,
    bullets: Query<Entity, With<PlayerBullet>>,
    font: Res<AsciiFont>,
) {
    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(entity1, entity2, _) => {

                let (bullet_entity, enemy_entity) = if bullets.get(*entity1).is_ok() && enemies.get(*entity2).is_ok() {
                    (*entity1, *entity2)
                } else if bullets.get(*entity2).is_ok() && enemies.get(*entity1).is_ok() {
                    (*entity2, *entity1)
                } else {
                    continue;
                };

                if let Ok((enemy_ent, mut health, transform)) = enemies.get_mut(enemy_entity) {
                    health.0 -= 1;
                    if health.0 <= 0 {
                        let power_count = rand::random::<u32>() % 3 + 1;
                        const ITEM_SPEED: f32 = 50.0;

                        for _ in 0..power_count {

                            commands.spawn((
                                PowerItem,

                                Sprite::from_color(Color::Srgba(RED_400), Vec2::new(20.0, 20.0)),
                                Transform::from_translation(transform.translation.xy().extend(-1.0)),

                                Collider::ball(8.0),
                                RigidBody::KinematicVelocityBased,
                                Velocity::linear(Vec2::new(
                                    (rand::random::<f32>() - 0.5) * ITEM_SPEED,
                                    150.0 + rand::random::<f32>() * 50.0
                                )),
                                ActiveEvents::COLLISION_EVENTS,
                                CollisionGroups::new(Group::GROUP_6, Group::GROUP_1),
                            )).with_children(|builder| {
                                builder.spawn((
                                    Text2d::new("P"),
                                    TextFont {
                                        font: font.0.clone(),
                                        font_size: 25.0,
                                        ..default()
                                    },
                                    TextLayout::default(),
                                    TextColor(Color::Srgba(WHITE)),
                                    Transform::from_translation(Vec3::Z),
                                ));
                            });
                        }

                        let point_count = rand::random::<u32>() % 3 + 1;
                        for _ in 0..point_count {
                            commands.spawn((
                                PointItem,

                                Sprite::from_color(Color::Srgba(BLUE_400), Vec2::new(20.0, 20.0)),
                                Transform::from_translation(transform.translation.xy().extend(-3.0)),
                                RigidBody::KinematicVelocityBased,
                                Collider::ball(8.0),
                                Velocity::linear(Vec2::new(
                                    (rand::random::<f32>() - 0.5) * ITEM_SPEED,
                                    150.0 + rand::random::<f32>() * 50.0
                                )),
                                CollisionGroups::new(Group::GROUP_6, Group::GROUP_1),
                                ActiveEvents::COLLISION_EVENTS,
                            )).with_children(|builder| {
                                builder.spawn((
                                    Text2d::new("%"),
                                    TextFont {
                                        font: font.0.clone(),
                                        font_size: 25.0,
                                        ..default()
                                    },
                                    TextLayout::default(),
                                    TextColor(Color::Srgba(WHITE)),
                                    Transform::from_translation(Vec3::Z),
                                ));
                            });
                        }

                        commands.entity(enemy_ent).despawn();
                    }
                }
                commands.entity(bullet_entity).despawn();
            }
            _ => {}
        }
    }
}

fn bullet_hit_player(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    player_query: Query<Entity, With<Player>>,
    bullets: Query<Entity, With<EnemyBullet>>,
    mut lives: ResMut<PlayerLives>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = event {
            let (bullet_entity, _) = if bullets.get(*entity1).is_ok() && player_query.get(*entity2).is_ok() {
                (*entity1, *entity2)
            } else if bullets.get(*entity2).is_ok() && player_query.get(*entity1).is_ok() {
                (*entity2, *entity1)
            } else {
                continue;
            };

            lives.0 = (lives.0 - 1).max(0);
            commands.entity(bullet_entity).despawn();
        }
    }
}
fn item_hit_player(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    players: Query<Entity, With<Player>>,
    power_items: Query<Entity, With<PowerItem>>,
    point_items: Query<Entity, With<PointItem>>,
    mut powers: ResMut<PlayerPowers>,
    mut points: ResMut<PlayerPoints>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(entity1, entity2, _) = event {
            let (_, item_entity) = if players.get(*entity1).is_ok() {
                (*entity1, *entity2)
            } else if players.get(*entity2).is_ok() {
                (*entity2, *entity1)
            } else {
                continue;
            };

            if power_items.get(item_entity).is_ok() {
                powers.0 += 1;
                commands.entity(item_entity).despawn_recursive();
            } else if point_items.get(item_entity).is_ok() {
                points.0 += 1;
                commands.entity(item_entity).despawn_recursive();
            }
        }
    }
}



fn toggle_debug_render(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut debug_state: ResMut<ShowColliderDebug>,
    mut ctx: ResMut<DebugRenderContext>,
) {
    if keyboard_input.just_pressed(KeyCode::F3) {
        debug_state.0 = !debug_state.0;
        ctx.enabled = debug_state.0;
        info!("Collider Debug View: {}", if debug_state.0 { "ON" } else { "OFF" });
    }
}

fn tick_cooldown_timer(
    mut query: Query<&mut ShootCooldown>,
    time: Res<Time>,
) {
    for mut cooldown in query.iter_mut() {
        cooldown.0.tick(time.delta());
    }
}

fn despawn_bullets(
    mut commands: Commands,
    query: Query<
        (Entity, &Transform),
        Or<(
            With<PlayerBullet>,
            With<EnemyBullet>,
        )>
    >,
    window: Res<WindowSize>,
) {
    for (entity, transform) in query.iter() {
        let pos = transform.translation;
        if pos.x.abs() > window.width / 2.0 || pos.y.abs() > window.height / 2.0 {
            commands.entity(entity).despawn();
        }
    }
}

fn despawn_items(
    mut commands: Commands,
    query: Query<
        (Entity, &Transform),
        Or<(
            With<PowerItem>,
            With<PointItem>,
        )>
    >,
    window: Res<WindowSize>,
) {
    const EXTRA_MARGIN: f32 = 200.0;

    let max_x = window.width / 2.0 + EXTRA_MARGIN;
    let max_y = window.height / 2.0 + EXTRA_MARGIN;

    for (entity, transform) in query.iter() {
        let pos = transform.translation;
        if pos.x.abs() > max_x || pos.y.abs() > max_y {
            commands.entity(entity).despawn();
        }
    }
}

fn despawn_enemies(
    mut commands: Commands,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
    window: Res<WindowSize>,
) {
    const EXTRA_MARGIN: f32 = 100.0;

    let max_x = window.width / 2.0 + EXTRA_MARGIN;
    let max_y = window.height / 2.0 + EXTRA_MARGIN;

    for (entity, transform) in enemy_query.iter() {
        let pos = transform.translation;
        if pos.x.abs() > max_x || pos.y.abs() > max_y {
            commands.entity(entity).despawn();
        }
    }
}

fn player_bomb(
    mut bombs: ResMut<PlayerBombs>,
) {
    bombs.0 = (bombs.0 - 1).max(0);
}

fn player_shoot(
    mut query: Query<(&Transform, &mut ShootCooldown), With<Player>>,
    font: Res<AsciiFont>,
    mut commands: Commands,
    powers: Res<PlayerPowers>,
) {
    for (transform, mut cooldown) in query.iter_mut() {
        if cooldown.0.finished() {
            const BULLET_SPEED: f32 = 800.0;
            const  BASE_DIRECTION: Vec2 = Vec2::Y;

            let (num_bullets, angle_step_deg) = if powers.0 > 50 {
                (5, 10.0)
            } else if powers.0 > 30 {
                (3, 10.0)
            } else {
                (1, 0.0)
            };

            for i in 0..num_bullets {
                let offset = i as i32 - (num_bullets as i32 - 1) / 2;
                let angle_rad = (offset as f32) * (angle_step_deg as f32).to_radians();
                let rotated_direction = Vec2::from_angle(angle_rad).rotate(BASE_DIRECTION);

                commands.spawn((
                    PlayerBullet,

                    Transform::from_translation(transform.translation),
                    Text2d::new("*"),
                    TextFont {
                        font: font.0.clone(),
                        font_size: 30.0,
                        ..default()
                    },
                    TextLayout::default(),
                    TextColor(Color::Srgba(BLACK)),

                    Collider::ball(5.0),
                    RigidBody::KinematicVelocityBased,
                    Velocity::linear(rotated_direction * BULLET_SPEED),
                    ActiveEvents::COLLISION_EVENTS,
                    CollisionGroups::new(Group::GROUP_2, Group::GROUP_4),
                ));
            }
            cooldown.0.reset();
        }
    }
}

fn clamp_player_position(
    mut query: Query<&mut Transform, With<Player>>,
    window: Res<WindowSize>,
) {
    for mut transform in query.iter_mut() {
        let pos = &mut transform.translation;
        pos.x = pos.x.clamp(-window.width / 2.0 + 45.0, window.width / 2.0 * 0.25 - 5.0);
        pos.y = pos.y.clamp(-window.height / 2.0 + 45.0, window.height / 2.0 - 45.0);
    }
}


fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>,
) {
    const SPEED: f32 = 210.0;
    for mut velocity in query.iter_mut() {
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0;
        }

        direction = direction.normalize_or_zero();

        let speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
            SPEED * 0.3
        } else {
            SPEED
        };

        velocity.linvel = direction * speed;
    }
}

fn auto_zoom_camera(
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<&mut OrthographicProjection, With<Camera2d>>,
    window_size: Res<WindowSize>,
) {
    for event in resize_events.read() {
        let base_width = window_size.width;
        let base_height = window_size.height;

        let scale_x = event.width / base_width;
        let scale_y = event.height / base_height;

        let new_scale = scale_x.min(scale_y); // 保持等比缩放，防止拉伸

        for mut projection in query.iter_mut() {
            projection.scale = 1.0 / new_scale;
        }

    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let font = asset_server.load("font/UbuntuMono-R.ttf");
    commands.insert_resource(AsciiFont(font.clone()));

    commands.insert_resource(ShowColliderDebug(false));
    commands.insert_resource(EnemySpawnTimer {
        timer: Timer::from_seconds(1.0, TimerMode::Repeating),
    });

    commands.insert_resource(PlayerLives(2));
    commands.insert_resource(PlayerBombs(3));
    commands.insert_resource(PlayerPoints(0));
    commands.insert_resource(PlayerPowers(0));

    let font_size = 40.0;
    let text_font = TextFont {
        font: font.clone(),
        font_size: font_size.clone(),
        ..default()
    };
    let text_justification = JustifyText::Center;

    commands.spawn(Camera2d);
    commands.spawn((
        Text2d::new("@"),
        text_font.clone(),
        TextLayout::new_with_justify(text_justification),
        TextColor(Color::Srgba(RED)),

        Player,
        ShootCooldown(Timer::from_seconds(0.1,  TimerMode::Once)),

        RigidBody::Dynamic,
        Sensor,
        Collider::ball(10.0),
        Velocity::zero(),

        ActiveEvents::COLLISION_EVENTS,
        CollisionGroups::new(Group::GROUP_1, Group::GROUP_4 | Group::GROUP_6 | Group::GROUP_8)
    ));

    let width = 1280.0;
    let height = 720.0;

    commands.insert_resource(WindowSize {
        width,
        height,
    });

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
        PlayerPointsText,
    ));
}


fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.build()
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        position: WindowPosition::Centered(MonitorSelection::Primary),
                        canvas: Some("#bevy".to_owned()),
                        present_mode: PresentMode::AutoVsync,
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: true,
                        ..default()
                    }),
                    ..default() })
                .set(AssetPlugin {
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
        )
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin {
            enabled: false,
            ..default()
        })
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_debug_render)
        .add_systems(Update, spawn_enemies)
        .add_systems(Update, linear_movement)
        .add_systems(Update, auto_zoom_camera)
        .add_systems(Update, bullet_hit_enemy)
        .add_systems(Update, bullet_hit_player)
        .add_systems(Update, item_hit_player)
        .add_systems(Update, enemy_single_shoot)
        .add_systems(Update, support_homing_shoot)
        .add_systems(Update, update_lives_text.run_if(resource_changed::<PlayerLives>))
        .add_systems(Update, update_bombs_text.run_if(resource_changed::<PlayerBombs>))
        .add_systems(Update, update_powers_text.run_if(resource_changed::<PlayerPowers>))
        .add_systems(Update, update_points_text.run_if(resource_changed::<PlayerPoints>))
        .add_systems(Update, player_shoot.run_if(input_pressed(KeyCode::KeyJ)))
        .add_systems(Update, player_bomb.run_if(input_just_pressed(KeyCode::KeyK)))
        .add_systems(Update, spawn_support_units.run_if(resource_changed::<PlayerPowers>))
        .add_systems(Update, despawn_support_units.run_if(resource_changed::<PlayerPowers>))
        .add_systems(FixedUpdate, tick_cooldown_timer)
        .add_systems(FixedUpdate, despawn_bullets)
        .add_systems(FixedUpdate, despawn_items)
        .add_systems(FixedUpdate, despawn_enemies)
        .add_systems(FixedUpdate, clamp_player_position)
        .add_systems(FixedUpdate, item_gravity)
        .add_systems(FixedUpdate, player_homing_bullet)
        .add_systems(
            RunFixedMainLoop,
            (
                player_movement.in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
            )
        )
        .run();
}