use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

use bevy::log::tracing_subscriber::fmt::time;
use rand::Rng;

use bevy::{prelude::*, render::camera};
use bevy_rapier3d::plugin::RapierPhysicsPlugin;
use bevy_rapier3d::plugin::{RapierConfiguration, TimestepMode};
use bevy_rapier3d::prelude::*;

use crate::components::button::button_system;
use crate::{
    GameState,
    game::{
        command_handler, controller_server,
        utils::{
            Ball, BallTableCollisionCount, CameraComponent, CommandDataType, LaunchState,
            LeftCamera, ModelComponent, MoveSpeedText, Racket, RacketCommandQueue,
            RacketTransformCommand, RightCamera, Table, TrajectoryPreview,
        },
        ws_handler,
    },
};

use super::despawn_screen;

#[derive(Component)]
pub struct OnPracticeGameScreen;

pub fn game_practice_plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::GamePracticeEntering), game_init)
        .add_systems(
            OnEnter(GameState::GamePracticeIniting),
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
                // move_racket,
            )
                .run_if(in_state(GameState::GamePracticeRunning)),
        )
        .add_systems(
            OnEnter(GameState::Menu),
            despawn_screen::<OnPracticeGameScreen>,
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

fn menu_action(
    interaction_query: Query<(&Interaction, &ButtonAction), (Changed<Interaction>, With<Button>)>,
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
    game_state.set(GameState::GamePracticeIniting);
}

fn over_init(mut game_state: ResMut<NextState<GameState>>) {
    game_state.set(GameState::GamePracticeRunning);
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
            OnPracticeGameScreen,
        ));

        // let com = components[i].clone();
        match components[i].as_ref() {
            Some(ModelComponent::Tbl) => {
                entity.insert((
                    Table,
                    RigidBody::Fixed,
                    ActiveEvents::COLLISION_EVENTS,
                    Collider::cuboid(1.3, 0.74, 0.8),
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
                        coefficient: 1.0, // 从 0.8 降到 0.4
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

    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(0.0, 3.0, 0.0),
        OnPracticeGameScreen,
    ));
    // let positions = [UVec2::new(0, 0), UVec2::new(width as u32 / 2, 0)];
    let size = UVec2::new(width as u32, height as u32);
    commands.spawn((
        Camera3d { ..default() },
        Camera {
            viewport: Some(camera::Viewport {
                physical_position: UVec2::new(0, 0),
                physical_size: size,
                ..default()
            }),
            // order: i as isize,
            order: 3,
            ..default()
        },
        Transform::from_xyz(2.5, 1.50, 0.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
        OnPracticeGameScreen,
        LeftCamera,
    ));

    commands.spawn((
        Transform::from_xyz(0., 0.74, 0.),
        OnPracticeGameScreen,
        RigidBody::Fixed,
        ActiveEvents::COLLISION_EVENTS,
        Collider::cuboid(0.02, 0.1, 0.8),
        Ccd { enabled: true },
        Restitution {
            coefficient: 0.9, // 让桌子也有高弹性
            combine_rule: CoefficientCombineRule::Max,
        },
    ));

    commands.spawn((
        Text::new("move speed:"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        OnPracticeGameScreen,
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
        OnPracticeGameScreen,
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
        OnPracticeGameScreen,
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
    mut preview: ResMut<TrajectoryPreview>,
    mut launch_state: ResMut<LaunchState>,
    mut counter: ResMut<BallTableCollisionCount>,
    mut gizmos: Gizmos,
    time: Res<Time>,
) {
    if preview.pending_reset {
        preview.timer.tick(time.delta());

        // 1 秒后正式重置球位置
        if preview.timer.finished() {
            if let Some(entity) = preview.entity {
                commands
                    .entity(entity)
                    .insert(
                        Transform::from_translation(preview.cached_translation)
                            .with_rotation(Quat::IDENTITY),
                    )
                    .insert(Velocity {
                        linvel: preview.cached_velocity,
                        angvel: Vec3::ZERO,
                    });

                launch_state.launched = false;
                counter.count = 0;
                preview.pending_reset = false;
                preview.entity = None;
            }
        } else {
            // 每帧继续画轨迹（可选）
            let mut current_pos = preview.cached_translation;
            let mut current_vel = preview.cached_velocity;
            let gravity = Vec3::new(0.0, -9.81, 0.0);
            let dt = 0.05;

            for _ in 0..40 {
                let next_vel = current_vel + gravity * dt;
                let next_pos = current_pos + current_vel * dt + 0.5 * gravity * dt * dt;
                gizmos.line(current_pos, next_pos, Color::srgb(0., 0., 1.));
                current_pos = next_pos;
                current_vel = next_vel;
            }

            return; // 等待中，不再执行其他逻辑
        }
    }

    for (entity, mut transform, rb, gs) in query.iter_mut() {
        if launch_state.launched {
            // 发射，设置为 Dynamic，由物理引擎接管
            if gs.unwrap().0 == 0.0 {
                commands.entity(entity).insert(GravityScale(1.0));
            }
            if transform.translation.x > 1.0
                || transform.translation.x < -1.0
                || transform.translation.y > 2.0
                || transform.translation.y < 0.5
                || transform.translation.z > 0.7
                || transform.translation.z < -0.7
                || counter.count > 2
            {
                let mut rng = rand::rng();

                let translation = Vec3::new(
                    rng.random_range(-0.3..=-0.1),
                    rng.random_range(1.0..=1.2),
                    rng.random_range(-0.4..=0.4),
                );
                let linvel = Vec3::new(
                    rng.random_range(2.0..=4.0),
                    0.0,
                    rng.random_range(-1.0..=1.0),
                );

                preview.pending_reset = true;
                preview.timer = Timer::from_seconds(2.0, TimerMode::Once);
                preview.cached_translation = translation;
                preview.cached_velocity = linvel;
                preview.entity = Some(entity);

                // launch_state.launched = false;
                // counter.count = 0;
            }
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
            let e1_is_ball = ball_q.get(*e1).is_ok();
            let e2_is_ball = ball_q.get(*e2).is_ok();
            let e1_is_racket = racket_q.get(*e1).is_ok();
            let e2_is_racket = racket_q.get(*e2).is_ok();

            let hit_racket = (e1_is_ball && e2_is_racket) || (e2_is_ball && e1_is_racket);
            if hit_racket && !launch_state.launched {
                launch_state.launched = true;
                counter.count = 0;
                // println!("Ball <-> Racket 碰撞触发！");
            }

            // 球-桌子碰撞计数
            let hit_table = (e1_is_ball && table_q.get(*e2).is_ok())
                || (e2_is_ball && table_q.get(*e1).is_ok());
            if hit_table {
                counter.count += 1;
                // println!("Ball <-> Table 碰撞，累计：{}", counter.count);
            }
        }
    }
}

fn contact_force_system(
    mut force_events: EventReader<ContactForceEvent>,
    ball_q: Query<&Ball>,
    racket_q: Query<&Racket>,
    table_q: Query<&Transform, (With<Table>, Without<Ball>)>,
    launch_state: ResMut<LaunchState>,
    mut params: ParamSet<(
        Query<&mut Velocity, With<Ball>>,
        Query<&mut Transform, With<Ball>>,
    )>,
    mut text: Single<&mut Text, With<MoveSpeedText>>,
) {
    for event in force_events.read() {
        let e1 = event.collider1;
        let e2 = event.collider2;

        let e1_is_ball = ball_q.get(e1).is_ok();
        let e2_is_ball = ball_q.get(e2).is_ok();
        let e1_is_racket = racket_q.get(e1).is_ok();
        let e2_is_racket = racket_q.get(e2).is_ok();
        let e1_is_table = table_q.get(e1).is_ok();
        let e2_is_table = table_q.get(e2).is_ok();

        let (ball_entity, hit) = match (e1_is_ball, e2_is_racket, e2_is_ball, e1_is_racket) {
            (true, true, _, _) => (e1, true),
            (_, _, true, true) => (e2, true),
            _ => (e1, false),
        };

        if hit {
            // 归一化方向，避免 NaN
            let force_dir = event.total_force.normalize_or_zero();

            if let Ok(mut vel) = params.p0().get_mut(ball_entity) {
                vel.linvel = force_dir * 3.0;
                /* println!(
                    "设置球 {:?} 速度为：方向 {:?}, 强度 {:.2}, 最终速度 {:?}",
                    ball_entity, force_dir, event.total_force_magnitude, vel.linvel
                ); */
            }
        }

        let hit_table = (e1_is_ball && e2_is_table) || (e2_is_ball && e1_is_table);
        if hit_table {
            if let Ok(transform) = params.p1().get(ball_entity) {
                println!("📍 球与桌子接触，桌子位置: {:?}", transform.translation);
            }
        }
        text.0 = format!("{:?} {:?}", hit_table, launch_state.launched);
    }
}

fn move_racket(
    mut query: Query<&mut Transform, (With<Racket>, Without<Ball>)>,
    mut ball_query: Query<&mut Transform, With<Ball>>,
    mut text: Single<&mut Text, With<MoveSpeedText>>,
    launch_state: ResMut<LaunchState>,
) {
    let mut ball_transform = match ball_query.get_single_mut() {
        Ok(t) => t,
        Err(_) => return, // 没有找到 Racket，跳过
    };
    let mut racket_transform = match query.get_single_mut() {
        Ok(t) => t,
        Err(_) => return,
    };

    // let base = Quat::from_euler(EulerRot::XYZ, 0.0, -PI / 2.0 + rotation.x, rotation.w);
    // transform.rotation = base;

    let rotation = racket_transform.rotation;
    let x = rotation.y + PI / 2.0;
    let w = rotation.z;

    racket_transform.translation = Vec3::new(
        -x.abs().cos() / 4. + 0.1, //rotation.x.abs().sin() / 2. - 0.05,
        -0.03,
        0., // rotation.x.abs().cos(),
    );
    if x < 0. {
        racket_transform.translation.z += 0.05;
    } else {
        racket_transform.translation.z -= 0.05;
    }
    if !launch_state.launched {
        // ball_transform.translation.z = transform.translation.z + rotation.x.sin() * 0.05;
        racket_transform.translation.z += ball_transform.translation.z;
        if ball_transform.translation.x > 0. {
            racket_transform.translation.x += ball_transform.translation.x;
            racket_transform.translation.y += ball_transform.translation.y;
        }
        // transform.translation += Vec3::new(0.9, 1.0, rotation.x / 4.);
    } else {
        racket_transform.translation += Vec3::new(0.9, 1.0, 0.);
    }
    text.0 = format!("{:?}", x);
}
