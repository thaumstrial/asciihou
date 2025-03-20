#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use asciihou::ui::GameUiPlugin;
use asciihou::state::GameState;
use asciihou::resource::{AsciiBoldFont, AsciiFont};
use asciihou::ui::{PlayerGrazeText, PlayerPointsText};
use asciihou::ui::{PlayerBombsText, PlayerPowersText};
use asciihou::ui::PlayerLivesText;
use asciihou::resource::WindowSize;
use asciihou::state::AppState;
use asciihou::ascii_animation::AsciiAnimationPlugin;
use bevy::asset::{AssetMetaCheck, AssetServer};
use bevy::color::palettes::css::*;
use bevy::color::palettes::tailwind::*;
use bevy::DefaultPlugins;
use bevy::ecs::query::QueryData;
use bevy::input::common_conditions::*;
use bevy::text::{JustifyText, Text2d, TextFont, TextLayout};
use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResized};
use bevy_rapier2d::prelude::*;
use bevy::core_pipeline::bloom::Bloom;
use bevy::core_pipeline::bloom::BloomPrefilter;

const PLAYER_RESPAWN_POS: Vec3 = Vec3::new(-200.0, -250.0, 0.0);
#[derive(Component, Clone)]
enum BulletTarget {
    Player,
    Enemy,
}
impl BulletTarget {
    pub fn collision_groups(&self) -> CollisionGroups {
        match self {
            BulletTarget::Player => CollisionGroups::new(Group::GROUP_8, Group::GROUP_1 | Group::GROUP_7),
            BulletTarget::Enemy => CollisionGroups::new(Group::GROUP_2, Group::GROUP_4),
        }
    }
}

#[derive(Component)]
struct EnemyDeathParticle(Timer);
#[derive(Component)]
struct PlayerDeathParticle(Timer);
#[derive(Component)]
struct EnemyHitParticle(Timer);
#[derive(Component, Clone)]
struct HomingBullet {
    speed: f32,
    rotate_speed: f32, // rad/s
}
#[derive(Component, Clone)]
struct SpiralBullet {
    radius: f32,
    radius_growth: f32,
    angular_speed: f32, // rad/s
    angle: f32, // current angle in rad
    forward_velocity: Vec2,
}
#[derive(Component)]
struct GrazingBullet{
    speed_decay: f32,
    original_color: Color,
}
#[derive(Component, Clone)]
struct LaserBullet {
    telegraph_duration: Timer,
    duration: Timer,
    animation_timer: Timer,
}
#[derive(Clone)]
enum BulletType {
    Normal,
    Homing(HomingBullet),
    Spiral(SpiralBullet),
    Laser(LaserBullet),
}
impl BulletType {
    pub fn insert_into(&self, entity: &mut EntityCommands) {
        match self {
            BulletType::Normal => {}
            BulletType::Homing(homing) => { entity.insert(homing.clone()); },
            BulletType::Spiral(spiral) => { entity.insert(spiral.clone()); },
            BulletType::Laser(laser) => { entity.insert(laser.clone()); },
        }
    }
}
#[derive(Bundle)]
struct BulletBundle {
    target: BulletTarget,
    text: Text2d,
    text_font: TextFont,
    text_layout: TextLayout,
    text_color: TextColor,
    collider: Collider,
    rigid_body: RigidBody,
    active_events: ActiveEvents,
    collision_groups: CollisionGroups,
}

#[derive(Clone)]
struct BulletInfo {
    bullet_type: BulletType,
    target: BulletTarget,
    text: Text2d,
    text_font: TextFont,
    text_layout: TextLayout,
    text_color: TextColor,
    collider: Collider,
}
impl BulletInfo {
    pub fn to_bundle(&self) -> BulletBundle {
        BulletBundle {
            target: self.target.clone(),
            text: self.text.clone(),
            text_font: self.text_font.clone(),
            text_layout: self.text_layout.clone(),
            text_color: self.text_color.clone(),
            collider: self.collider.clone(),
            rigid_body: RigidBody::KinematicVelocityBased,
            active_events: ActiveEvents::COLLISION_EVENTS,
            collision_groups: self.target.collision_groups(),
        }
    }
}
#[derive(Component)]
struct JudgePoint;
#[derive(Component)]
struct Player;
#[derive(Component)]
struct GrazeZone;
#[derive(Component)]
struct ShootCooldown(Timer);
#[derive(Component)]
struct Enemy;
#[derive(Component)]
struct Health(i32);
#[derive(Component)]
struct LinearMovement(Vec2);
#[derive(Component)]
struct SingleShoot {
    bullet: BulletInfo,
    velocity: Vec2,
    cooldown: Timer,
    times: i32,
}
#[derive(Component)]
struct FanShoot {
    bullet: BulletInfo,
    num_bullets: i32,
    angle_deg: f32,
    velocity: Vec2,
    cooldown: Timer,
    times: i32,
}
#[derive(Component)]
pub struct PowerItem;
#[derive(Component)]
pub struct PointItem;
#[derive(Component)]
pub struct Invincible(pub Timer);
#[derive(Component)]
struct SupportUnit {
    original_position: Vec3,
    focus_position: Vec3,
}
#[derive(Resource)]
struct ShowColliderDebug(bool);
#[derive(Resource)]
struct EnemySpawnTimer {
    timer: Timer,
}
#[derive(Resource)]
struct PlayerLives(pub i32);
#[derive(Resource)]
struct PlayerBombs(pub i32);
#[derive(Resource)]
struct PlayerPowers(pub i32);
#[derive(Resource)]
struct PlayerPoints(pub i32);
#[derive(Resource)]
struct PlayerGraze(pub i32);
fn attract_items(
    rapier_context: ReadDefaultRapierContext,
    player_query: Query<(Entity, &Transform), With<Player>>,
    mut item_query: Query<(&mut Velocity, &Transform), Or<(With<PowerItem>, With<PointItem>)>>,
    window: Res<WindowSize>,
) {
    const TOP_ZONE_HEIGHT: f32 = 150.0;
    const AUTO_ATTRACT_SPEED: f32 = 400.0;
    const ATTRACT_RADIUS: f32 = 80.0;
    const ATTRACT_SPEED: f32 = 100.0;

    if let Ok((player_entity, player_transform)) = player_query.get_single() {
        let player_pos = player_transform.translation.truncate();
        let shape = Collider::ball(ATTRACT_RADIUS);

        if player_pos.y > window.height / 2.0 - TOP_ZONE_HEIGHT {
            let player_pos = player_transform.translation.truncate();

            for (mut item_velocity, item_transform) in item_query.iter_mut() {
                let dir = (player_pos - item_transform.translation.truncate()).normalize_or_zero();
                item_velocity.linvel = dir * AUTO_ATTRACT_SPEED;
            }

            return;
        } else {
            rapier_context.intersections_with_shape(
                player_pos,
                0.0,
                &shape,
                QueryFilter {
                    exclude_rigid_body: Some(player_entity),
                    groups: Some(CollisionGroups::new(Group::ALL, Group::GROUP_6)),
                    ..default()
                },
                |item_entity| {
                    if let Ok((mut velocity, item_pos)) = item_query.get_mut(item_entity) {
                        let dir = (player_pos - item_pos.translation.truncate()).normalize_or_zero();
                        let distance = player_pos.distance(item_pos.translation.truncate());
                        let strength = 1.0 - (distance / ATTRACT_RADIUS);
                        let attract_speed = ATTRACT_SPEED * (1.0 + strength.clamp(0.0, 1.0));
                        velocity.linvel = dir * attract_speed;
                    }
                    true
                }
            );
        }
    }
}

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
    let margins = " ".repeat(powers.0.to_string().len().max(0));
    for mut text in query.iter_mut() {
        text.0 = format!(" {}Power: {}", margins, num);
    }
}
fn update_graze_text(
    graze: Res<PlayerGraze>,
    mut query: Query<&mut Text2d, With<PlayerGrazeText>>,
) {
    let num = graze.0.to_string();
    let margins = " ".repeat(graze.0.to_string().len().max(0));
    for mut text in query.iter_mut() {
        text.0 = format!(" {}Graze: {}", margins, num);
    }
}
fn update_points_text(
    points: Res<PlayerPoints>,
    mut query: Query<&mut Text2d, With<PlayerPointsText>>,
) {
    let num = points.0.to_string();
    let margins = " ".repeat(points.0.to_string().len().max(0));
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

fn laser_bullet(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut LaserBullet,
        &mut TextColor,
        &mut CollisionGroups,
        &mut Text2d,
        &BulletTarget,
    )>,
) {
    for (
        laser_entity,
        mut laser,
        mut text_color,
        mut groups,
        mut text,
        target,
    ) in query.iter_mut() {
        if !laser.telegraph_duration.finished() {
            // telegraph phase
            laser.telegraph_duration.tick(time.delta());

            let progress = laser.telegraph_duration.elapsed_secs()
                / laser.telegraph_duration.duration().as_secs_f32();
            let eased_alpha = progress.clamp(0.0, 1.0).powf(5.0);

            let mut color = text_color.0;
            color.set_alpha(eased_alpha);
            text_color.0 = color;

            if laser.telegraph_duration.finished() {
                *groups = target.collision_groups();
            }
        } else {
            if laser.duration.finished() {
                commands.entity(laser_entity).despawn();
            } else {
                laser.duration.tick(time.delta());

                if !laser.animation_timer.finished() {
                    laser.animation_timer.tick(time.delta());
                }
                let total_rows = text.0.lines().count();
                let animation_progress = laser.animation_timer.elapsed_secs() / laser.animation_timer.duration().as_secs_f32();
                let duration_progress = laser.duration.elapsed_secs() / laser.duration.duration().as_secs_f32();

                let rows_to_replace = (total_rows as f32 * animation_progress.clamp(0.0, 1.0)).ceil() as usize;
                let rows_decays = (total_rows as f32 * (duration_progress - 0.85).max(0.0) / 0.15).ceil() as usize;
                text.0 = format!("{}{}{}",
                    " \n".repeat(rows_decays),
                    "V\n".repeat(rows_to_replace - rows_decays),
                    "!\n".repeat(total_rows - rows_to_replace));
            }
        }
    }
}

fn homing_bullet_find_nearest<'a>(
    reference: Vec3,
    targets: impl Iterator<Item = &'a Transform>,
) -> Option<&'a Transform> {
    targets.min_by(|a, b| {
        let da = reference.distance_squared(a.translation);
        let db = reference.distance_squared(b.translation);
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    })
}


fn homing_bullet(
    mut query: Query<(
        &mut Velocity,
        &Transform,
        &HomingBullet,
        &BulletTarget,
        Option<&GrazingBullet>,
    )>,
    players: Query<&Transform, With<Player>>,
    enemies: Query<&Transform, With<Enemy>>,
    time: Res<Time>,
) {
    for (
        mut velocity,
        bullet_transform,
        homing,
        target,
        option_graze
    ) in query.iter_mut() {
        let current_dir = velocity.linvel.normalize_or_zero();

        let target_transform = match target {
            BulletTarget::Player => homing_bullet_find_nearest(bullet_transform.translation, players.iter()),
            BulletTarget::Enemy => homing_bullet_find_nearest(bullet_transform.translation, enemies.iter()),
        };

        if let Some(target) = target_transform {
            let desired_dir = (target.translation.truncate() - bullet_transform.translation.truncate()).normalize_or_zero();
            let angle_between = current_dir.angle_to(desired_dir);
            let max_rotate = homing.rotate_speed * time.delta_secs();
            let clamped_angle = angle_between.clamp(-max_rotate, max_rotate);
            let new_dir = current_dir.rotate(Vec2::from_angle(clamped_angle)).normalize_or_zero();

            let decay = if let Some(graze) = option_graze {
                graze.speed_decay
            } else {
                1.0
            };
            velocity.linvel = new_dir * homing.speed * decay;
        }
    }
}

fn spiral_bullet(
    mut query: Query<(&mut SpiralBullet, &mut Velocity, Option<&GrazingBullet>,)>,
    time: Res<Time>,
) {
    for (mut spiral, mut velocity, option_graze) in query.iter_mut() {
        let tangent = Vec2::from_angle(spiral.angle).perp().normalize_or_zero();

        let decay = if let Some(graze) = option_graze {
            graze.speed_decay
        } else {
            1.0
        };

        velocity.linvel = tangent * spiral.radius * spiral.angular_speed + spiral.forward_velocity;
        velocity.linvel *= decay;

        spiral.angle += spiral.angular_speed * time.delta_secs();
        spiral.radius += spiral.radius_growth * time.delta_secs();
    }
}

fn single_shoot(
    mut commands: Commands,
    mut query: Query<(Entity, &GlobalTransform, &mut SingleShoot)>,
    time: Res<Time>,
) {
    for (entity, transform, mut shoot) in query.iter_mut() {
        shoot.cooldown.tick(time.delta());

        if shoot.cooldown.finished() {
            if shoot.times > 0 {
                shoot.times -= 1;
            } else if shoot.times == 0 {
                commands.entity(entity).remove::<SingleShoot>();
                continue
            }

            let spawn_pos = transform.translation();

            let mut bullet_entity = commands.spawn((
                StateScoped(AppState::InGame),
                shoot.bullet.to_bundle(),
                Transform::from_translation(spawn_pos),
                Velocity::linear(shoot.velocity),
            ));

            match shoot.bullet.clone().bullet_type {
                BulletType::Normal => {
                    shoot.bullet.bullet_type.insert_into(&mut bullet_entity);
                }
                BulletType::Homing(_) => {
                    shoot.bullet.bullet_type.insert_into(&mut bullet_entity);
                }
                BulletType::Spiral(mut spiral) => {
                    spiral.forward_velocity = shoot.velocity;
                    BulletType::Spiral(spiral).insert_into(&mut bullet_entity);
                }
                BulletType::Laser(_) => {
                    let rotation = Quat::from_rotation_z(shoot.velocity.normalize_or_zero().to_angle());
                    bullet_entity.insert(Transform {
                        translation: spawn_pos,
                        rotation,
                        ..default()
                    });
                    bullet_entity.insert(CollisionGroups::new(Group::NONE, Group::NONE));
                    bullet_entity.insert(Velocity::zero());
                    shoot.bullet.bullet_type.insert_into(&mut bullet_entity);
                }
            }

            shoot.cooldown.reset();
        }
    }
}

fn fan_shoot(
    mut commands: Commands,
    mut query: Query<(Entity, &GlobalTransform, &mut FanShoot)>,
    time: Res<Time>,
) {
    for (entity, transform, mut shoot) in query.iter_mut() {
        shoot.cooldown.tick(time.delta());
        if !shoot.cooldown.finished() {
            continue;
        }

        if shoot.times > 0 {
            shoot.times -= 1;
        } else if shoot.times == 0 {
            commands.entity(entity).remove::<FanShoot>();
            continue
        }

        let base_direction = shoot.velocity;

        for i in 0..shoot.num_bullets {
            let offset_index = i - (shoot.num_bullets - 1) / 2;
            let angle_rad = (offset_index as f32) * shoot.angle_deg.to_radians();
            let direction = Vec2::from_angle(angle_rad).rotate(base_direction);

            let mut bullet_entity = commands.spawn((
                StateScoped(AppState::InGame),
                shoot.bullet.to_bundle(),
                Transform::from_translation(transform.translation()),
                Velocity::linear(direction),
            ));

            match shoot.bullet.clone().bullet_type {
                BulletType::Normal => {
                    shoot.bullet.bullet_type.insert_into(&mut bullet_entity);
                }
                BulletType::Homing(_) => {
                    shoot.bullet.bullet_type.insert_into(&mut bullet_entity);
                }
                BulletType::Spiral(mut spiral) => {
                    spiral.forward_velocity = direction.normalize_or_zero() * spiral.forward_velocity.length();
                    BulletType::Spiral(spiral).insert_into(&mut bullet_entity);
                }
                BulletType::Laser(_) => {
                    let rotation = Quat::from_rotation_z(direction.normalize_or_zero().to_angle());
                    bullet_entity.insert(Transform {
                        translation: transform.translation(),
                        rotation,
                        ..default()
                    });
                    bullet_entity.insert(CollisionGroups::new(Group::NONE, Group::NONE));
                    bullet_entity.insert(Velocity::zero());
                    shoot.bullet.bullet_type.insert_into(&mut bullet_entity);
                }
            }
        }

        shoot.cooldown.reset();
    }
}

fn spawn_support_units(
    mut commands: Commands,
    font: Res<AsciiFont>,
    powers: Res<PlayerPowers>,
    player_query: Query<(Entity, &Transform), With<Player>>,
    support_query: Query<Entity, With<SupportUnit>>,
) {
    if powers.0 < 1 || support_query.iter().count() >= 2 {
        return;
    }

    if let Ok((player_entity, _)) = player_query.get_single() {
        let offsets = [
            (Vec3::new(30.0, 0.0, 0.0), Vec3::new(15.0, 30.0, 0.0)),
            (Vec3::new(-30.0, 0.0, 0.0), Vec3::new(-15.0, 30.0, 0.0))
        ];

        for (original_offset, focus_offset) in offsets {
            commands.spawn((
                StateScoped(AppState::InGame),
                SupportUnit {
                    original_position: original_offset,
                    focus_position: focus_offset,
                },
                SingleShoot {
                    bullet: BulletInfo {
                        bullet_type: BulletType::Homing(HomingBullet {
                            speed: 800.0,
                            rotate_speed: 1.5,
                        }),
                        target: BulletTarget::Enemy,
                        text: Text2d::new("*"),
                        text_font: TextFont {
                            font: font.0.clone(),
                            font_size: 30.0,
                            ..default()
                        },
                        text_layout: Default::default(),
                        text_color: TextColor(Color::Srgba(PURPLE)),
                        collider: Collider::ball(5.0),
                    },
                    velocity: Vec2::Y * 800.0,
                    cooldown: Timer::from_seconds(0.2, TimerMode::Repeating),
                    times: -1,
                },
                Text2d::new("N"),
                TextFont {
                    font: font.0.clone(),
                    font_size: 30.0,
                    ..default()
                },
                Transform::from_translation(original_offset),
                RigidBody::KinematicVelocityBased,
                Velocity {
                    angvel: 2.0 * (-original_offset.x.signum()),
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
    if powers.0 < 1 {
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
    const SPAWN_CHANCE: f32 = 0.8;
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

            let mut enemy_entity = commands.spawn((
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
                GravityScale(0.0),
                Collider::ball(10.0),
                Velocity::zero(),
                ActiveEvents::COLLISION_EVENTS,
                CollisionGroups::new(Group::GROUP_4, Group::GROUP_1 | Group::GROUP_2),

                Transform::from_translation(spawn_pos.extend(0.0)),
                LinearMovement(movement_vec),
                Health((rand::random::<u32>() % 10 + 1) as i32),
            ));
            enemy_entity.insert(StateScoped(AppState::InGame));


            let bullet_rand = rand::random::<f32>();
            let shoot_rand = rand::random::<f32>();

            let bullet_info = match bullet_rand {
                x if x < 0.25 => BulletInfo {
                    bullet_type: BulletType::Normal,
                    target: BulletTarget::Player,
                    text: Text2d::new("o"),
                    text_font: TextFont {
                        font: font.0.clone(),
                        font_size: 30.0,
                        ..default()
                    },
                    text_layout: Default::default(),
                    text_color: TextColor(Color::Srgba(WHITE)),
                    collider: Collider::ball(5.0),
                },
                x if x < 0.5 => BulletInfo {
                    bullet_type: BulletType::Homing(HomingBullet {
                        speed: shoot_direction.length() * (rand::random::<f32>() * 1.0 + 1.0),
                        rotate_speed: rand::random::<f32>() * 0.4 + 0.1,
                    }),
                    target: BulletTarget::Player,
                    text: Text2d::new("o"),
                    text_font: TextFont {
                        font: font.0.clone(),
                        font_size: 30.0,
                        ..default()
                    },
                    text_layout: Default::default(),
                    text_color: TextColor(Color::Srgba(GOLD)),
                    collider: Collider::ball(5.0),
                },
                x if x < 0.6 => {
                    let laser_length: f32 = 1600.0;
                    let laser_font_size: f32 = 30.0;

                    let collider_x = laser_font_size / 3.5;
                    let collider_y = laser_length / 2.0;

                    let laser_text = "!\n".repeat((laser_length / laser_font_size / 1.2).floor() as usize);
                    let mut initial_color = Color::Srgba(RED_500);
                    initial_color.set_alpha(0.0);

                    BulletInfo {
                        bullet_type: BulletType::Laser(LaserBullet {
                            telegraph_duration: Timer::from_seconds(3.0, TimerMode::Once),
                            duration: Timer::from_seconds(2.0, TimerMode::Once),
                            animation_timer: Timer::from_seconds(0.2, TimerMode::Once),
                        }),
                        target: BulletTarget::Player,
                        text: Text2d::new(laser_text),
                        text_font: TextFont {
                            font: font.0.clone(),
                            font_size: laser_font_size,
                            ..default()
                        },
                        text_layout: Default::default(),
                        text_color: TextColor(initial_color),
                        collider: Collider::cuboid(collider_x, collider_y),
                    }
                },
                _ => BulletInfo {
                    bullet_type: BulletType::Spiral(SpiralBullet {
                        angular_speed: rand::random::<f32>() * 1.0 + 0.5,
                        radius: rand::random::<f32>() * 60.0 + 20.0,
                        radius_growth: rand::random::<f32>() * 10.0 - 5.0,
                        angle: rand::random::<f32>() * std::f32::consts::TAU,
                        forward_velocity: shoot_direction * (rand::random::<f32>() * 0.3 + 0.2),
                    }),
                    target: BulletTarget::Player,
                    text: Text2d::new("o"),
                    text_font: TextFont {
                        font: font.0.clone(),
                        font_size: 30.0,
                        ..default()
                    },
                    text_layout: Default::default(),
                    text_color: TextColor(Color::Srgba(GREEN_400)),
                    collider: Collider::ball(5.0),
                },
            };

            match shoot_rand {
                x if x < 0.6 => {
                    enemy_entity.insert(SingleShoot {
                        bullet: bullet_info,
                        velocity: shoot_direction * (rand::random::<f32>() * 1.0 + 1.0),
                        cooldown: Timer::from_seconds(rand::random::<f32>() * 0.5 + 0.5, TimerMode::Repeating),
                        times: rand::random::<i32>().abs() % 8 + 8,
                    });
                }
                _ => {
                    enemy_entity.insert(FanShoot {
                        bullet: bullet_info,
                        num_bullets: rand::random::<i32>().abs() % 6 + 3,
                        angle_deg: 5.0 + rand::random::<f32>() * 10.0,
                        velocity: shoot_direction * (rand::random::<f32>() * 0.5 + 1.0),
                        cooldown: Timer::from_seconds(rand::random::<f32>() * 0.5 + 0.5, TimerMode::Repeating),
                        times: rand::random::<i32>().abs() % 8 + 8,
                    });
                }
            }
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

fn enemy_hit_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut EnemyHitParticle, &mut TextColor)>,
) {
    for (entity, mut timer, mut color) in query.iter_mut() {
        timer.0.tick(time.delta());

        let progress = timer.0.elapsed_secs() / timer.0.duration().as_secs_f32();
        let alpha = (1.0 - progress.powf(2.0)).clamp(0.0, 1.0);

        color.0.set_alpha(alpha);

        if timer.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn enemy_death_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut EnemyDeathParticle, &mut TextColor, &mut Velocity)>,
) {
    const DECAY_COEFFICIENT: f32 = 1.5;
    for (entity, mut timer, mut color, mut velocity) in query.iter_mut() {
        timer.0.tick(time.delta());

        let progress = timer.0.elapsed_secs() / timer.0.duration().as_secs_f32();
        let alpha = (1.0 - progress).clamp(0.0, 1.0);

        color.0.set_alpha(alpha);

        let decay = 1.0 - time.delta_secs() * DECAY_COEFFICIENT;
        velocity.linvel *= decay.clamp(0.0, 1.0);
        velocity.angvel *= decay.clamp(0.0, 1.0);

        if timer.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}
fn player_death_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PlayerDeathParticle, &mut TextColor, &mut Velocity)>,
) {
    const LINVEL_DECAY_COEFFICIENT: f32 = 0.2;
    const ANGVEL_DECAY_COEFFICIENT: f32 = 1.5;
    for (entity, mut timer, mut color, mut velocity) in query.iter_mut() {
        timer.0.tick(time.delta());

        let progress = timer.0.elapsed_secs() / timer.0.duration().as_secs_f32();
        color.0.set_alpha((1.0 - progress).clamp(0.0, 1.0));

        let linvel_decay = 1.0 - time.delta_secs() * LINVEL_DECAY_COEFFICIENT;
        let angvel_decay = 1.0 - time.delta_secs() * ANGVEL_DECAY_COEFFICIENT;
        velocity.linvel *= linvel_decay.clamp(0.0, 1.0);
        velocity.angvel *= angvel_decay.clamp(0.0, 1.0);

        if timer.0.finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn tick_invincibility(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Invincible, &mut Visibility)>,
) {
    const BLINK_FREQ: f32 = 10.0;
    for (entity, mut inv, mut visibility) in query.iter_mut() {
        inv.0.tick(time.delta());

        let phase = inv.0.elapsed_secs() * BLINK_FREQ * std::f32::consts::TAU;
        let blink_on = phase.sin() >= 0.0;

        *visibility = if blink_on {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        if inv.0.finished() {
            commands.entity(entity).remove::<Invincible>();
            *visibility = Visibility::Visible;
        }
    }
}

fn match_bullet_hit_pair<
    Target: Component,
    D: QueryData
>(
    entity1: Entity,
    entity2: Entity,
    bullets: &Query<(Entity, &BulletTarget, &Transform)>,
    targets: &Query<D, With<Target>>,
) -> Option<(Entity, Entity)> {
    if bullets.get(entity1).is_ok() && targets.get(entity2).is_ok() {
        Some((entity1, entity2))
    } else if bullets.get(entity2).is_ok() && targets.get(entity1).is_ok() {
        Some((entity2, entity1))
    } else {
        None
    }
}

fn bullet_hit(
    mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,

    mut enemies: Query<(Entity, &mut Health, &Transform, Option<&Invincible>), With<Enemy>>,
    player: Query<(Entity, &Transform, Option<&Invincible>), With<Player>>,
    bullets: Query<(Entity, &BulletTarget, &Transform)>,

    mut lives: ResMut<PlayerLives>,
    mut powers: ResMut<PlayerPowers>,
    font: Res<AsciiFont>,
) {
    for event in collision_events.read() {
        match event {
            CollisionEvent::Started(entity1, entity2, _) => {
                if let Some((bullet_entity, enemy_entity)) =
                    match_bullet_hit_pair::<
                        Enemy,
                        (Entity, &mut Health, &Transform, Option<&Invincible>)
                    >(*entity1, *entity2, &bullets, &enemies)
                {
                    if let Ok((enemy_ent, mut health, transform, invincible)) = enemies.get_mut(enemy_entity) {
                        if invincible.is_some() {
                            continue
                        }
                        health.0 -= 1;

                        // generate enemy hit particle
                        let chars = ["(", ")", "<", ">", "{", "}", "[", "]"];
                        let random_char = chars[rand::random::<usize>() % chars.len()];
                        let random_rotation = Quat::from_rotation_z(rand::random::<f32>() * std::f32::consts::TAU);

                        let gray = 0.3 + rand::random::<f32>() * 0.3;
                        let random_color = Color::srgb(gray, gray, gray);

                        if let Ok((_, _, bullet_transform)) = bullets.get(bullet_entity) {
                            commands.spawn((
                                StateScoped(AppState::InGame),
                                EnemyHitParticle(Timer::from_seconds(0.5, TimerMode::Once)),
                                Text2d::new(random_char),
                                TextFont {
                                    font: font.0.clone(),
                                    font_size: 45.0,
                                    ..default()
                                },
                                TextLayout::default(),
                                TextColor(random_color),
                                Transform {
                                    translation: bullet_transform.translation.xy().extend(-5.0), // 在子弹位置
                                    rotation: random_rotation,
                                    ..default()
                                },
                            ));
                        }

                        // enemy death
                        if health.0 <= 0 {
                            let power_count = rand::random::<u32>() % 3 + 1;
                            const ITEM_SPEED: f32 = 50.0;

                            for _ in 0..power_count {

                                commands.spawn((
                                    StateScoped(AppState::InGame),
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
                                    StateScoped(AppState::InGame),
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

                            let num_particles = rand::random::<i32>().abs() % 9 + 8;
                            for _ in 0..num_particles {
                                let char = if rand::random::<bool>() { "0" } else { "1" };
                                let gray = rand::random::<f32>();
                                let angle = rand::random::<f32>() * std::f32::consts::TAU;
                                let speed = rand::random::<f32>() * 50.0 + 50.0;
                                let dir = Vec2::from_angle(angle) * speed;

                                commands.spawn((
                                    StateScoped(AppState::InGame),
                                    EnemyDeathParticle(Timer::from_seconds(rand::random::<f32>() * 2.0 + 1.0, TimerMode::Once)),
                                    Text2d::new(char),
                                    TextFont {
                                        font: font.0.clone(),
                                        font_size: 20.0,
                                        ..default()
                                    },
                                    TextLayout::default(),
                                    TextColor(Color::srgba(gray, gray, gray, 1.0)),
                                    Transform::from_translation(transform.translation),
                                    RigidBody::KinematicVelocityBased,
                                    Velocity {
                                        linvel: dir,
                                        angvel: rand::random::<f32>() * 10.0 - 2.0,
                                    },
                                ));
                            }

                            // before despawn enemy
                            commands.entity(enemy_ent).despawn();
                        }
                    }
                    commands.entity(bullet_entity).despawn();
                } else if let Some((bullet_entity, player_entity )) =
                    match_bullet_hit_pair::<
                        Player,
                        (Entity, &Transform, Option<&Invincible>)
                    >(*entity1, *entity2, &bullets, &player)
                {
                    // player death
                    if let Ok((_, player_transform, invincible)) = player.get_single() {

                        if invincible.is_some() {
                            continue
                        }

                        lives.0 = (lives.0 - 1).max(0);

                        let dropped_power = (powers.0 as f32 * 0.5).ceil() as u32;
                        powers.0 = 0;

                        let player_pos = player_transform.translation.truncate();

                        for _ in 0..dropped_power {
                            let base_angle = std::f32::consts::FRAC_PI_2;
                            let spread_range = std::f32::consts::FRAC_PI_8;

                            let angle = base_angle + (rand::random::<f32>() - 0.5) * 2.0 * spread_range;
                            let speed = rand::random::<f32>() * 100.0 + 100.0;
                            let dir = Vec2::from_angle(angle) * speed;

                            commands.spawn((
                                StateScoped(AppState::InGame),
                                PowerItem,
                                Sprite::from_color(Color::Srgba(RED_400), Vec2::new(20.0, 20.0)),
                                Transform::from_translation(player_pos.extend(-1.0)),
                                Collider::ball(8.0),
                                RigidBody::KinematicVelocityBased,
                                Velocity::linear(dir),
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

                        let num_particles = rand::random::<i32>().abs() % 4 + 8;
                        for _ in 0..num_particles {
                            let hex_str = format!("0x{:02X}", rand::random::<u8>());
                            let hue = 90.0 + rand::random::<f32>() * 60.0;
                            let saturation = 0.6 + rand::random::<f32>() * 0.4;
                            let lightness = 0.4 + rand::random::<f32>() * 0.4;
                            let color = Color::hsl(hue, saturation, lightness);

                            let angle = rand::random::<f32>() * std::f32::consts::TAU;
                            let speed = rand::random::<f32>() * 50.0 + 80.0;
                            let dir = Vec2::from_angle(angle) * speed;

                            commands.spawn((
                                StateScoped(AppState::InGame),
                                PlayerDeathParticle(Timer::from_seconds(rand::random::<f32>() * 1.5 + 2.0, TimerMode::Once)),
                                Text2d::new(hex_str),
                                TextFont {
                                    font: font.0.clone(),
                                    font_size: 20.0,
                                    ..default()
                                },
                                TextLayout::default(),
                                TextColor(color),
                                Transform::from_translation(player_pos.extend(2.0)),
                                RigidBody::KinematicVelocityBased,
                                Velocity {
                                    linvel: dir,
                                    angvel: rand::random::<f32>() * 10.0 - 5.0,
                                },
                            ));
                        }
                    }

                    commands.entity(player_entity)
                        .insert(Visibility::Hidden)
                        .insert(Invincible(Timer::from_seconds(3.0, TimerMode::Once)))
                        .insert(Transform::from_translation(PLAYER_RESPAWN_POS));
                    commands.entity(bullet_entity).despawn();
                }

            }
            _ => {}
        }
    }
}

fn item_hit(
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

fn despawn_out_of_bounds<'a>(
    commands: &mut Commands,
    entities: impl Iterator<Item = (Entity, &'a Transform)>,
    window: &WindowSize,
    extra_margin: f32,
) {
    let max_x = window.width / 2.0 + extra_margin;
    let max_y = window.height / 2.0 + extra_margin;

    for (entity, transform) in entities {
        let pos = transform.translation;
        if pos.x.abs() > max_x || pos.y.abs() > max_y {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn despawn_bullets(
    mut commands: Commands,
    query: Query<
        (Entity, &Transform),
        With<BulletTarget>
    >,
    window: Res<WindowSize>,
) {
    despawn_out_of_bounds(&mut commands, query.iter(), &window, 0.0);
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
    despawn_out_of_bounds(&mut commands, query.iter(), &window, 200.0);
}

fn despawn_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Enemy>>,
    window: Res<WindowSize>,
) {
    despawn_out_of_bounds(&mut commands, query.iter(), &window, 100.0);
}

fn player_bomb(
    mut bombs: ResMut<PlayerBombs>,
) {
    bombs.0 = (bombs.0 - 1).max(0);
}

fn match_graze_bullet_pair<'a>(
    e1: Entity,
    e2: Entity,
    graze_zone: &Query<(), With<GrazeZone>>,
    bullets: &Query<(Entity, &mut Velocity, Option<&GrazingBullet>, &mut TextColor), With<BulletTarget>>,
) -> Option<Entity> {
    if graze_zone.get(e1).is_ok() && bullets.get(e2).is_ok() {
        Some(e2)
    } else if graze_zone.get(e2).is_ok() && bullets.get(e1).is_ok() {
        Some(e1)
    } else {
        None
    }
}


fn player_graze(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    graze_zone: Query<(), With<GrazeZone>>,
    mut bullets: Query<(
            Entity,
            &mut Velocity,
            Option<&GrazingBullet>,
            &mut TextColor,
        ),
        With<BulletTarget>>,
    mut player_graze: ResMut<PlayerGraze>,
) {
    const GRAZE_DECAY: f32 = 0.7;
    const BLOOM_BRIGHTNESS: f32 = 4.0;

    for event in events.read() {
        match event {
            CollisionEvent::Started(e1, e2, _) => {
                if let Some(bullet_entity) = match_graze_bullet_pair(*e1, *e2, &graze_zone, &bullets) {
                    if let Ok((entity, mut velocity, option_graze, mut text_color)) = bullets.get_mut(bullet_entity) {
                        if option_graze.is_none() {
                            velocity.linvel *= GRAZE_DECAY;

                            let original = text_color.0;
                            commands.entity(entity).insert(GrazingBullet {
                                speed_decay: GRAZE_DECAY,
                                original_color: original
                            });

                            text_color.0 = Color::from(original.to_linear() * BLOOM_BRIGHTNESS);
                            player_graze.0 += 1;
                        }
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                if let Some(bullet_entity) = match_graze_bullet_pair(*e1, *e2, &graze_zone, &bullets) {
                    if let Ok((
                        entity,
                        mut velocity,
                        option_graze,
                        mut text_color
                    )) = bullets.get_mut(bullet_entity) {
                        if let Some(graze) = option_graze {
                            velocity.linvel /= graze.speed_decay;
                            text_color.0 = graze.original_color;
                            commands.entity(entity).remove::<GrazingBullet>();
                        }
                    }
                }
            }
        }
    }
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
                let offset = i - (num_bullets - 1) / 2;
                let angle_rad = (offset as f32) * (angle_step_deg as f32).to_radians();
                let rotated_direction = Vec2::from_angle(angle_rad).rotate(BASE_DIRECTION);

                commands.spawn((
                    StateScoped(AppState::InGame),
                    BulletTarget::Enemy,

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

fn show_judge_point(
    mut query: Query<&mut Visibility, With<JudgePoint>>,
) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Visible;
    }
}
fn hide_judge_point(
    mut query: Query<&mut Visibility, With<JudgePoint>>,
) {
    for mut visibility in query.iter_mut() {
        *visibility = Visibility::Hidden;
    }
}
fn support_unit_focus(
    mut query: Query<(&SupportUnit, &mut Transform)>,
    time: Res<Time>,
) {
    const FOCUS_SPEED: f32 = 10.0;
    const POSITION_EPSILON: f32 = 0.5;
    for (support, mut transform) in query.iter_mut() {
        let target = support.focus_position;
        let current = transform.translation;
        if current.distance(target) < POSITION_EPSILON {
            continue;
        }
        let new = current.lerp(target, FOCUS_SPEED * time.delta_secs());
        transform.translation = new;
    }
}
fn support_unit_reset(
    mut query: Query<(&SupportUnit, &mut Transform)>,
    time: Res<Time>,
) {
    const RESET_SPEED: f32 = 10.0;
    const POSITION_EPSILON: f32 = 0.5;
    for (support, mut transform) in query.iter_mut() {
        let target = support.original_position;
        let current = transform.translation;
        if current.distance(target) < POSITION_EPSILON {
            continue;
        }
        let new = current.lerp(target, RESET_SPEED * time.delta_secs());
        transform.translation = new;
    }
}


fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Velocity, With<Player>>,
) {
    const PLAYER_SPEED: f32 = 300.0;
    for mut velocity in player_query.iter_mut() {
        let mut direction = Vec2::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        direction = direction.normalize_or_zero();

        let speed = if keyboard_input.pressed(KeyCode::ShiftLeft) {
            PLAYER_SPEED * 0.5
        } else {
            PLAYER_SPEED
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

        let new_scale = scale_x.min(scale_y);

        for mut projection in query.iter_mut() {
            projection.scale = 1.0 / new_scale;
        }

    }
}
fn pause_game(
    mut next_state: ResMut<NextState<GameState>>,
    mut rapier_query: Query<&mut RapierConfiguration>,
) {
    next_state.set(GameState::Paused);
    if let Ok(mut rapier) = rapier_query.get_single_mut() {
        rapier.physics_pipeline_active = false;
    }
}

fn resume_game(
    mut rapier_query: Query<&mut RapierConfiguration>,
) {
    if let Ok(mut rapier) = rapier_query.get_single_mut() {
        rapier.physics_pipeline_active = true;
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut app_state: ResMut<NextState<AppState>>,
) {

    commands.insert_resource(EnemySpawnTimer {
        timer: Timer::from_seconds(1.0, TimerMode::Repeating),
    });
    commands.insert_resource(PlayerLives(2));
    commands.insert_resource(PlayerBombs(3));
    commands.insert_resource(PlayerPowers(0));
    commands.insert_resource(PlayerGraze(0));
    commands.insert_resource(PlayerPoints(0));

    let font = asset_server.load("font/UbuntuMono-R.ttf");
    commands.insert_resource(AsciiFont(font.clone()));

    let bold_font = asset_server.load("font/UbuntuMono-B.ttf");
    commands.insert_resource(AsciiBoldFont(bold_font.clone()));

    commands.insert_resource(ShowColliderDebug(false));

    commands.insert_resource(WindowSize {
        width: 1280.0,
        height: 720.0,
    });

    commands.spawn((
        Camera2d,
        Camera { hdr: true, ..default() },
        // Tonemapping::default(),
        Bloom {
            prefilter: BloomPrefilter {
                threshold: 0.5,
                ..default()
            },
            ..default()
        },
    ));
    // audio

    app_state.set(AppState::MainMenu);
}

fn setup_game(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    font: Res<AsciiFont>,
) {
    commands.insert_resource(EnemySpawnTimer {
        timer: Timer::from_seconds(1.0, TimerMode::Repeating),
    });
    commands.insert_resource(PlayerLives(2));
    commands.insert_resource(PlayerBombs(3));
    commands.insert_resource(PlayerPowers(0));
    commands.insert_resource(PlayerGraze(0));
    commands.insert_resource(PlayerPoints(0));

    let font_size = 40.0;
    let text_font = TextFont {
        font: font.0.clone(),
        font_size: font_size.clone(),
        ..default()
    };

    commands.spawn((
        StateScoped(AppState::InGame),
        Text2d::new("@"),
        text_font.clone(),
        TextLayout::default(),
        TextColor(Color::Srgba(RED)),

        Player,
        ShootCooldown(Timer::from_seconds(0.1,  TimerMode::Once)),

        RigidBody::Dynamic,
        Sensor,
        GravityScale(0.0),
        Collider::ball(5.0),
        Velocity::zero(),
        Transform::from_translation(PLAYER_RESPAWN_POS),

        ActiveEvents::COLLISION_EVENTS,
        CollisionGroups::new(Group::GROUP_1, Group::GROUP_4 | Group::GROUP_6 | Group::GROUP_8)
    )).with_children(|builder| {
        builder.spawn((
            JudgePoint,
            Text2d::new("·"),
            TextFont {
                font: font.0.clone(),
                font_size: 60.0,
                ..default()
            },
            TextLayout::default(),
            TextColor(Color::Srgba(WHITE)),
            Visibility::Hidden,
            Transform::from_translation(Vec3::new(0.0, 5.0, 1.0)),
        ));
        builder.spawn((
            GrazeZone,
            Collider::ball(15.0),
            CollisionGroups::new(Group::GROUP_7, Group::GROUP_8),
            ActiveEvents::COLLISION_EVENTS,
            Sensor,
        ));
    });

    let a: Handle<AudioSource> = asset_server.load("audio/Character-Encoding-Initiation.ogg");
    commands.spawn(
        AudioPlayer::new(a),
    );
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
        .add_plugins((
            GameUiPlugin,
            AsciiAnimationPlugin
        ))
        .init_state::<AppState>()
        .add_sub_state::<GameState>()
        .enable_state_scoped_entities::<AppState>()
        .enable_state_scoped_entities::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(Update, auto_zoom_camera)
        .add_systems(OnEnter(AppState::InGame), setup_game)
        .add_systems(Update, pause_game.run_if(in_state(GameState::Running).and(input_just_pressed(KeyCode::Escape))))
        .add_systems(OnExit(GameState::Paused), resume_game)
        .add_systems(Update, (
            toggle_debug_render,
            spawn_enemies,
            laser_bullet,
            linear_movement,
            single_shoot,
            fan_shoot,
            tick_invincibility,
            enemy_death_particles,
            player_death_particles,
            enemy_hit_particles,

            player_shoot.run_if(input_pressed(KeyCode::KeyZ)),
            show_judge_point.run_if(input_just_pressed(KeyCode::ShiftLeft)),
            hide_judge_point.run_if(input_just_released(KeyCode::ShiftLeft)),
            support_unit_focus.run_if(input_pressed(KeyCode::ShiftLeft)),
            support_unit_reset.run_if(not(input_pressed(KeyCode::ShiftLeft))),

            update_lives_text.run_if(resource_changed::<PlayerLives>),
            update_bombs_text.run_if(resource_changed::<PlayerBombs>),
            (
                update_powers_text,
                spawn_support_units,
                despawn_support_units,
            ).run_if(resource_changed::<PlayerPowers>),
            update_graze_text.run_if(resource_changed::<PlayerGraze>),
            update_points_text.run_if(resource_changed::<PlayerPoints>)

        ).run_if(in_state(GameState::Running)))
        .add_systems(Update, (
            bullet_hit,
            player_graze.before(bullet_hit).before(laser_bullet),
            item_hit
        ).run_if(on_event::<CollisionEvent>))
        .add_systems(Update, player_bomb.run_if(input_just_pressed(KeyCode::KeyX)))
        .add_systems(FixedUpdate, (
            tick_cooldown_timer,
            despawn_bullets,
            despawn_items,
            despawn_enemies,
            clamp_player_position,
            item_gravity,
            homing_bullet,
            spiral_bullet,
            attract_items
        ).run_if(in_state(GameState::Running)))
        .add_systems(
            RunFixedMainLoop,
            (
                player_movement
                    .in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop)
                    .run_if(in_state(GameState::Running)),
            )
        )
        .run();
}