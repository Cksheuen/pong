use bevy::prelude::*;
use std::f32::consts::PI;

use crate::{CommandDataType, MoveSpeedText, Racket, RacketCommandQueue};

pub fn apply_racket_commands(
    mut query: Query<&mut Transform, With<Racket>>,
    mut text: Single<&mut Text, With<MoveSpeedText>>,
    command_queue: Res<RacketCommandQueue>,
) {
    let mut queue = command_queue.0.lock().unwrap();
    for command in queue.drain(..) {
        let handler: fn(CommandDataType, &mut Transform, &mut Single<&mut Text, With<MoveSpeedText>>) =//, &mut Single<&mut Text>
            match command.command {
                CommandDataType::Position(_) => handle_position_command,
                CommandDataType::Rotation(_) => handle_rotation_command,
            };
        for mut transform in query.iter_mut() {
            handler(command.command, &mut transform, &mut text); //, &mut text
        }
    }
}

pub fn handle_rotation_command(
    data: CommandDataType,
    transform: &mut Transform,
    text: &mut Single<&mut Text, With<MoveSpeedText>>, // text: &mut Single<(&mut Text)>,
) {
    let rotation = match data {
        CommandDataType::Rotation(rotation) => rotation,
        _ => return,
    };
    let base = Quat::from_euler(EulerRot::XYZ, 0.0, -PI / 2.0 + rotation.x, rotation.w);
    transform.rotation = base;

    transform.translation = Vec3::new(1.0, 1.0, rotation.x / 4.);
    text.0 = format!("{:.3}", rotation.x);
}

pub fn handle_position_command(
    data: CommandDataType,
    transform: &mut Transform,
    text: &mut Single<&mut Text, With<MoveSpeedText>>,
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
