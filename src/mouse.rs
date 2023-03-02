use bevy::prelude::*;

pub struct MousePositionPlugin;

impl Plugin for MousePositionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MousePos>()
            .add_system_to_stage(CoreStage::PreUpdate, update_mouse_pos);
    }
}

#[derive(Resource, Default)]
pub struct MousePos(pub Vec2);

fn update_mouse_pos(windows: Res<Windows>, mut mouse_pos: ResMut<MousePos>) {
    let window = windows.get_primary().unwrap();
    let Some(mpos) = window.cursor_position() else { return };
    *mouse_pos = MousePos(mpos - Vec2::new(window.width(), window.height()) / 2.0);
}
