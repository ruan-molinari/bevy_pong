use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume}, prelude::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}, window::close_on_esc
};
use iyes_perf_ui::{PerfUiCompleteBundle, PerfUiPlugin};

// Ball
const BALL_INITIAL_POSITION:  Vec3 = Vec3::new(200.0, 0.0, 1.0);
const BALL_INITIAL_DIRECTION: Vec2 = Vec2::new(0.5, 0.0);
const BALL_SPEED: f32 = 400.0;
const BALL_DIAMETER: f32 = 25.0;

// Window
const WINDOW_TITLE: &str = "Pong with your friend";
const WINDOW_W: f32 = 1200.0;
const WINDOW_H: f32 = 800.0;

// Walls
const WALL_THICKNESS: f32 = 10.0;

const ARENA_W: f32 = 1200.0;
const ARENA_H: f32 = 800.0;

const TOP_WALL:    f32 = ARENA_H / 2.0;
const BOTTOM_WALL: f32 = -(ARENA_H / 2.0);
const LEFT_WALL:   f32 = -(ARENA_W / 2.0);
const RIGHT_WALL:  f32 = ARENA_W / 2.0;

// Paddle
const PADDLE_W: f32 = 10.0;
const PADDLE_H: f32 = 100.0;
const PADDLE_DISTANCE_TO_WALL: f32 = 20.0;
const PADDLE_SPEED: f32 = 300.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(Window {
                    title: WINDOW_TITLE.into(),
                    resolution: (WINDOW_W, WINDOW_H).into(),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }
        ))
        .add_plugins(PerfUiPlugin)
        .add_systems(Update, close_on_esc)
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate, (
                apply_velocity,
                move_paddle,
                check_for_collision,
            ).chain() // chaining systems together runs them in order
        )
        .run();
}

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Paddle;

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct Collider;

#[derive(Event, Default)]
struct CollisionEvent;

// This bundle is a collection of the components that define a "wall" in our game
#[derive(Bundle)]
struct WallBundle {
    // You can nest bundles inside of other bundles like this
    // Allowing you to compose their functionality
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
        match self {
            WallLocation::Left | WallLocation::Right => {
                Vec2::new(WALL_THICKNESS, ARENA_H + WALL_THICKNESS)
            }
            WallLocation::Bottom | WallLocation::Top => {
                Vec2::new(ARENA_W + WALL_THICKNESS, WALL_THICKNESS)
            }
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
                    color: Color::BISQUE,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn(PerfUiCompleteBundle::default());

    // Spawn Ball
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle { radius: BALL_DIAMETER })),
            material: materials.add(Color::ORANGE_RED),
            transform: Transform::from_translation(BALL_INITIAL_POSITION),
            ..default()
        },
        Ball,
        Velocity(BALL_INITIAL_DIRECTION.normalize() * BALL_SPEED),
    ));
    // Paddles
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle { half_size: Vec2::new(PADDLE_W, PADDLE_H) })),
            material: materials.add(Color::ALICE_BLUE),
            transform: Transform::from_translation(Vec3::new(
                    LEFT_WALL + WALL_THICKNESS + PADDLE_DISTANCE_TO_WALL + PADDLE_W / 2.0,
                    0.0,
                    1.0,
                )),
            ..default()
        },
        Paddle,
        Collider,
        Velocity(Vec2::new(0.0, 0.0))
    ));

    // Spawn Walls
    commands.spawn(WallBundle::new(WallLocation::Top));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));
}

fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>
) {
    for (mut transform, velocity) in &mut query {
        transform.translation.x += velocity.x * time.delta_seconds();
        transform.translation.y += velocity.y * time.delta_seconds();
    }
}

fn check_for_collision(
    mut ball_query: Query<(&mut Velocity, &Transform), With<Ball>>,
    collider_query: Query<(Entity, &Transform), With<Collider>>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let (mut ball_velocity, ball_transform) = ball_query.single_mut();

    // check collision with Walls
    for (collider_entity, transform) in &collider_query {
        let collision = collide_with_side(
            // `BALL_DIAMETER * 0.8` makes the ball overlap 20% before considerinc a
            // collision, this makes the "bounce" feel more natural
            BoundingCircle::new(ball_transform.translation.truncate(), BALL_DIAMETER * 0.8),
            Aabb2d::new(
                transform.translation.truncate(),
                transform.scale.truncate() / 2.
            )
        );


        if let Some(collision) = collision {
            collision_events.send_default();

            let mut reflect_x = false;
            let mut reflect_y = false;

            match collision {
                Collision::Top => reflect_y = ball_velocity.y < 0.0,
                Collision::Bottom => reflect_y = ball_velocity.y > 0.0,
                Collision::Left => reflect_x = ball_velocity.x > 0.0,
                Collision::Right => reflect_x = ball_velocity.x < 0.0,
            }

            // TODO: collision on x-axis scores a point to opposite side and starts a new round
            if reflect_x {
                ball_velocity.x = -ball_velocity.x;
            }

            if reflect_y {
                ball_velocity.y = -ball_velocity.y;
            }
        }
    }
}

fn move_paddle(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Paddle>>,
    time: Res<Time>
) {
    let mut paddle_transform = query.single_mut();
    let mut direction = 0.0;

    if keyboard_input.pressed(KeyCode::ArrowUp) {
        direction += 1.0;
    }

    if keyboard_input.pressed(KeyCode::ArrowDown) {
        direction -= 1.0;
    }

    let new_paddle_position = 
        paddle_transform.translation.y + direction * time.delta_seconds() * PADDLE_SPEED;

    let top_bound = TOP_WALL + WALL_THICKNESS / 2. - PADDLE_H - PADDLE_DISTANCE_TO_WALL;
    let bottom_bound = BOTTOM_WALL - WALL_THICKNESS / 2. + PADDLE_H + PADDLE_DISTANCE_TO_WALL;

    paddle_transform.translation.y = new_paddle_position.clamp(bottom_bound, top_bound);
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Collision {
    Left,
    Right,
    Top,
    Bottom
}

fn collide_with_side(ball: BoundingCircle, wall: Aabb2d,) -> Option<Collision> {
    if !ball.intersects(&wall) {
        return None;
    }

    let closest = wall.closest_point(ball.center());
    let offset = ball.center() - closest;
    let side = if offset.x.abs() > offset.y.abs() {
        if offset.x > 0. {
            Collision::Right
        } else {
            Collision::Left
        }
    } else if offset.y > 0.0 {
        Collision::Top
    } else {
        Collision::Bottom
    };

    Some(side)
}
