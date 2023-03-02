use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_rapier2d::prelude::*;

use self::bullet::BulletPlugin;
use self::camera::GameCameraPlugin;
use self::mouse::MousePositionPlugin;
use self::player::PlayerPlugin;
use self::wall::WallPlugin;

mod bullet;
mod camera;
mod mouse;
mod player;
mod wall;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::window::WindowPlugin {
            window: WindowDescriptor {
                width: 1200.,
                height: 800.,
                position: WindowPosition::Centered,
                title: "Rust game!".into(),
                resizable: false,
                ..default()
            },
            ..default()
        }))
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugin(InspectableRapierPlugin)
        .add_plugin(HanabiPlugin)
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(GameCameraPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(BulletPlugin)
        .add_plugin(WallPlugin)
        .add_plugin(MousePositionPlugin)
        .run();
}
