//! A simplified implementation of the classic game "Breakout"

use bevy::{
    core::FixedTimestep,
    math::{const_vec2, const_vec3},
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
};
use bevy_inspector_egui::{Inspectable, RegisterInspectable, WorldInspectorPlugin};

// Defines the amount of time that should elapse between each physics step.
const TIME_STEP: f32 = 1.0 / 60.0;

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
// The `const_vec3!` macros are needed as functions that operate on floats cannot be constant in Rust.
const PADDLE_SIZE: Vec3 = const_vec3!([120.0, 20.0, 0.0]);
const GAP_BETWEEN_PADDLE_AND_FLOOR: f32 = 60.0;
const PADDLE_SPEED: f32 = 500.0;
// How close can the paddle get to the wall
const PADDLE_PADDING: f32 = 10.0;

// We set the z-value of the ball to 1 so it renders on top in the case of overlapping sprites.
const BALL_STARTING_POSITION: Vec3 = const_vec3!([0.0, -50.0, 1.0]);
const BALL_SIZE: Vec3 = const_vec3!([30.0, 30.0, 0.0]);
const BALL_SPEED: f32 = 400.0;
const INITIAL_BALL_DIRECTION: Vec2 = const_vec2!([0.5, -0.5]);

const WALL_THICKNESS: f32 = 10.0;
// x coordinates
const LEFT_WALL: f32 = -450.;
const RIGHT_WALL: f32 = 450.;
// y coordinates
const BOTTOM_WALL: f32 = -300.;
const TOP_WALL: f32 = 300.;

const BRICK_SIZE: Vec2 = const_vec2!([100., 30.]);
// These values are exact
const GAP_BETWEEN_PADDLE_AND_BRICKS: f32 = 270.0;
const GAP_BETWEEN_BRICKS: f32 = 5.0;
// These values are lower bounds, as the number of bricks is computed
const GAP_BETWEEN_BRICKS_AND_CEILING: f32 = 20.0;
const GAP_BETWEEN_BRICKS_AND_SIDES: f32 = 20.0;

const SCOREBOARD_FONT_SIZE: f32 = 40.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(5.0);

const BACKGROUND_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const PADDLE_COLOR: Color = Color::rgb(0.3, 0.3, 0.7);
const BALL_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);
const BRICK_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard { score: 0 })
        .insert_resource(CheatCodes::new())
        .add_plugin(WorldInspectorPlugin::new())
        .register_inspectable::<Ball>()
        .register_inspectable::<Velocity>()
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_startup_system(setup)
        .add_event::<CollisionEvent>()
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(check_for_collisions)
                .with_system(move_paddle.before(check_for_collisions))
                .with_system(apply_velocity.before(check_for_collisions))
                .with_system(play_collision_sound.after(check_for_collisions)),
        )
        .add_system(update_scoreboard)
        .add_system(text_input)
        .add_system(check_for_cheats)
        .add_system(freeze_ball_when_pressing_space)
        .add_system(bevy::input::system::exit_on_esc_system)
        .run();
}

#[derive(Component)]
struct Paddle;

#[derive(Component, Inspectable)]
struct Ball;

#[derive(Component, Deref, DerefMut, Inspectable)]
struct Velocity(Vec2);

#[derive(Component)]
struct Collider;

#[derive(Default)]
struct CollisionEvent;

#[derive(Component)]
struct Brick;

struct CollisionSound(Handle<AudioSource>);

// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
struct WallBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
    #[bundle]
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

/// Which side of the arena is this wall located on?
enum WallLocation {
    Left,
    Right,
    Bottom,
    Top,
}

impl WallLocation {
    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.),
            WallLocation::Bottom => Vec2::new(0., BOTTOM_WALL),
            WallLocation::Top => Vec2::new(0., TOP_WALL),
        }
    }

    fn size(&self) -> Vec2 {
        let arena_height = TOP_WALL - BOTTOM_WALL;
        let arena_width = RIGHT_WALL - LEFT_WALL;
        // Make sure we haven't messed up our constants
        assert!(arena_height > 0.0);
        assert!(arena_width > 0.0);

        match self {
            WallLocation::Left => Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS),
            WallLocation::Right => Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS),
            WallLocation::Bottom => Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS),
            WallLocation::Top => Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS),
        }
    }
}

impl WallBundle {
    // This "builder method" allows us to reuse logic across our wall entities,
    // making our code easier to read and less prone to bugs when we change the logic
    fn new(location: WallLocation) -> WallBundle {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    // We need to convert our Vec2 into a Vec3, by giving it a z-coordinate
                    // This is used to determine the order of our sprites
                    translation: location.position().extend(0.0),
                    // The z-scale of 2D objects must always be 1.0,
                    // or their ordering will be affected in surprising ways.
                    // See https://github.com/bevyengine/bevy/issues/4149
                    scale: location.size().extend(1.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

// This resource tracks the game's score
struct Scoreboard {
    score: usize,
}
#[derive(Inspectable, Default)]
struct CheatCodes {
    keypresses: Vec<String>,
    list_of_codes: Vec<String>,
}

impl CheatCodes {
    fn new() -> CheatCodes {
        CheatCodes {
            keypresses: Vec::new(),
            list_of_codes: vec!["hesoyam".to_string(), "slow".to_string()],
        }
    }
    fn to_string(&self) -> String {
        let mut string = String::new();
        for code in &self.keypresses {
            string.push_str(&code.clone());
        }
        string
    }
    fn add_code(&mut self, code: String) {
        self.keypresses.push(code);
    }

    fn check_cheat_codes(&mut self) -> Option<String> {
        let string = self.to_string();
        for code in &self.list_of_codes {
            if string.contains(code) {
                println!("Cheat code used: {}", code);
                self.keypresses.clear();

                return Some(code.clone());
            }
        }
        None
    }
}

// Add the game's entities to our world
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    // Sound
    let ball_collision_sound = asset_server.load("sounds/breakout_collision.ogg");
    commands.insert_resource(CollisionSound(ball_collision_sound));

    // Paddle
    let paddle_y = BOTTOM_WALL + GAP_BETWEEN_PADDLE_AND_FLOOR;

    commands
        .spawn()
        .insert(Paddle)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, paddle_y, 0.0),
                scale: PADDLE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: PADDLE_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Collider);

    // Ball
    commands
        .spawn()
        .insert(Ball)
        .insert_bundle(SpriteBundle {
            transform: Transform {
                scale: BALL_SIZE,
                translation: BALL_STARTING_POSITION,
                ..default()
            },
            sprite: Sprite {
                color: BALL_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Velocity(INITIAL_BALL_DIRECTION.normalize() * BALL_SPEED))
        .insert(Name::new("ball"));

    // Scoreboard
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Score: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: TEXT_COLOR,
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: SCORE_COLOR,
                    },
                },
            ],
            ..default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: SCOREBOARD_TEXT_PADDING,
                left: SCOREBOARD_TEXT_PADDING,
                ..default()
            },
            ..default()
        },
        ..default()
    });

    // Walls
    commands.spawn_bundle(WallBundle::new(WallLocation::Left));
    commands.spawn_bundle(WallBundle::new(WallLocation::Right));
    commands.spawn_bundle(WallBundle::new(WallLocation::Bottom));
    commands.spawn_bundle(WallBundle::new(WallLocation::Top));

    // Bricks
    // Negative scales result in flipped sprites / meshes,
    // which is definitely not what we want here
    assert!(BRICK_SIZE.x > 0.0);
    assert!(BRICK_SIZE.y > 0.0);

    let total_width_of_bricks = (RIGHT_WALL - LEFT_WALL) - 2. * GAP_BETWEEN_BRICKS_AND_SIDES;
    let bottom_edge_of_bricks = paddle_y + GAP_BETWEEN_PADDLE_AND_BRICKS;
    let total_height_of_bricks = TOP_WALL - bottom_edge_of_bricks - GAP_BETWEEN_BRICKS_AND_CEILING;

    assert!(total_width_of_bricks > 0.0);
    assert!(total_height_of_bricks > 0.0);

    // Given the space available, compute how many rows and columns of bricks we can fit
    let n_columns = (total_width_of_bricks / (BRICK_SIZE.x + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_rows = (total_height_of_bricks / (BRICK_SIZE.y + GAP_BETWEEN_BRICKS)).floor() as usize;
    let n_vertical_gaps = n_columns - 1;

    // Because we need to round the number of columns,
    // the space on the top and sides of the bricks only captures a lower bound, not an exact value
    let center_of_bricks = (LEFT_WALL + RIGHT_WALL) / 2.0;
    let left_edge_of_bricks = center_of_bricks
        // Space taken up by the bricks
        - (n_columns as f32 / 2.0 * BRICK_SIZE.x)
        // Space taken up by the gaps
        - n_vertical_gaps as f32 / 2.0 * GAP_BETWEEN_BRICKS;

    // In Bevy, the `translation` of an entity describes the center point,
    // not its bottom-left corner
    let offset_x = left_edge_of_bricks + BRICK_SIZE.x / 2.;
    let offset_y = bottom_edge_of_bricks + BRICK_SIZE.y / 2.;

    for row in 0..n_rows {
        for column in 0..n_columns {
            let brick_position = Vec2::new(
                offset_x + column as f32 * (BRICK_SIZE.x + GAP_BETWEEN_BRICKS),
                offset_y + row as f32 * (BRICK_SIZE.y + GAP_BETWEEN_BRICKS),
            );

            // brick
            commands
                .spawn()
                .insert(Brick)
                .insert_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: BRICK_COLOR,
                        ..default()
                    },
                    transform: Transform {
                        translation: brick_position.extend(1.0),
                        scale: Vec3::new(BRICK_SIZE.x, BRICK_SIZE.y, 1.0),
                        ..default()
                    },
                    ..default()
                })
                .insert(Collider);
        }
    }
}

fn text_input(mut char_evr: EventReader<ReceivedCharacter>, mut cheat: ResMut<CheatCodes>) {
    for ev in char_evr.iter() {
        cheat.add_code(ev.char.to_string());
    }
}

fn check_for_cheats(
    mut cheat: ResMut<CheatCodes>,
    mut scoreboard: ResMut<Scoreboard>,
    mut velocity: Query<&mut Velocity>,
) {
    let code = cheat.check_cheat_codes();
    if let Some(code) = code {
        if code == "hesoyam" {
            scoreboard.score += 1000;
        }
        if code == "slow" {
            // make the blow slow
            for mut vel in velocity.iter_mut() {
                vel.x *= 0.5;
                vel.y *= 0.5;
            }
        }
    }
}

fn move_paddle(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>,
    mut score: ResMut<Scoreboard>,
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;
    let mut vertical_direction = 0.0;
    if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
        direction -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
        direction += 1.0;
    }

    if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
        vertical_direction = 1.0;
    }

    if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
        vertical_direction = -1.0;
    }

    // add hack: if user press M key, the points will increase +100000
    if keyboard_input.just_pressed(KeyCode::P) {
        score.score += 100000;
    }

    // Calculate the new horizontal paddle position based on player input
    let new_paddle_position = paddle_transform.translation.x + direction * PADDLE_SPEED * TIME_STEP;
    let new_paddle_position_y =
        paddle_transform.translation.y + vertical_direction * PADDLE_SPEED * TIME_STEP;

    // Update the paddle position,
    // making sure it doesn't cause the paddle to leave the arena
    let left_bound = LEFT_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.x / 2.0 + PADDLE_PADDING;
    let right_bound = RIGHT_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.x / 2.0 - PADDLE_PADDING;
    let top_bound = TOP_WALL - WALL_THICKNESS / 2.0 - PADDLE_SIZE.y / 2.0 - PADDLE_PADDING;
    let bottom_bound = BOTTOM_WALL + WALL_THICKNESS / 2.0 + PADDLE_SIZE.y / 2.0 + PADDLE_PADDING;

    paddle_transform.translation.x = new_paddle_position.clamp(left_bound, right_bound);
    paddle_transform.translation.y = new_paddle_position_y.clamp(bottom_bound, top_bound);
}

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x * TIME_STEP;
        transform.translation.y += velocity.y * TIME_STEP;
    }
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = format!("${}", scoreboard.score);
}

fn check_for_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(Entity, &Transform, Option<&Brick>), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();
    let ball_size = ball_transform.scale.truncate();

    // check collision with walls
    for (collider_entity, transform, maybe_brick) in collider_query.iter() {
        let collision = collide(
            ball_transform.translation,
            ball_size,
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            // Sends a collision event so that other systems can react to the collision
            collision_events.send_default();

            // Bricks should be despawned and increment the scoreboard on collision
            if maybe_brick.is_some() {
                scoreboard.score += 1;
                commands.entity(collider_entity).despawn();
            }

            // reflect the ball when it collides
            let mut reflect_x = false;
            let mut reflect_y = false;

            // only reflect if the ball's velocity is going in the opposite direction of the
            // collision
            match collision {
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                Collision::Inside => { /* do nothing */ }
            }

            // reflect velocity on the x-axis if we hit something on the x-axis
            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            // reflect velocity on the y-axis if we hit something on the y-axis
            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

fn freeze_ball_when_pressing_space(
    mut keyboard_input: Res<Input<KeyCode>>,
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    mut speed_state: Local<(f32, f32)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        let mut ball_velocity = ball_query.single_mut();
        *speed_state = (ball_velocity.0.x, ball_velocity.0.y);
        ball_velocity.0.x = 0.0;
        ball_velocity.0.y = 0.0;
    }
    if keyboard_input.just_released(KeyCode::Space) {
        let mut ball_velocity = ball_query.single_mut();
        ball_velocity.0.x = speed_state.0;
        ball_velocity.0.y = speed_state.1;
    }
}

fn play_collision_sound(
    mut collision_events: EventReader<CollisionEvent>,
    audio: Res<Audio>,
    sound: Res<CollisionSound>,
    score: Res<Scoreboard>,
) {
    // Play a sound once per frame if a collision occurred. `count` consumes the
    // events, preventing them from triggering a sound on the next frame.
    if collision_events.iter().count() > 0 {
        audio.play(sound.0.clone());
        if score.score % 10 == 0 && score.score != 0 {
            audio.play(sound.0.clone());
        }
    }
}