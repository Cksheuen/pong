pub mod command_handler;
pub mod controller_server;
pub mod ws_handler;

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Resource)]
pub struct WsRuntime(tokio::runtime::Runtime);

#[derive(Component, Clone, Copy)]
pub struct Racket;

#[derive(Component, Clone, Copy)]
pub struct Ball;

#[derive(Component, Clone, Copy)]
pub struct Table;

#[derive(Component, Clone, Copy)]
pub enum ModelComponent {
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
pub struct LeftCamera;

#[derive(Component)]
pub struct RightCamera;

pub enum CameraComponent {
    LC,
    RC,
}

#[derive(Component)]
pub struct MoveSpeedText;

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
pub struct BallTableCollisionCount {
    pub count: u32,
}

#[derive(Resource)]
pub struct TrajectoryPreview {
    pub timer: Timer,
    pub pending_reset: bool,
    pub cached_translation: Vec3,
    pub cached_velocity: Vec3,
    pub entity: Option<Entity>,
}

pub fn init_resources(app: &mut App) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let command_queue: Arc<Mutex<Vec<RacketTransformCommand>>> = Arc::new(Mutex::new(Vec::new()));

    app.insert_resource(WsRuntime(rt))
        .insert_resource(RacketCommandQueue(command_queue))
        .insert_resource(LaunchState::default())
        .insert_resource(BallTableCollisionCount::default())
        .insert_resource(TrajectoryPreview {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
            pending_reset: false,
            cached_translation: Vec3::ZERO,
            cached_velocity: Vec3::ZERO,
            entity: None,
        })
        .add_event::<CollisionEvent>()
        .add_event::<ContactForceEvent>();
}
