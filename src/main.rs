use bevy::{prelude::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}, window::{close_on_esc, WindowResolution}};

// Ball
const BALL_INITIAL_POSITION: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const BALL_INITIAL_DIRECTION: Vec2 = Vec2::new(0.5, 0.5);
const BALL_SPEED: f32 = 400.0;

// Window
const WINDOW_TITLE: &str = "Pong with your friend";
const WINDOW_W: f32 = 1200.;
const WINDOW_H: f32 = 800.;

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
        .add_systems(Update, close_on_esc)
        .add_systems(Startup, setup)
        .add_systems(Update, apply_velocity)
        .run();
}

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

#[derive(Component)]
struct Ball;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn(
        Camera2dBundle::default()
    );

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle { radius: 25.0 })),
            material: materials.add(Color::ORANGE_RED),
            transform: Transform::from_translation(BALL_INITIAL_POSITION),
            ..default()
        },
        Ball,
        Velocity(BALL_INITIAL_DIRECTION.normalize() * BALL_SPEED),
    ));
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
