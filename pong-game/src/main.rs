use std::f32::consts::PI;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

use bevy::{
    ecs::observer::Trigger,
    prelude::*,
    render::{
        camera,
        mesh::{Mesh, Mesh3d, VertexAttributeValues},
    },
    scene::{SceneInstance, SceneInstanceReady, SceneRoot},
};
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

fn main() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let command_queue: Arc<Mutex<Vec<RacketTransformCommand>>> = Arc::new(Mutex::new(Vec::new()));
    App::new()
        .insert_resource(WsRuntime(rt))
        .insert_resource(RacketCommandQueue(command_queue))
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(
            Startup,
            (
                setup,
                ws_handler::start_websocket_server,
                controller_server::start_controller_server,
            ),
        )
        .add_systems(Update, command_handler::apply_racket_commands)
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
        Vec3::new(0.75, 1.0, 0.0),
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
            Transform {
                translation: pos[i],
                rotation: rotation[i],
                scale: Vec3::splat(scale_num),
            },
        ));

        // let com = components[i].clone();
        match components[i].as_ref() {
            Some(ModelComponent::Tbl) => {
                entity.insert((
                    Table,
                    RigidBody::Fixed,
                    Collider::cuboid(1.0, 0.75, 1.0),
                    Ccd { enabled: true },
                ));
            }
            Some(ModelComponent::Rkt) => {
                entity.insert((
                    Racket,
                    RigidBody::KinematicPositionBased,
                    Collider::cuboid(0.05, 0.01, 0.1),
                    Ccd { enabled: true },
                ));
            }
            Some(ModelComponent::Bal) => {
                entity.insert((
                    Ball,
                    RigidBody::Dynamic,
                    Collider::ball(0.01),
                    Ccd { enabled: true },
                    Restitution::coefficient(0.8),
                    Friction::coefficient(0.2),
                ));
            }
            _ => {}
        };

        /* entity.observe(
            move |trigger: Trigger<SceneInstanceReady>,
                      mut cmds: Commands,
                      meshes: Res<Assets<Mesh>>,
                      query: Query<(Entity, &Mesh3d), Without<Collider>>,
                      instances: Query<&SceneInstance>|
                {
                    // 1. 直接读取字段，不是方法
                    // let instance_id: SceneInstance = trigger.event().instance_id;
                    let instance_id = trigger.event().instance_id;

                    // 2. 直接用 query.iter()，不要传 &cmds.world :contentReference[oaicite:0]{index=0}
                    for (entity, mesh3d) in query.iter() {
                        // 3. 比较时，将 si（&SceneInstance）解引用后与 instance_id 比较
                        if let Ok(si) = instances.get(entity) {
                            if si.deref() != &instance_id {
                                continue;
                            }
                        } else {
                            continue;
                        }

                        // 4. 拿到 Mesh，并提取顶点生成凸包
                        if let Some(mesh) = meshes.get(&mesh3d.0) {
                            if let Some(VertexAttributeValues::Float32x3(pos)) =
                                mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                            {
                                let points: Vec<Vec3> = pos.iter().map(|&p| Vec3::from(p)).collect();
                                if let Some(collider) = Collider::convex_hull(&points) {
                                    let mut ec = cmds.entity(entity);
                                    ec.insert(collider);

                                    // 5. 根据捕获的 com 插入对应刚体
                                    match com.clone() {
                                        Some(ModelComponent::Tbl) => {
                                            ec.insert(RigidBody::Fixed);
                                            println!("table");
                                        }
                                        Some(ModelComponent::Bal) => {
                                            ec.insert((
                                                RigidBody::Dynamic,
                                                Ccd { enabled: true },
                                                Restitution::coefficient(0.8),
                                                Friction::coefficient(0.2),
                                            ));
                                            println!("ball");
                                        }
                                        Some(ModelComponent::Rkt) => {
                                            ec.insert(RigidBody::KinematicPositionBased);
                                            println!("racket");
                                        }
                                        None => {}
                                    }
                                }
                                println!("instance_id: {:?}", instance_id);
                                println!("entity: {:?}", entity);
                                println!("mesh3d: {:?}", mesh3d);
                            }
                        }
                    }
                },
        ); */
    }

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
