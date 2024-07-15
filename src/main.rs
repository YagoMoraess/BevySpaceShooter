use bevy::{prelude::*, math::*, input::*};
use bevy::core_pipeline::bloom::BloomSettings;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use rand::prelude::*;

const PLAYER_SIZE: f32 = 155.;
const PROJECTILE_SIZE: f32 = 65.;
const PLAYER_SPEED: f32 = 400.;

const ENEMY_SIZE: f32 = 100.;
const NUMBER_OF_ENEMIES: i32 = 5;
const ENEMY_SPEED: f32 = 230.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup,(setup))
        .add_systems(FixedUpdate,
                     (player_movement_system,
                      player_shoot_system,
                      projectile_movement_system,
                      enemy_moviment_system,
                      )
        )
        .add_systems(Update, animate_sprite)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy {
    direction: Vec2,
}

#[derive(Component)]
struct Projectile;

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Cooldown {
    timer: Timer,
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

struct Message {
    text: String,
    position: Vec2,
    color: Color,
    font_size: f32,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let ship_texture = asset_server.load("textures/mainShip.png");
    let window = window_query.get_single().unwrap();

    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings {
            intensity: 0.1,
            ..default()
        },
    ));

    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: vec3(0., -300., 0.),
                ..default()
            },
            sprite: Sprite {
                custom_size: Some(vec2(PLAYER_SIZE,PLAYER_SIZE)),
                ..default()
            },
            texture: ship_texture,
            ..default()
        },
        Player,
        Cooldown {
            timer: Timer::from_seconds(0.6, TimerMode::default()),
        }
    ));

    for _ in 0..NUMBER_OF_ENEMIES {
        let enemy_ship_texture = asset_server.load("textures/enemyShip.png");
        let random_x = random::<f32>() * window.width() / 2.;
        let random_y = random::<f32>() * window.height() / 2.;
        println!("{}", window.width());
        println!("{}", window.height());

        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: vec3(random_x, random_y, 10.),
                    rotation: quat(1., 0., 0., 0.),
                    ..default()
                },
                sprite: Sprite {
                    custom_size: Some(vec2(ENEMY_SIZE,ENEMY_SIZE)),
                    ..default()
                },
                texture: enemy_ship_texture,
                ..default()
            },
            Enemy {
                direction: Vec2::new(random::<f32>(),  random::<f32>()).normalize(),
            },
        ));
    }

}

fn player_movement_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &mut Transform)>,
    time: Res<Time>
) {
    for (_player, mut transform) in query.iter_mut() {

        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::A) {
            direction -= Vec3::X;
        }

        if keyboard_input.pressed(KeyCode::D) {
            direction += Vec3::X;
        }

        transform.translation += direction * PLAYER_SPEED * time.delta_seconds();
    }
}

fn player_shoot_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Player, &Transform, &mut Cooldown)>,
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    for (_player, player_transform, mut cooldown) in query.iter_mut() {

        cooldown.timer.tick(time.delta());

        if keyboard_input.pressed(KeyCode::Space) && cooldown.timer.finished() {

            let bullet_texture_handle = asset_server.load("textures/bullet.png");

            let texture_atlas = TextureAtlas::from_grid(bullet_texture_handle, Vec2::new(32.,32.), 4, 1, None, None);
            let texture_atlas_handle = texture_atlases.add(texture_atlas);
            let animation_indices = AnimationIndices { first: 1, last: 3};


            commands.spawn((
                SpriteSheetBundle {
                    transform: Transform {
                        translation: vec3(player_transform.translation.x, player_transform.translation.y + 45., 0.),
                        ..default()
                    },
                    texture_atlas: texture_atlas_handle,
                    sprite: TextureAtlasSprite {
                        custom_size: Some(vec2(PROJECTILE_SIZE, PROJECTILE_SIZE)),
                        index: animation_indices.first,
                        ..default()
                    },
                    ..default()
                },
                Projectile,
                Velocity(Vec3::new(0., 400., 0.)),
                animation_indices,
                AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
            ));

            cooldown.timer.reset();
        }
    }
}

fn projectile_movement_system (
    mut query: Query<(&Projectile, &mut Transform, &Velocity)>,
    time: Res<Time>
) {
    for (_projectile, mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * time.delta_seconds();
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
    }
}

fn enemy_moviment_system(
    mut query: Query<(&mut Enemy, &mut Transform)>,
    time: Res<Time>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let window = window_query.get_single().unwrap();

    for (mut _enemy, mut transform) in query.iter_mut() {
        let direction = Vec3::new(_enemy.direction.x, 0., 0.);
        println!("{}", direction);
        transform.translation += direction * ENEMY_SPEED * time.delta_seconds();

        if transform.translation.x >= window.width() / 2. {
            _enemy.direction.x = -_enemy.direction.x;
        }
        if transform.translation.x <= window.width() - window.width() * 1.5 {
            _enemy.direction.x = -_enemy.direction.x;
        }
    }
}

// fn spawn_message(
//     commands: &mut Commands,
//     message: &str,
//     position: Vec2,
//     color: Color,
//     font_size: f32,
// ) {
//     let message = Text::from_section(
//         &message.to_string(),
//         TextStyle {
//             font_size,
//             color,
//             ..default()
//         }
//     );
//     commands.spawn(()).insert_bundle(Text2dBundle {
//         text: Text {
//             sections: vec![message],
//             ..default()
//         },
//         transform: Transform::from_translation(Vec3::new(position.x, position.y, 0.)),
//         ..default()
//     })
// }