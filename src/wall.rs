use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct WallPlugin;

impl Plugin for WallPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_some_walls);
    }
}

fn spawn_some_walls(mut commands: Commands) {
    commands.spawn((
        Name::new("Wall"),
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLUE,
                custom_size: Some(Vec2::new(500.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -100.0, 0.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(250.0, 25.0),
    ));
}
