use bevy::prelude::*;

pub mod game;
pub mod menu;
pub mod components;

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn();
    }
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Menu,
    GameEntering,
    GameIniting,
    GameRunning,
    GamePracticeEntering,
    GamePracticeIniting,
    GamePracticeRunning,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_plugins((
            menu::menu_plugin,
            game::game_plugin,
            game::practice::game_practice_plugin,
        ))
        .run();
}
