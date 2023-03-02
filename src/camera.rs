use bevy::prelude::*;

pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(spawn_camera);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2dBundle::default()));
}
