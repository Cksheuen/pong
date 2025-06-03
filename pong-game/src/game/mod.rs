use std::f32::consts::PI;

use bevy::{prelude::*, render::camera};
use bevy_rapier3d::plugin::RapierPhysicsPlugin;
use bevy_rapier3d::plugin::{RapierConfiguration, TimestepMode};
use bevy_rapier3d::prelude::*;

use crate::GameState;

use crate::components::button::button_system;

pub mod practice;
pub mod utils;

use utils::{command_handler, controller_server, init_resources, ws_handler};

use utils::{
    Ball, BallTableCollisionCount, CameraComponent, CommandDataType, LaunchState, LeftCamera,
    ModelComponent, MoveSpeedText, Racket, RacketCommandQueue, RacketTransformCommand, RightCamera,
    Table,
};

use super::despawn_screen;

#[derive(Component)]
pub struct OnNormalGameScreen;

pub fn game_plugin(app: &mut App) {
    app.add_plugins(init_resources)
        .add_systems(OnEnter(GameState::GameEntering), game_init)
        .add_plugins((
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .add_systems(
            OnEnter(GameState::GameIniting),
            (
                setup,
                ws_handler::start_websocket_server,
                controller_server::start_controller_server,
                setup_physics_config,
                over_init,
            ),
        )
        .add_systems(
            Update,
            (
                command_handler::apply_racket_commands,
                collision_event_system.in_set(PhysicsSet::SyncBackend),
                contact_force_system.in_set(PhysicsSet::SyncBackend),
                bounce_system.in_set(PhysicsSet::SyncBackend),
                control_ball_system,
            )
                .run_if(in_state(GameState::GameRunning)),
        )
        .add_systems(
            OnEnter(GameState::Menu),
            despawn_screen::<OnNormalGameScreen>,
        )
        .add_systems(
            Update,
            (button_system, menu_action).run_if(not(in_state(GameState::Menu))),
        );
}

#[derive(Component)]
enum ButtonAction {
    Esc,
}

pub fn menu_action(
    interaction_query: Query<(&Interaction, &ButtonAction), (Changed<Interaction>, With<Button>)>,
    mut app_exit_events: EventWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                ButtonAction::Esc => {
                    game_state.set(GameState::Menu);
                }
            }
        }
    }
}

fn setup_physics_config(mut commands: Commands, mut timestep_mode: ResMut<TimestepMode>) {
    commands.spawn(RapierConfiguration {
        gravity: Vec3::new(0.0, -9.81, 0.0),
        physics_pipeline_active: true,
        query_pipeline_active: true,
        scaled_shape_subdivision: 1,
        force_update_from_transform_changes: true,
    });
    *timestep_mode = TimestepMode::Variable {
        max_dt: 1. / 120.,
        time_scale: 0.5,
        substeps: 1,
    }
}

fn game_init(mut commands: Commands, mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::GameIniting);
}

fn over_init(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::GameRunning);
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
            OnNormalGameScreen,
        ));

        // let com = components[i].clone();
        match components[i].as_ref() {
            Some(ModelComponent::Tbl) => {
                entity.insert((
                    Table,
                    RigidBody::Fixed,
                    ActiveEvents::COLLISION_EVENTS,
                    Collider::cuboid(1.2, 0.75, 1.0),
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
                    Collider::cuboid(0.07, 0.01, 0.12),
                    Ccd { enabled: true },
                    Restitution {
                        coefficient: 0.,
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
                    ActiveEvents::COLLISION_EVENTS | ActiveEvents::CONTACT_FORCE_EVENTS,
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
                        linear_damping: 0.3, // 默认 0.0，设为 0.1–0.3 让速度自然衰减
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
        OnNormalGameScreen,
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
            OnNormalGameScreen,
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
        OnNormalGameScreen,
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
        OnNormalGameScreen,
    ));
    let button_node = Node {
        // width: Val::Px(100.0),
        // height: Val::Px(100.0),
        position_type: PositionType::Absolute,
        top: Val::Px(10.0),
        right: Val::Px(10.0),
        margin: UiRect::all(Val::Px(10.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,

        ..default()
    };
    let button_text = TextFont {
        font_size: 32.0,
        ..default()
    };
    commands.spawn((
        Text::new("Esc"),
        button_text.clone(),
        Button,
        button_node.clone(),
        ButtonAction::Esc,
        OnNormalGameScreen
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
        } /* else {
        // transform.translation.z = racket_transform.translation.z + transform.rotation.y * 0.05;
        // 未发射，保持固定高度，但允许参与碰撞
        // transform.translation.y = 1.04;
        // 如果不是 Kinematic，设置为 Kinematic
        if !matches!(rb, Some(RigidBody::KinematicPositionBased)) {
        commands
        .entity(entity)
        .insert(GravityScale(0.0))
        .insert(Velocity::zero());
        }
        } */
        if transform.translation.x > 2.0
            || transform.translation.y > 2.0
            || transform.translation.y < 0.0
            || transform.translation.z > 2.0
            || transform.translation.z < -2.0
            || counter.count > 2
        {
            // 球超出边界，重置位置
            transform.translation = Vec3::new(0.9, 1.0, 0.0);
            commands
                .entity(entity)
                .insert(GravityScale(0.0))
                .insert(Velocity::zero());
            launch_state.launched = false;
            counter.count = 0;
        }
    }
}

fn bounce_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Ball>>,
    mut velocities: Query<&mut Velocity, With<Ball>>,
) {
    for (entity, tf) in query.iter() {
        if tf.translation.x < -1.5 {
            // 直接改 Velocity 组件
            if let Ok(mut vel) = velocities.get_mut(entity) {
                vel.linvel.x = vel.linvel.x.abs() * 0.8;
            }
        }
    }
}

fn collision_event_system(
    mut collision_events: EventReader<CollisionEvent>,
    colliders: Query<(Entity, &Collider)>,
    racket_q: Query<&Racket>,
    ball_q: Query<&Ball>,
    mut ball_vel_q: Query<&mut Velocity>,
    table_q: Query<(), With<Table>>,
    mut launch_state: ResMut<LaunchState>,
    mut counter: ResMut<BallTableCollisionCount>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            /* let hit = (racket_q.get(*e1).is_ok() && ball_q.get(*e2).is_ok())
                           || (racket_q.get(*e2).is_ok() && ball_q.get(*e1).is_ok());
                       if hit {
                           launch_state.launched = true;
                           counter.count = 0;
                       }
            */
            let e1_is_ball = ball_q.get(*e1).is_ok();
            let e2_is_ball = ball_q.get(*e2).is_ok();
            let e1_is_racket = racket_q.get(*e1).is_ok();
            let e2_is_racket = racket_q.get(*e2).is_ok();

            let hit_racket = (e1_is_ball && e2_is_racket) || (e2_is_ball && e1_is_racket);
            if hit_racket {
                launch_state.launched = true;
                counter.count = 0;
                println!("Ball <-> Racket 碰撞触发！");
            }

            // 球-桌子碰撞计数
            let hit_table = (e1_is_ball && table_q.get(*e2).is_ok())
                || (e2_is_ball && table_q.get(*e1).is_ok());
            if hit_table {
                counter.count += 1;
                println!("Ball <-> Table 碰撞，累计：{}", counter.count);
            }
        }
    }
}

fn contact_force_system(
    mut force_events: EventReader<ContactForceEvent>,
    ball_q: Query<&Ball>,
    racket_q: Query<&Racket>,
    mut ball_vel_q: Query<&mut Velocity, With<Ball>>,
) {
    for event in force_events.read() {
        let e1 = event.collider1;
        let e2 = event.collider2;

        let e1_is_ball = ball_q.get(e1).is_ok();
        let e2_is_ball = ball_q.get(e2).is_ok();
        let e1_is_racket = racket_q.get(e1).is_ok();
        let e2_is_racket = racket_q.get(e2).is_ok();

        let (ball_entity, hit) = match (e1_is_ball, e2_is_racket, e2_is_ball, e1_is_racket) {
            (true, true, _, _) => (e1, true),
            (_, _, true, true) => (e2, true),
            _ => (e1, false),
        };

        if hit {
            // 归一化方向，避免 NaN
            let force_dir = event.total_force.normalize_or_zero();
            let speed = event.total_force_magnitude * 0.2; // 自定义系数

            if let Ok(mut vel) = ball_vel_q.get_mut(ball_entity) {
                vel.linvel = force_dir * 3.0;
                println!(
                    "设置球 {:?} 速度为：方向 {:?}, 强度 {:.2}, 最终速度 {:?}",
                    ball_entity, force_dir, event.total_force_magnitude, vel.linvel
                );
            }
        }
    }
}
