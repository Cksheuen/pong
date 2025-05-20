use bevy::{math, prelude::*};
use std::f32::consts::PI;

use crate::{Ball, CommandDataType, LaunchState, MoveSpeedText, Racket, RacketCommandQueue};

pub fn apply_racket_commands(
    mut query: Query<&mut Transform, (With<Racket>, Without<Ball>)>,
    mut ball_query: Query<&mut Transform, With<Ball>>,
    mut text: Single<&mut Text, With<MoveSpeedText>>,
    command_queue: Res<RacketCommandQueue>,
    launch_state: ResMut<LaunchState>,
) {
    let mut ball_transform = match ball_query.get_single_mut() {
        Ok(t) => t,
        Err(_) => return, // 没有找到 Racket，跳过
    };
    let mut queue: std::sync::MutexGuard<'_, Vec<crate::RacketTransformCommand>> =
        command_queue.0.lock().unwrap();
    for command in queue.drain(..) {
        let handler: fn(CommandDataType, &mut Transform,&mut Transform, 
            &mut Single<&mut Text, With<MoveSpeedText>>, &ResMut<LaunchState>) =//, &mut Single<&mut Text>
            match command.command {
                CommandDataType::Position(_) => handle_position_command,
                CommandDataType::Rotation(_) => handle_rotation_command,
            };
        for mut transform in query.iter_mut() {
            handler(
                command.command,
                &mut transform,
                &mut ball_transform,
                &mut text,
                &launch_state,
            ); //, &mut text
        }
    }
}

pub fn handle_rotation_command(
    data: CommandDataType,
    transform: &mut Transform,
    ball_transform: &mut Transform,
    text: &mut Single<&mut Text, With<MoveSpeedText>>, // text: &mut Single<(&mut Text)>,
    launch_state: &ResMut<LaunchState>,
) {
    let rotation = match data {
        CommandDataType::Rotation(rotation) => rotation,
        _ => return,
    };
    let base = Quat::from_euler(EulerRot::XYZ, 0.0, -PI / 2.0 + rotation.x, rotation.w);
    transform.rotation = base;

    transform.translation = Vec3::new(
        0.9 + (rotation.x / 4.).abs().sin() / 6.,
        1.0,
        rotation.x / 4.,
    );
    if !launch_state.launched {
        ball_transform.translation.z = transform.translation.z + rotation.x.sin() * 0.05;
    }
    
    text.0 = format!("{:.3}", rotation.x);
}

pub fn handle_position_command(
    data: CommandDataType,
    transform: &mut Transform,
    ball_transform: &mut Transform,
    text: &mut Single<&mut Text, With<MoveSpeedText>>,
    launch_state: &ResMut<LaunchState>,
    // text: &mut Single<(&mut Text)>,
) {
    /* let position = match data {
        CommandDataType::Position(position) => position,
        _ => return,
    };
    // let mut base = Vec3::new(1.0, 1.0, 0.0);
    let mut base = transform.translation;
    base += position.zyx();
    // text.0 = format!("{:.2}", base.x);
    text.0 = format!("{:.3}", position.x);
    transform.translation = base; */
}
