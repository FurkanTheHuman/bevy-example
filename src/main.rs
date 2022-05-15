//! A simplified implementation of the classic game "Breakout"

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::sprite::collide_aabb::collide;
use bevy::{
    core::FixedTimestep,
    math::{const_vec2, const_vec3},
    prelude::*,
    sprite::MaterialMesh2dBundle,
    transform::components::Transform,
};
use bevy_inspector_egui::{Inspectable, InspectorPlugin, WorldInspectorPlugin};
// BACKGROUND_COLOR
const BACKGROUND_COLOR: Color = Color::rgb(0.0, 0.0, 0.0);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        //.add_plugin(InspectorPlugin::<Ball>::new())
        //.add_plugin(FrameTimeDiagnosticsPlugin::default())
        //.insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(PrintTimer(Timer::from_seconds(0.01, true)))
        .add_startup_system(setup)
        //.add_system(ball_movement_system) // causes walls and players to be disaapear
        //.add_system(move_sticks)
        //.add_system(collision_management)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Score(u32);

#[derive(Component, Inspectable, Default)]
struct Ball {
    velocity: Vec3,
}

#[derive(Component)]
struct Wall {
    collider: Collider,
}

#[derive(Component, PartialEq)]
enum Collider {
    Solid,
    Score,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // commands.spawn_bundle(MaterialMesh2dBundle {
    //     mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
    //     transform: Transform::default().with_scale(Vec3::new(30.1, 200.1, 1.1)),
    //     material: materials.add(ColorMaterial::from(Color::PURPLE)),
    //     ..default()
    // });

    // ball
    commands
        .spawn_bundle(SpriteBundle {
            texture: asset_server.load("images/top.png"),
            transform: Transform::default().with_scale(Vec3::new(0.06, 0.06, 1000.0)),
            ..default()
        })
        .insert(Ball {
            velocity: Vec3::new(150.0, 150.0, 0.0),
        });

    //players
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.25, 0.25, 0.75),
                custom_size: Some(Vec2::new(30.0, 200.0)),
                ..default()
            },
            // note: ensure that those numbers don't go out of the screen
            transform: Transform::default().with_translation(Vec3::new(-600.0, 0.0, 0.0)),
            ..default()
        })
        .insert(Name::new("Player1"))
        .insert_bundle((Player, Score(0)));

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.76, 0.12, 0.12),
                custom_size: Some(Vec2::new(30.0, 200.0)),
                ..default()
            },
            // note: ensure that those numbers don't go out of the screen
            transform: Transform::default().with_translation(Vec3::new(600.0, 0.0, 0.0)),
            ..default()
        })
        .insert(Name::new("Player2"))
        .insert_bundle((Player, Score(0)));

    // Walls and goal areas
    commands
        // top
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.76, 0.12, 0.12),
                custom_size: Some(Vec2::new(1280.0, 30.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, 345.0, 0.0)),

            ..default()
        })
        .insert(Wall {
            collider: Collider::Solid,
        })
        .insert(Name::new("WallTop"));
    commands
        // bottom
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.76, 0.12, 0.12),
                custom_size: Some(Vec2::new(1280.0, 30.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(0.0, -345.0, 0.0)),
            ..default()
        })
        .insert(Wall {
            collider: Collider::Solid,
        })
        .insert(Name::new("WallBottom"));
    commands
        // left
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.76, 0.12, 0.12),
                custom_size: Some(Vec2::new(100.0, 720.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(-690.0, 0.0, 0.0)),
            ..default()
        })
        .insert(Wall {
            collider: Collider::Score,
        })
        .insert(Name::new("WallLeft"));
    commands
        // right
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.76, 0.12, 0.12),
                custom_size: Some(Vec2::new(60.0, 720.0)),
                ..default()
            },
            transform: Transform::from_translation(Vec3::new(650.0, 0.0, 0.0)),
            ..default()
        })
        .insert(Wall {
            collider: Collider::Score,
        })
        .insert(Name::new("WallRight"));
}
struct PrintTimer(Timer);
fn move_sticks(
    //mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<PrintTimer>,
    mut query: Query<(&mut Transform, &Name, &mut Score)>,
    keys: Res<Input<KeyCode>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        if keys.pressed(KeyCode::W) {
            for (mut pos, name, mut score) in query.iter_mut() {
                if name == &Name::new("Player1") {
                    pos.translation.y += 10.0;
                }
            }
        }
        if keys.pressed(KeyCode::S) {
            for (mut pos, name, mut score) in query.iter_mut() {
                if name == &Name::new("Player1") {
                    pos.translation.y -= 10.0;
                }
            }
        }
        if keys.pressed(KeyCode::Up) {
            for (mut pos, name, mut score) in query.iter_mut() {
                if name == &Name::new("Player2") {
                    pos.translation.y += 10.0;
                }
            }
        }
        if keys.pressed(KeyCode::Down) {
            for (mut pos, name, mut score) in query.iter_mut() {
                if name == &Name::new("Player2") {
                    pos.translation.y -= 10.0;
                }
            }
        }
    }
}

fn ball_movement_system(time: Res<Time>, mut query: Query<(&Ball, &mut Transform)>) {
    let delta_seconds = f32::min(0.2, time.delta_seconds());
    // for (ball, mut transform) in query.iter_mut() {
    //     transform.translation += ball.velocity * delta_seconds
    // }
}

fn collision_management(
    mut wall_query: Query<(&Wall, &Transform, &Sprite, &Name)>,
    mut ball_query: Query<(&mut Ball, &Transform)>,
) {
    let (mut ball, ball_transform) = ball_query.single_mut();
    // for walls
    for (wall, transform, sprite, name) in wall_query.iter() {
        let collision = collide(
            ball_transform.translation,
            Vec2::splat(0.06),
            transform.translation,
            sprite.custom_size.unwrap(),
        );
        if collision.is_some() {
            println!("collision!!!!");
            if name.as_str() == "WallLeft" {
                ball.velocity.x = -ball.velocity.x;
            } else if name.as_str() == "WallRight" {
                ball.velocity.x = -ball.velocity.x;
            } else if name.as_str() == "WallTop" {
                ball.velocity.y = -ball.velocity.y;
            } else if name.as_str() == "WallBottom" {
                ball.velocity.y = -ball.velocity.y;
            }
            // note: we can calculate collisions for everything seperately
            // but we could also do collision check for everything with Collision in it
            // better that way
        }
    }
}
