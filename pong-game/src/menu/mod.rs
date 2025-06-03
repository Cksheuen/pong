use bevy::{
    app::AppExit,
    prelude::*,
};

use super::{GameState, despawn_screen};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
    #[default]
    Disabled,
    Main,
}

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Quit,
    Practice,
}

#[derive(Component)]
struct OnMainMenuScreen;

pub fn menu_plugin(app: &mut App) {
    app.init_state::<MenuState>()
        .add_systems(OnEnter(GameState::Menu), menu_setup)
        .add_systems(OnEnter(MenuState::Main), main_menu_setup)
        .add_systems(OnExit(MenuState::Main), despawn_screen::<OnMainMenuScreen>)
        .add_systems(
            Update,
            (button_system, menu_action).run_if(in_state(GameState::Menu)),
        );
}

fn menu_setup(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

#[derive(Component)]
struct MenuCamera;

fn main_menu_setup(mut commands: Commands) {
    commands.spawn((Camera2d, MenuCamera, OnMainMenuScreen));
    let button_node = Node {
        // width: Val::Px(100.0),
        // height: Val::Px(100.0),
        margin: UiRect::all(Val::Px(10.0)),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,

        ..default()
    };
    let button_text = TextFont {
        font_size: 32.0,
        ..default()
    };

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(30.0),
                ..default()
            },
            OnMainMenuScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Pong Game"),
                TextFont {
                    font_size: 67.0,
                    ..default()
                },
                OnMainMenuScreen
            ));
            parent.spawn((
                Text::new("Start Game"),
                button_text.clone(),
                Button,
                button_node.clone(),
                MenuButtonAction::Play,
                OnMainMenuScreen
            ));
            parent.spawn((
                Text::new("Practice Mode"),
                button_text.clone(),
                Button,
                button_node.clone(),
                MenuButtonAction::Practice,
                OnMainMenuScreen
            ));
            parent.spawn((
                Text::new("Exit"),
                button_text.clone(),
                Button,
                button_node.clone(),
                MenuButtonAction::Quit,
                OnMainMenuScreen
            ));
        });
}

#[derive(Component)]
struct SelectedOption;

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const HOVERED_PRESSED_BUTTON: Color = Color::srgb(0.25, 0.65, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut background_color, selected) in &mut interaction_query {
        *background_color = match (*interaction, selected) {
            (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
            (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
            (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
            (Interaction::None, None) => NORMAL_BUTTON.into(),
        }
    }
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Pressed {
            match menu_button_action {
                MenuButtonAction::Quit => {
                    app_exit_events.send(AppExit::Success);
                }
                MenuButtonAction::Play => {
                    game_state.set(GameState::GameEntering);
                    menu_state.set(MenuState::Disabled);
                }
                MenuButtonAction::Practice => {
                    game_state.set(GameState::GamePracticeEntering);
                    menu_state.set(MenuState::Disabled);
                }
            }
        }
    }
}
