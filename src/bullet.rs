use std::f32::consts::PI;

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_hanabi::EffectAsset;
use bevy_rapier2d::prelude::*;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Bullet::move_bullet)
            .add_startup_system(setup_bullet_trail)
            .add_system(Bullet::cleanup);
    }
}
const SPEED: f32 = 1000.0;

#[derive(Component)]
pub struct Bullet {
    lifetime: f32,
    dir: Vec2,
}

impl Bullet {
    pub fn spawn(commands: &mut Commands, pos: Vec3, dir: Vec2, trail: Handle<EffectAsset>) {
        commands.spawn((
            Name::new("Bullet"),
            Bullet {
                lifetime: 1.0,
                dir: dir.normalize() * SPEED,
            },
            SpriteBundle {
                sprite: Sprite {
                    color: Color::YELLOW,
                    custom_size: Some(Vec2::splat(10.0)),
                    ..default()
                },
                transform: Transform::from_translation(pos),
                ..default()
            },
            ParticleEffect::new(trail).with_z_layer_2d(Some(0.1)),
        ));
    }

    fn move_bullet(
        mut commands: Commands,
        mut bullets: Query<(Entity, &mut Transform, &mut Bullet)>,
        rapier: Res<RapierContext>,
        time: Res<Time>,
        effects: Res<BulletEffects>,
    ) {
        for (entity, mut tf, mut bullet) in &mut bullets {
            if let Some((_, toi)) = rapier.cast_ray(
                tf.translation.xy(),
                bullet.dir,
                bullet.dir.length() * time.delta_seconds() / SPEED,
                true,
                QueryFilter::default(),
            ) {
                let hit_pos = tf.translation.xy(); // + toi * bullet.dir;
                commands.spawn((
                    SpatialBundle {
                        transform: Transform {
                            translation: hit_pos.extend(0.0),
                            rotation: Quat::from_rotation_z(-PI/2.0 + (-bullet.dir.y).atan2(bullet.dir.x)),
                            ..default()
                        },
                        ..default()
                    },
                    ParticleEffect::new(effects.debris.clone()).with_z_layer_2d(Some(0.2)),
                ));
                commands.entity(entity).despawn();
            } else {
                tf.translation += bullet.dir.extend(0.0) * time.delta_seconds();
                bullet.lifetime -= time.delta_seconds();
            }
        }
    }

    fn cleanup(mut commands: Commands, bullets: Query<(Entity, &Bullet)>) {
        for (entity, bullet) in &bullets {
            if bullet.lifetime <= 0.0 {
                commands.entity(entity).despawn();
            }
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct BulletEffects {
    pub trail: Handle<EffectAsset>,
    pub debris: Handle<EffectAsset>,
}

fn setup_bullet_trail(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(0.5, 0.5, 1.0, 1.0));
    gradient.add_key(1.0, Vec4::new(0.5, 0.5, 1.0, 0.0));

    let spawner = Spawner::rate(300.0.into());
    let trail = effects.add(
        EffectAsset {
            name: "Bullet trail".into(),
            capacity: 4096,
            spawner,
            ..default()
        }
        .init(InitPositionCircleModifier {
            radius: 3.0,
            dimension: ShapeDimension::Surface,
            ..default()
        })
        .init(InitVelocityCircleModifier {
            speed: 1.0.into(),
            ..default()
        })
        .init(InitLifetimeModifier {
            lifetime: Value::Single(0.2),
        })
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::constant(Vec2::splat(1.0)),
        })
        .render(ColorOverLifetimeModifier { gradient }),
    );

    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(1.0, 1.0, 1.0, 1.0));
    gradient.add_key(1.0, Vec4::new(1.0, 1.0, 1.0, 0.0));

    let spawner = Spawner::once(20.0.into(), true);

    let debris = effects.add(
        EffectAsset {
            name: "Debris".into(),
            capacity: 4096,
            spawner,
            ..default()
        }
        .init(PositionCone3dModifier {
            height: 10.0,
            base_radius: 10.0,
            top_radius: 0.0,
            speed: Value::Uniform((100.0, 500.0)),
            dimension: ShapeDimension::Surface,
            ..default()
        })
        .init(InitLifetimeModifier {
            lifetime: Value::Single(0.2),
        })
        .render(SizeOverLifetimeModifier {
            gradient: Gradient::constant(Vec2::splat(1.0)),
        })
        .render(ColorOverLifetimeModifier { gradient }),
    );
    commands.insert_resource(BulletEffects { trail, debris });
}
