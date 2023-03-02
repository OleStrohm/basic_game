use std::f32::consts::PI;

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use leafwing_input_manager::prelude::*;

use crate::bullet::{Bullet, BulletEffects};
use crate::mouse::MousePos;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<Action>::default())
            .add_startup_system(spawn_player)
            .add_system(move_player)
            .add_system(orient_player.after(move_player))
            .add_system(orient_legs.after(orient_player))
            .add_system(shoot);
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum Action {
    Up,
    Down,
    Left,
    Right,
    Shoot,
}

impl Action {
    fn player_one() -> InputMap<Self> {
        InputMap::new([
            (KeyCode::W, Action::Up),
            (KeyCode::A, Action::Left),
            (KeyCode::S, Action::Down),
            (KeyCode::D, Action::Right),
            (KeyCode::F, Action::Shoot),
        ])
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct LowerBody;

#[derive(Component)]
struct UpperBody;

fn shoot(
    mut commands: Commands,
    player: Query<(&Transform, &ActionState<Action>), With<Player>>,
    bullet_effects: Res<BulletEffects>,
    mpos: Res<MousePos>,
) {
    let (tf, actions) = player.single();

    if actions.just_pressed(Action::Shoot) {
        Bullet::spawn(
            &mut commands,
            tf.translation,
            mpos.0 - tf.translation.xy(),
            bullet_effects.trail.clone(),
        );
    }
}

fn orient_legs(
    player: Query<&Transform, With<Player>>,
    mut legs: Query<&mut Transform, (Without<Player>, Without<UpperBody>, With<LowerBody>)>,
    mut state: Local<(f32, Vec3)>,
) {
    let tf = player.single();
    let mut legs_tf = legs.single_mut();

    if tf.translation != state.1 {
        let movement_dir = tf.translation - state.1;
        state.0 = movement_dir.y.atan2(movement_dir.x) + PI / 2.0;
        state.1 = tf.translation;
    }

    legs_tf.rotation = tf.rotation.inverse() * Quat::from_rotation_z(state.0);
}

fn orient_player(
    mut player: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
    mpos: Res<MousePos>,
) {
    let mut tf = player.single_mut();

    let look_dir = tf.translation.xy() - mpos.0;
    let target_angle = Quat::from_rotation_z(look_dir.y.atan2(look_dir.x));

    // Limit speed of rotation
    const ANGULAR_SPEED: f32 = 360.0 / 180.0 * PI;
    let movement = (ANGULAR_SPEED / tf.rotation.angle_between(target_angle) * time.delta_seconds())
        .clamp(0.0, 1.0);

    tf.rotation = tf.rotation.slerp(target_angle, movement);
}

fn move_player(
    mut player: Query<(&mut Transform, &ActionState<Action>), With<Player>>,
    time: Res<Time>,
) {
    let (mut tf, actions) = player.single_mut();
    let speed = 200. * time.delta_seconds();

    if actions.pressed(Action::Up) {
        tf.translation += Vec3::Y * speed;
    }
    if actions.pressed(Action::Down) {
        tf.translation += Vec3::NEG_Y * speed;
    }
    if actions.pressed(Action::Left) {
        tf.translation += Vec3::NEG_X * speed;
    }
    if actions.pressed(Action::Right) {
        tf.translation += Vec3::X * speed;
    }
}

fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn((
            Name::new("Player"),
            Player,
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 300.0, 0.0),
                ..default()
            },
            InputManagerBundle {
                input_map: Action::player_one(),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("Upper body"),
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(Mesh::from(shape::RegularPolygon::new(50.0, 3)))
                        .into(),
                    material: materials.add(ColorMaterial::from(Color::PURPLE)),
                    transform: Transform {
                        rotation: Quat::from_rotation_z(PI / 2.0),
                        translation: Vec3::new(0.0, 0.0, 0.1),
                        ..default()
                    },
                    ..default()
                },
                UpperBody,
            ));
            parent.spawn((
                Name::new("Lower body"),
                SpriteBundle {
                    sprite: Sprite {
                        color: Color::BEIGE,
                        custom_size: Some(Vec2::new(100.0, 50.0)),
                        ..default()
                    },
                    ..default()
                },
                LowerBody,
            ));
        });
}
