use std::f32::consts::PI;

use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use bevy_hanabi::EffectAsset;
use bevy_rapier2d::prelude::*;
use serde::{Deserialize, Serialize};

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Lifetime>()
            .register_type::<Bullet>()
            .add_system(Bullet::move_bullet)
            .add_startup_system(setup_bullet_trail)
            .add_system(Bullet::cleanup)
            .add_system(despawn_after_lifetime);
    }
}

const SPEED: f32 = 1500.0;

#[derive(Reflect, Component)]
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
                    custom_size: Some(Vec2::splat(3.0)),
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
            if let Some((_, intersection)) = rapier.cast_ray_and_get_normal(
                tf.translation.xy(),
                bullet.dir,
                bullet.dir.length() * time.delta_seconds() / SPEED,
                true,
                QueryFilter::default(),
            ) {
                let debris_dir = bullet.dir.normalize()
                    - 2.0 * bullet.dir.normalize().dot(intersection.normal) * intersection.normal;
                commands.spawn((
                    Name::new("Debris particles"),
                    SpatialBundle {
                        transform: Transform {
                            translation: intersection.point.extend(0.0),
                            rotation: Quat::from_rotation_z(
                                debris_dir.y.atan2(debris_dir.x) - PI / 2.0,
                            ),
                            ..default()
                        },
                        ..default()
                    },
                    ParticleEffect::new(effects.debris.clone()).with_z_layer_2d(Some(0.2)),
                    Lifetime(5.0),
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

#[derive(Reflect, Component)]
struct Lifetime(f32);

fn despawn_after_lifetime(
    mut commands: Commands,
    mut lifetimes: Query<(Entity, &mut Lifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in &mut lifetimes {
        lifetime.0 -= time.delta_seconds();
        if lifetime.0 <= 0.0 {
            commands.entity(entity).despawn();
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
        .init(MyPositionCone3dModifier {
            height: 100.0,
            base_radius: 50.0,
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Reflect, FromReflect, Serialize, Deserialize)]
pub struct MyPositionCone3dModifier {
    /// The cone height along its axis, between the base and top radii.
    pub height: f32,
    /// The cone radius at its base, perpendicularly to its axis.
    pub base_radius: f32,
    /// The cone radius at its truncated top, perpendicularly to its axis.
    /// This can be set to zero to get a non-truncated cone.
    pub top_radius: f32,
    /// The speed of the particles on spawn.
    pub speed: Value<f32>,
    /// The shape dimension to spawn from.
    pub dimension: ShapeDimension,
}

#[typetag::serde]
impl Modifier for MyPositionCone3dModifier {
    fn context(&self) -> ModifierContext {
        ModifierContext::Init
    }

    fn as_init(&self) -> Option<&dyn InitModifier> {
        Some(self)
    }

    fn as_init_mut(&mut self) -> Option<&mut dyn InitModifier> {
        Some(self)
    }

    fn attributes(&self) -> &[&'static Attribute] {
        &[Attribute::POSITION, Attribute::VELOCITY]
    }

    fn boxed_clone(&self) -> BoxedModifier {
        Box::new(*self)
    }
}

#[typetag::serde]
impl InitModifier for MyPositionCone3dModifier {
    fn apply(&self, context: &mut InitContext) {
        context.init_extra += &format!(
            r##"fn init_position_cone3d(transform: mat4x4<f32>, particle: ptr<function, Particle>) {{
    // Truncated cone height
    let h0 = {0};
    // Random height ratio
    let alpha_h = pow(rand(), 1.0 / 3.0);
    // Random delta height from top
    let h = h0 * alpha_h;
    // Top radius
    let rt = {1};
    // Bottom radius
    let rb = {2};
    // Radius at height h
    let r0 = rt + (rb - rt) * alpha_h;
    // Random delta radius
    let alpha_r = sqrt(rand());
    // Random radius at height h
    let r = r0 * alpha_r;
    // Random base angle
    let theta = rand() * tau;
    let cost = cos(theta);
    let sint = sin(theta);
    // Random position relative to truncated cone origin (not apex)
    let x = r * cost;
    let y = h;
    let z = r * sint;
    let p = vec3<f32>(x, y, z);
    let p2 = transform * vec4<f32>(p, 0.0);
    (*particle).{3} = p2.xyz;
    // Emit direction
    let rb2 = rb * alpha_r;
    let pb = vec3<f32>(rb2 * cost, h0, rb2 * sint);
    let dir = transform * vec4<f32>(normalize(pb - p), 0.0);
    // Emit speed
    let speed = 0.0;//{4};
    // Velocity away from cone top/apex
    (*particle).{5} = dir.xyz * speed;
}}
"##,
            self.height.to_wgsl_string(),
            self.top_radius.to_wgsl_string(),
            self.base_radius.to_wgsl_string(),
            Attribute::POSITION.name(),
            self.speed.to_wgsl_string(),
            Attribute::VELOCITY.name(),
        );

        context.init_code += "init_position_cone3d(transform, &particle);\n";
    }
}
