use std::f32::consts::{PI, TAU};

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::user_input::InputKind;

use crate::bullet::{Bullet, BulletEffects};
use crate::mouse::MousePos;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<Action>::default())
            .add_startup_system(spawn_player)
            .add_system(move_player)
            .add_system(update_player_pos.after(move_player))
            .add_system(orient_player.after(update_player_pos))
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
        let mut input_map = InputMap::new([
            (KeyCode::W, Action::Up),
            (KeyCode::A, Action::Left),
            (KeyCode::S, Action::Down),
            (KeyCode::D, Action::Right),
            (KeyCode::F, Action::Shoot),
        ]);
        input_map.insert(InputKind::Mouse(MouseButton::Left), Action::Shoot);
        input_map
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct LowerBody;

#[derive(Component)]
struct UpperBody;

#[derive(Component, Deref, DerefMut)]
struct MoveDir(Vec2);

fn shoot(
    mut commands: Commands,
    player: Query<(&Transform, &ActionState<Action>), With<Player>>,
    bullet_effects: Res<BulletEffects>,
) {
    let (tf, actions) = player.single();

    if actions.just_pressed(Action::Shoot) {
        Bullet::spawn(
            &mut commands,
            tf.translation - 50.0 * tf.right(),
            -tf.right().xy(),
            bullet_effects.trail.clone(),
        );
    }
}

fn orient_legs(
    player: Query<(&Transform, &MoveDir), With<Player>>,
    mut legs: Query<&mut Transform, (Without<Player>, Without<UpperBody>, With<LowerBody>)>,
    mut angle: Local<f32>,
) {
    let (player_tf, move_dir) = player.single();
    let mut legs_tf = legs.single_mut();

    if **move_dir != Vec2::ZERO {
        *angle = move_dir.y.atan2(move_dir.x);
        if move_dir.dot(-player_tf.right().xy()) < 0.0 {
            *angle = (*angle + PI).rem_euclid(TAU);
        }
    }

    let leg_diff = (*angle - player_tf.rotation.to_euler(EulerRot::ZYX).0).rem_euclid(TAU) - PI;
    if leg_diff.abs() > PI / 4.0 {
        *angle -= PI / 4.0 * leg_diff.signum();
    }

    legs_tf.rotation = player_tf.rotation.inverse() * Quat::from_rotation_z(*angle);
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
    const ANGULAR_SPEED: f32 = 180.0 / 180.0 * PI;
    let movement = (ANGULAR_SPEED / tf.rotation.angle_between(target_angle) * time.delta_seconds())
        .clamp(0.0, 1.0);

    tf.rotation = tf.rotation.slerp(target_angle, movement.sqrt());
}

fn move_player(
    mut player: Query<(&mut MoveDir, &ActionState<Action>), With<Player>>,
    time: Res<Time>,
) {
    let (mut move_dir, actions) = player.single_mut();
    let mut dir = Vec2::ZERO;
    if actions.pressed(Action::Up) {
        dir += Vec2::Y;
    }
    if actions.pressed(Action::Down) {
        dir += Vec2::NEG_Y;
    }
    if actions.pressed(Action::Left) {
        dir += Vec2::NEG_X;
    }
    if actions.pressed(Action::Right) {
        dir += Vec2::X;
    }
    let speed = 200. * time.delta_seconds();

    dir = speed * dir.normalize_or_zero();

    **move_dir = dir;
}

fn update_player_pos(mut player: Query<(&mut Transform, &MoveDir), With<Player>>) {
    let (mut tf, dir) = player.single_mut();
    tf.translation += dir.extend(0.0);
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
            MoveDir(Vec2::ZERO),
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
            parent
                .spawn((Name::new("Lower body"), SpatialBundle::default(), LowerBody))
                .with_children(|parent| {
                    parent.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::GRAY,
                            custom_size: Some(Vec2::new(25.0, 80.0)),
                            ..default()
                        },
                        transform: Transform::from_xyz(12.5, 0.0, 0.0),
                        ..default()
                    });
                    parent.spawn(SpriteBundle {
                        sprite: Sprite {
                            color: Color::DARK_GRAY,
                            custom_size: Some(Vec2::new(25.0, 100.0)),
                            ..default()
                        },
                        transform: Transform::from_xyz(-12.5, 0.0, 0.0),
                        ..default()
                    });
                });
        });
}
