use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

use bevy::{
    prelude::*,
    render::{
        camera,
        mesh::{Mesh, Mesh3d, VertexAttributeValues},
    },
    scene::SceneRoot,
};
use bevy_rapier3d::plugin::RapierPhysicsPlugin;
use bevy_rapier3d::prelude::*;

pub mod command_handler;
pub mod controller_server;
pub mod ws_handler;

#[derive(Resource)]
pub struct WsRuntime(tokio::runtime::Runtime);

#[derive(Component, Clone, Copy)]
pub struct Racket;

#[derive(Component, Clone, Copy)]
pub struct Ball;

#[derive(Component, Clone, Copy)]
pub struct Table;

#[derive(Component, Clone, Copy)]
enum ModelComponent {
    Tbl,
    Rkt,
    Bal,
}

#[derive(Resource, Clone)]
pub struct RacketCommandQueue(pub Arc<Mutex<Vec<RacketTransformCommand>>>);

#[derive(Debug, Clone, Copy)]
pub enum CommandDataType {
    Position(Vec3),
    Rotation(Quat),
}

#[derive(Clone, Debug)]
pub struct RacketTransformCommand {
    pub command: CommandDataType,
}

#[derive(Component)]
struct LeftCamera;

#[derive(Component)]
struct RightCamera;

enum CameraComponent {
    LC,
    RC,
}

#[derive(Component)]
struct MoveSpeedText;

#[derive(Resource)]
pub struct LaunchState {
    pub launched: bool,
}

impl Default for LaunchState {
    fn default() -> Self {
        LaunchState { launched: false }
    }
}

#[derive(Resource, Default)]
struct BallTableCollisionCount {
    pub count: u32,
}

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let command_queue: Arc<Mutex<Vec<RacketTransformCommand>>> = Arc::new(Mutex::new(Vec::new()));
    App::new()
        .insert_resource(WsRuntime(rt))
        .insert_resource(RacketCommandQueue(command_queue))
        .insert_resource(LaunchState::default())
        .insert_resource(BallTableCollisionCount::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_event::<CollisionEvent>()
        .add_systems(
            Startup,
            (
                setup,
                ws_handler::start_websocket_server,
                controller_server::start_controller_server,
            ),
        )
        .add_systems(
            Update,
            (
                command_handler::apply_racket_commands,
                collision_event_system.in_set(PhysicsSet::SyncBackend),
                control_ball_system,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, windows: Query<&Window>) {
    let window = windows.single();
    let width = window.width();
    let height = window.height();

    let model_names = vec!["tennis_table.glb", "pong-racket.glb", "ball.glb"];
    let pos = vec![
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(1.0, 1.0, 0.0),
        Vec3::new(0.95, 1.05, 0.0),
    ];
    let rotation = vec![
        Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
        Quat::from_euler(EulerRot::XYZ, 0.0, -PI / 2.0, 0.0),
        Quat::from_euler(EulerRot::XYZ, 0.0, 0.0, 0.0),
    ];
    let components = vec![
        Some(ModelComponent::Tbl),
        Some(ModelComponent::Rkt),
        Some(ModelComponent::Bal),
    ];

    for (i, model_name) in model_names.iter().enumerate() {
        let model_path = format!("models/{}", model_name);
        let gltf_handle = asset_server.load(GltfAssetLabel::Scene(0).from_asset(model_path));

        let scale_num = match *model_name {
            "ball.glb" => 2.0,
            _ => 1.0,
        };

        let mut entity = commands.spawn((
            SceneRoot(gltf_handle),
            Transform::from_xyz(pos[i].x, pos[i].y, pos[i].z)
                .with_rotation(rotation[i])
                .with_scale(Vec3::splat(scale_num)),
            /* Transform {
                translation: pos[i],
                rotation: rotation[i],
                scale: Vec3::splat(scale_num),
            }, */
        ));

        // let com = components[i].clone();
        match components[i].as_ref() {
            Some(ModelComponent::Tbl) => {
                entity.insert((
                    Table,
                    RigidBody::Fixed,
                    ActiveEvents::COLLISION_EVENTS,
                    Collider::cuboid(1.0, 0.75, 1.0),
                    Ccd { enabled: true },
                    Restitution {
                        coefficient: 0.9, // 让桌子也有高弹性
                        combine_rule: CoefficientCombineRule::Max,
                    },
                ));
            }
            Some(ModelComponent::Rkt) => {
                entity.insert((
                    Racket,
                    RigidBody::KinematicPositionBased,
                    ActiveEvents::COLLISION_EVENTS,
                    Collider::cuboid(0.05, 0.01, 0.1),
                    Ccd { enabled: true },
                    Restitution {
                        coefficient: 0.3, 
                        combine_rule: CoefficientCombineRule::Max,
                    },
                ));
            }
            Some(ModelComponent::Bal) => {
                entity.insert((
                    Ball,
                    RigidBody::Dynamic,
                    Velocity::zero(),
                    GravityScale(0.0),
                    ActiveEvents::COLLISION_EVENTS,
                    Collider::ball(0.01),
                    Ccd { enabled: true },
                    Restitution {
                        coefficient: 0.4, // 从 0.8 降到 0.4
                        combine_rule: CoefficientCombineRule::Average,
                    },
                    Friction {
                        coefficient: 0.6,
                        combine_rule: CoefficientCombineRule::Average,
                    },
                    Damping {
                        linear_damping: 0.2, // 默认 0.0，设为 0.1–0.3 让速度自然衰减
                        angular_damping: 0.1,
                    },
                ));
            }
            _ => {}
        };
    }

    commands.spawn((
        Transform::from_xyz(0.0, 0.75, 0.0),
        RigidBody::Fixed,
        ActiveEvents::COLLISION_EVENTS,
        Collider::cuboid(0.1, 0.1, 1.0),
        Ccd { enabled: true },
    ));

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 3.0, 0.0),
    ));
    // let positions = [UVec2::new(0, 0), UVec2::new(width as u32 / 2, 0)];
    let size = UVec2::new(width as u32 / 2, height as u32);
    let components = [CameraComponent::LC, CameraComponent::RC];
    for (i, component) in components.iter().enumerate() {
        let mut cmd = commands.spawn((
            Camera3d { ..default() },
            Camera {
                viewport: Some(camera::Viewport {
                    physical_position: UVec2::new(i as u32 * width as u32 / 2, 0),
                    physical_size: size,
                    ..default()
                }),
                order: i as isize,
                ..default()
            },
            Transform::from_xyz(-2.5 * (i as f32 * 2.0 - 1.0), 1.50, 0.0)
                .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        ));
        match component {
            CameraComponent::LC => cmd.insert(LeftCamera),
            CameraComponent::RC => cmd.insert(RightCamera),
        };
    }

    commands.spawn((
        Text::new("move speed:"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
    commands.spawn((
        Text::new("0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(30.0),
            left: Val::Px(10.0),
            ..default()
        },
        MoveSpeedText,
    ));
}

fn control_ball_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &mut Transform,
            Option<&RigidBody>,
            Option<&GravityScale>,
        ),
        (With<Ball>, Without<Racket>),
    >,
    mut launch_state: ResMut<LaunchState>,
    mut counter: ResMut<BallTableCollisionCount>,
    racket_query: Query<&Transform, With<Racket>>,
) {
    let racket_transform = match racket_query.get_single() {
        Ok(t) => t,
        Err(_) => return, // 没有找到 Racket，跳过
    };
    for (entity, mut transform, rb, gs) in query.iter_mut() {
        if launch_state.launched {
            // 发射，设置为 Dynamic，由物理引擎接管
            if gs.unwrap().0 == 0.0 {
                commands.entity(entity).insert(GravityScale(1.0));
            }
        } else {
            // transform.translation.z = racket_transform.translation.z + transform.rotation.y * 0.05;
            // 未发射，保持固定高度，但允许参与碰撞
            transform.translation.y = 1.04;
            // 如果不是 Kinematic，设置为 Kinematic
            if !matches!(rb, Some(RigidBody::KinematicPositionBased)) {
                commands
                    .entity(entity)
                    .insert(GravityScale(0.0))
                    .remove::<Velocity>(); // 清除可能残留的速度
            }
        }
        if transform.translation.x > 2.0
            || transform.translation.x < -2.0
            || transform.translation.y > 2.0
            || transform.translation.y < 0.0
            || transform.translation.z > 2.0
            || transform.translation.z < -2.0 || counter.count > 2
        {
            // 球超出边界，重置位置
            transform.translation = Vec3::new(0.95, 1.05, 0.0);
            commands
                .entity(entity)
                .insert(GravityScale(0.0))
                .insert(Velocity::zero());
            launch_state.launched = false;
            counter.count = 0;
        }
    }
}

fn collision_event_system(
    mut collision_events: EventReader<CollisionEvent>,
    colliders: Query<(Entity, &Collider)>,
    racket_q: Query<&Racket>,
    ball_q: Query<&Ball>,
    table_q: Query<(), With<Table>>,
    mut launch_state: ResMut<LaunchState>,
    mut counter: ResMut<BallTableCollisionCount>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            let hit = (racket_q.get(*e1).is_ok() && ball_q.get(*e2).is_ok())
                || (racket_q.get(*e2).is_ok() && ball_q.get(*e1).is_ok());
            if hit {
                launch_state.launched = true;
                counter.count = 0;
            }

            let is_ball_table = (ball_q.get(*e1).is_ok() && table_q.get(*e2).is_ok())
                || (ball_q.get(*e2).is_ok() && table_q.get(*e1).is_ok());
            if is_ball_table {
                counter.count += 1;
                info!("Ball-Table 碰撞次数：{}", counter.count);
            }
        }
    }
}

fn add_convex_colliders(
    mut commands: Commands,
    meshes: Res<Assets<Mesh>>,
    query: Query<
        (
            Entity,
            &Mesh3d,
            &GlobalTransform,
            Option<&Ball>,
            Option<&Table>,
        ),
        Without<Collider>,
    >,
) {
    for (entity, mesh3d, _gtrans, maybe_ball, maybe_table) in query.iter() {
        // 从 Mesh3d 解出真正的 Handle<Mesh>
        if let Some(mesh) = meshes.get(&mesh3d.0) {
            if let Some(VertexAttributeValues::Float32x3(positions)) =
                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            {
                let points: Vec<Vec3> = positions
                    .iter()
                    .map(|p| Vec3::new(p[0], p[1], p[2]))
                    .collect();
                if let Some(collider) = Collider::convex_hull(&points) {
                    let mut entity_cmd = commands.entity(entity);

                    // 先插入 Collider
                    entity_cmd.insert(collider);

                    // 根据条件插入不同的刚体类型
                    if maybe_table.is_some() {
                        entity_cmd.insert(RigidBody::Fixed);
                    } else if maybe_ball.is_some() {
                        entity_cmd
                            .insert(RigidBody::Dynamic)
                            .insert(Ccd { enabled: true })
                            .insert(Restitution::coefficient(0.8))
                            .insert(Friction::coefficient(0.2));
                    }
                }
            }
        }
    }
}
