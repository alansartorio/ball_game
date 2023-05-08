use bevy::{app::AppExit, prelude::*};

use crate::{despawn_screen, GameState, colors};

#[derive(Component)]
pub struct OnMenu;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), menu_setup)
            .add_systems(Update, menu_action.run_if(in_state(GameState::Menu)))
            .add_systems(OnExit(GameState::Menu), despawn_screen::<OnMenu>);
    }
}

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Quit,
}

fn menu_setup(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn((Camera2dBundle::default(), OnMenu));
    let font = assets.load::<Font, _>("fonts/OpenSans-Regular.ttf");

    let button_text_style = TextStyle {
        font_size: 30.0,
        color: *colors::LIGHT_TEXT,
        font: font.clone(),
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            OnMenu,
        ))
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    "Test Menu",
                    TextStyle {
                        font_size: 40.0,
                        color: *colors::DARK_TEXT,
                        font,
                    },
                )
                .with_style(Style {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                }),
            );
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        background_color: (*colors::BUTTON_BACKGROUND).into(),
                        ..default()
                    },
                    MenuButtonAction::Play,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Play", button_text_style.clone()));
                });
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            margin: UiRect::all(Val::Px(20.0)),
                            ..default()
                        },
                        background_color: (*colors::BUTTON_BACKGROUND).into(),
                        ..default()
                    },
                    MenuButtonAction::Quit,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Quit", button_text_style.clone()));
                });
        });
}

fn menu_action(
    interaction_query: Query<
        (&Interaction, &MenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_exit_events: EventWriter<AppExit>,
    mut game_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Clicked {
            match menu_button_action {
                MenuButtonAction::Quit => app_exit_events.send(AppExit),
                MenuButtonAction::Play => {
                    game_state.set(GameState::Game);
                }
            }
        }
    }
}
