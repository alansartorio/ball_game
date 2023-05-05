use bevy::{app::AppExit, prelude::*};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_menu).add_system(menu_action);
    }
}

#[derive(Component)]
enum MenuButtonAction {
    Play,
    Quit,
}

fn setup_menu(mut commands: Commands, assets: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    let font = assets.load::<Font, _>("fonts/OpenSans-Regular.ttf");

    let button_text_style = TextStyle {
        font_size: 30.0,
        color: Color::BLACK,
        font: font.clone(),
    };

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(
                TextBundle::from_section(
                    "Test Menu",
                    TextStyle {
                        font_size: 40.0,
                        color: Color::WHITE,
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
                        background_color: Color::WHITE.into(),
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
                        background_color: Color::WHITE.into(),
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
) {
    for (interaction, menu_button_action) in &interaction_query {
        if *interaction == Interaction::Clicked {
            match menu_button_action {
                MenuButtonAction::Quit => app_exit_events.send(AppExit),
                MenuButtonAction::Play => {
                    //game_state.set(GameState::Game);
                    println!("Play");
                }
            }
        }
    }
}
