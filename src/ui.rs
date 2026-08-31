use bevy::prelude::*;
use crate::cube::{
    trigger_reset, trigger_scramble, trigger_undo, Cubie, Face, GameTimerState,
    MoveCommand, MoveHistory, MoveQueue, RotationState, ScrambleInfo,
};
use crate::interaction::CameraOrbit;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<UiHoverState>()
            .init_resource::<UiControlsState>()
            .add_systems(Startup, setup_ui)
            .add_systems(
                Update,
                (
                    button_interaction_system,
                    update_hud_system,
                    toggle_controls_keyboard_system,
                    sync_ui_visibility_system,
                ),
            );
    }
}

#[derive(Resource, Default)]
pub struct UiHoverState {
    pub is_hovering_ui: bool,
}

#[derive(Resource)]
pub struct UiControlsState {
    pub show_face_buttons: bool,
    pub is_inverse: bool,
    pub is_clean_screen: bool,
}

impl Default for UiControlsState {
    fn default() -> Self {
        Self {
            show_face_buttons: false,
            is_inverse: false,
            is_clean_screen: false,
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq)]
pub enum UiButtonAction {
    Scramble,
    Reset,
    Undo,
    ToggleFloating,
    ToggleCleanScreen,
    ToggleFaceControls,
    RotateFace(Face),
    ToggleInverse,
}

#[derive(Component)]
struct HudTimerText;

#[derive(Component)]
struct HudStatusText;

#[derive(Component)]
struct HudScrambleText;

#[derive(Component)]
struct HeaderContainerNode;

#[derive(Component)]
struct FacePanelNode;

#[derive(Component)]
struct ZenRestoreButtonNode;

#[derive(Component)]
struct ToggleFloatingBtnText;

#[derive(Component)]
struct ToggleControlsBtnText;

#[derive(Component)]
struct InverseBtnText;

const BTN_BG: Color = Color::rgba(0.10, 0.14, 0.20, 0.88);
const BTN_HOVER: Color = Color::rgba(0.18, 0.26, 0.38, 0.95);
const BTN_PRESSED: Color = Color::rgba(0.18, 0.52, 0.85, 1.0);

const FACE_BTN_BG: Color = Color::rgba(0.12, 0.16, 0.22, 0.92);
const FACE_BTN_HOVER: Color = Color::rgba(0.24, 0.32, 0.44, 0.98);

fn setup_ui(mut commands: Commands) {
    // Root container transparente ocupando a tela toda
    commands.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
        background_color: BackgroundColor(Color::NONE),
        ..default()
    }).with_children(|root| {

        // ==================== HEADER PRINCIPAL ====================
        root.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(8.0),
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.05, 0.07, 0.11, 0.82)),
                border_color: BorderColor(Color::rgba(0.25, 0.35, 0.48, 0.45)),
                ..default()
            },
            HeaderContainerNode,
        )).with_children(|header| {

            // Bloco Esquerdo: Timer + Movimentos + Status
            header.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..default()
            }).with_children(|info_col| {

                // Linha do Timer e Status
                info_col.spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Baseline,
                        column_gap: Val::Px(10.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::NONE),
                    ..default()
                }).with_children(|timer_row| {
                    timer_row.spawn((
                        TextBundle::from_section(
                            "00:00.00",
                            TextStyle {
                                font_size: 26.0,
                                color: Color::WHITE,
                                ..default()
                            },
                        ),
                        HudTimerText,
                    ));

                    timer_row.spawn((
                        TextBundle::from_section(
                            "0 MOVS | PRONTO",
                            TextStyle {
                                font_size: 13.0,
                                color: Color::rgb(0.7, 0.75, 0.82),
                                ..default()
                            },
                        ),
                        HudStatusText,
                    ));
                });

                // Linha de Scramble (aparece quando embaralhado)
                info_col.spawn((
                    TextBundle::from_section(
                        "",
                        TextStyle {
                            font_size: 11.5,
                            color: Color::rgba(0.65, 0.75, 0.88, 0.85),
                            ..default()
                        },
                    ),
                    HudScrambleText,
                ));
            });

            // Bloco Direito: Botões de Ação Touch-Friendly
            header.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..default()
            }).with_children(|btn_group| {
                spawn_responsive_btn(btn_group, "Embaralhar", UiButtonAction::Scramble);
                spawn_responsive_btn(btn_group, "Desfazer", UiButtonAction::Undo);
                spawn_responsive_btn(btn_group, "Reset", UiButtonAction::Reset);

                // Botão de Flutuar / Flutuação no Espaço
                btn_group.spawn((
                    ButtonBundle {
                        style: Style {
                            min_height: Val::Px(38.0),
                            padding: UiRect::axes(Val::Px(11.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(BTN_BG),
                        border_color: BorderColor(Color::rgba(0.3, 0.4, 0.55, 0.4)),
                        ..default()
                    },
                    UiButtonAction::ToggleFloating,
                )).with_children(|btn| {
                    btn.spawn((
                        TextBundle::from_section(
                            "Flutuar",
                            TextStyle {
                                font_size: 13.0,
                                color: Color::rgb(0.9, 0.92, 0.95),
                                ..default()
                            },
                        ),
                        ToggleFloatingBtnText,
                    ));
                });

                // Botão de Tela Limpa (Modo Zen)
                spawn_responsive_btn(btn_group, "Tela Limpa", UiButtonAction::ToggleCleanScreen);

                // Botão para Alternar a Exibição dos Botões de Face
                btn_group.spawn((
                    ButtonBundle {
                        style: Style {
                            min_height: Val::Px(38.0),
                            padding: UiRect::axes(Val::Px(11.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(BTN_BG),
                        border_color: BorderColor(Color::rgba(0.3, 0.4, 0.55, 0.4)),
                        ..default()
                    },
                    UiButtonAction::ToggleFaceControls,
                )).with_children(|btn| {
                    btn.spawn((
                        TextBundle::from_section(
                            "Botoes",
                            TextStyle {
                                font_size: 13.0,
                                color: Color::rgb(0.85, 0.88, 0.95),
                                ..default()
                            },
                        ),
                        ToggleControlsBtnText,
                    ));
                });
            });
        });

        // ==================== BOTÃO DISCRETO PARA RESTAURAR HUD (MODO TELA LIMPA) ====================
        root.spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(12.0),
                    right: Val::Px(12.0),
                    display: Display::None,
                    ..default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
            ZenRestoreButtonNode,
        )).with_children(|zen_wrap| {
            zen_wrap.spawn((
                ButtonBundle {
                    style: Style {
                        min_height: Val::Px(36.0),
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.08, 0.12, 0.18, 0.70)),
                    border_color: BorderColor(Color::rgba(0.3, 0.45, 0.65, 0.50)),
                    ..default()
                },
                UiButtonAction::ToggleCleanScreen,
            )).with_children(|btn| {
                btn.spawn(TextBundle::from_section(
                    "Mostrar Menu",
                    TextStyle {
                        font_size: 12.5,
                        color: Color::rgba(0.85, 0.9, 1.0, 0.85),
                        ..default()
                    },
                ));
            });
        });

        // ==================== PAINEL INFERIOR DE FACES (OPCIONAL) ====================
        root.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    display: Display::None,
                    ..default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
            FacePanelNode,
        )).with_children(|panel| {
            panel.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(6.0),
                    row_gap: Val::Px(6.0),
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(8.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                background_color: BackgroundColor(Color::rgba(0.05, 0.07, 0.11, 0.88)),
                border_color: BorderColor(Color::rgba(0.25, 0.35, 0.48, 0.45)),
                ..default()
            }).with_children(|face_row| {
                spawn_face_btn(face_row, "U", Face::U, Color::rgb(0.95, 0.95, 0.95));
                spawn_face_btn(face_row, "D", Face::D, Color::rgb(1.0, 0.88, 0.2));
                spawn_face_btn(face_row, "R", Face::R, Color::rgb(0.9, 0.2, 0.2));
                spawn_face_btn(face_row, "L", Face::L, Color::rgb(1.0, 0.5, 0.1));
                spawn_face_btn(face_row, "F", Face::F, Color::rgb(0.1, 0.85, 0.35));
                spawn_face_btn(face_row, "B", Face::B, Color::rgb(0.2, 0.5, 1.0));

                face_row.spawn((
                    ButtonBundle {
                        style: Style {
                            min_height: Val::Px(38.0),
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            margin: UiRect::left(Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(FACE_BTN_BG),
                        border_color: BorderColor(Color::rgba(0.3, 0.4, 0.55, 0.4)),
                        ..default()
                    },
                    UiButtonAction::ToggleInverse,
                )).with_children(|btn| {
                    btn.spawn((
                        TextBundle::from_section(
                            "Inverter",
                            TextStyle {
                                font_size: 13.0,
                                color: Color::rgb(0.85, 0.85, 0.9),
                                ..default()
                            },
                        ),
                        InverseBtnText,
                    ));
                });
            });
        });
    });
}

fn spawn_responsive_btn(parent: &mut ChildBuilder, label: &str, action: UiButtonAction) {
    parent.spawn((
        ButtonBundle {
            style: Style {
                min_height: Val::Px(38.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(BTN_BG),
            border_color: BorderColor(Color::rgba(0.3, 0.4, 0.55, 0.4)),
            ..default()
        },
        action,
    )).with_children(|btn| {
        btn.spawn(TextBundle::from_section(
            label,
            TextStyle {
                font_size: 13.0,
                color: Color::rgb(0.9, 0.92, 0.95),
                ..default()
            },
        ));
    });
}

fn spawn_face_btn(parent: &mut ChildBuilder, label: &str, face: Face, color: Color) {
    parent.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(42.0),
                height: Val::Px(38.0),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: BackgroundColor(FACE_BTN_BG),
            border_color: BorderColor(Color::rgba(0.3, 0.4, 0.55, 0.4)),
            ..default()
        },
        UiButtonAction::RotateFace(face),
    )).with_children(|btn| {
        btn.spawn(TextBundle::from_section(
            label,
            TextStyle {
                font_size: 15.0,
                color,
                ..default()
            },
        ));
    });
}

fn button_interaction_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &UiButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    all_buttons: Query<&Interaction, With<Button>>,
    mut ui_hover: ResMut<UiHoverState>,
    mut ui_controls: ResMut<UiControlsState>,
    mut queue: ResMut<MoveQueue>,
    mut history: ResMut<MoveHistory>,
    mut state: ResMut<RotationState>,
    mut timer_state: ResMut<GameTimerState>,
    mut scramble_info: ResMut<ScrambleInfo>,
    mut cubies: Query<(Entity, &mut Transform, &mut Cubie)>,
    mut orbit_query: Query<&mut CameraOrbit>,
    mut floating_text_query: Query<&mut Text, (With<ToggleFloatingBtnText>, Without<InverseBtnText>)>,
    mut inverse_text_query: Query<&mut Text, (With<InverseBtnText>, Without<ToggleFloatingBtnText>)>,
) {
    ui_hover.is_hovering_ui = all_buttons.iter().any(|i| *i == Interaction::Hovered || *i == Interaction::Pressed);

    for (interaction, mut bg_color, action) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BTN_PRESSED.into();
                match action {
                    UiButtonAction::Scramble => {
                        trigger_scramble(&mut queue, &mut timer_state, &mut scramble_info);
                    }
                    UiButtonAction::Reset => {
                        trigger_reset(&mut queue, &mut history, &mut state, &mut timer_state, &mut scramble_info, &mut cubies);
                    }
                    UiButtonAction::Undo => {
                        trigger_undo(&mut queue, &mut history, &state);
                    }
                    UiButtonAction::ToggleFloating => {
                        if let Ok(mut orbit) = orbit_query.get_single_mut() {
                            orbit.floating = !orbit.floating;
                            for mut text in &mut floating_text_query {
                                text.sections[0].value = if orbit.floating {
                                    "Flutuando".to_string()
                                } else {
                                    "Flutuar".to_string()
                                };
                                text.sections[0].style.color = if orbit.floating {
                                    Color::rgb(0.3, 0.9, 1.0)
                                } else {
                                    Color::rgb(0.9, 0.92, 0.95)
                                };
                            }
                        }
                    }
                    UiButtonAction::ToggleCleanScreen => {
                        ui_controls.is_clean_screen = !ui_controls.is_clean_screen;
                    }
                    UiButtonAction::ToggleFaceControls => {
                        ui_controls.show_face_buttons = !ui_controls.show_face_buttons;
                    }
                    UiButtonAction::RotateFace(face) => {
                        queue.queue.push_back(MoveCommand::from_face(*face, ui_controls.is_inverse, false));
                    }
                    UiButtonAction::ToggleInverse => {
                        ui_controls.is_inverse = !ui_controls.is_inverse;
                        for mut text in &mut inverse_text_query {
                            text.sections[0].value = if ui_controls.is_inverse {
                                "Invertido".to_string()
                            } else {
                                "Inverter".to_string()
                            };
                            text.sections[0].style.color = if ui_controls.is_inverse {
                                Color::rgb(1.0, 0.45, 0.45)
                            } else {
                                Color::rgb(0.85, 0.85, 0.9)
                            };
                        }
                    }
                }
            }
            Interaction::Hovered => {
                *bg_color = match action {
                    UiButtonAction::RotateFace(_) | UiButtonAction::ToggleInverse => FACE_BTN_HOVER.into(),
                    _ => BTN_HOVER.into(),
                };
            }
            Interaction::None => {
                *bg_color = match action {
                    UiButtonAction::RotateFace(_) | UiButtonAction::ToggleInverse => FACE_BTN_BG.into(),
                    _ => BTN_BG.into(),
                };
            }
        }
    }
}

fn toggle_controls_keyboard_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut ui_controls: ResMut<UiControlsState>,
) {
    if keys.just_pressed(KeyCode::Tab) || keys.just_pressed(KeyCode::KeyH) {
        ui_controls.show_face_buttons = !ui_controls.show_face_buttons;
    }
    if keys.just_pressed(KeyCode::KeyZ) {
        ui_controls.is_clean_screen = !ui_controls.is_clean_screen;
    }
}

fn sync_ui_visibility_system(
    ui_controls: Res<UiControlsState>,
    mut header_query: Query<&mut Style, (With<HeaderContainerNode>, Without<FacePanelNode>, Without<ZenRestoreButtonNode>)>,
    mut panel_query: Query<&mut Style, (With<FacePanelNode>, Without<HeaderContainerNode>, Without<ZenRestoreButtonNode>)>,
    mut restore_btn_query: Query<&mut Style, (With<ZenRestoreButtonNode>, Without<HeaderContainerNode>, Without<FacePanelNode>)>,
    mut toggle_btn_query: Query<&mut Text, With<ToggleControlsBtnText>>,
) {
    if ui_controls.is_changed() {
        for mut style in &mut header_query {
            style.display = if ui_controls.is_clean_screen {
                Display::None
            } else {
                Display::Flex
            };
        }

        for mut style in &mut panel_query {
            style.display = if !ui_controls.is_clean_screen && ui_controls.show_face_buttons {
                Display::Flex
            } else {
                Display::None
            };
        }

        for mut style in &mut restore_btn_query {
            style.display = if ui_controls.is_clean_screen {
                Display::Flex
            } else {
                Display::None
            };
        }

        for mut text in &mut toggle_btn_query {
            text.sections[0].value = if ui_controls.show_face_buttons {
                "Botoes (ON)".to_string()
            } else {
                "Botoes".to_string()
            };
        }
    }
}

fn update_hud_system(
    timer_state: Res<GameTimerState>,
    queue: Res<MoveQueue>,
    scramble_info: Res<ScrambleInfo>,
    mut timer_query: Query<&mut Text, (With<HudTimerText>, Without<HudStatusText>, Without<HudScrambleText>, Without<ToggleControlsBtnText>, Without<InverseBtnText>, Without<ToggleFloatingBtnText>)>,
    mut status_query: Query<&mut Text, (With<HudStatusText>, Without<HudTimerText>, Without<HudScrambleText>, Without<ToggleControlsBtnText>, Without<InverseBtnText>, Without<ToggleFloatingBtnText>)>,
    mut scramble_query: Query<&mut Text, (With<HudScrambleText>, Without<HudTimerText>, Without<HudStatusText>, Without<ToggleControlsBtnText>, Without<InverseBtnText>, Without<ToggleFloatingBtnText>)>,
) {
    let total_secs = timer_state.elapsed;
    let mins = (total_secs / 60.0).floor() as u32;
    let secs = (total_secs % 60.0).floor() as u32;
    let millis = ((total_secs % 1.0) * 100.0).floor() as u32;

    for mut text in &mut timer_query {
        text.sections[0].value = format!("{:02}:{:02}.{:02}", mins, secs, millis);
        text.sections[0].style.color = if timer_state.is_solved {
            Color::rgb(0.2, 1.0, 0.4)
        } else if timer_state.is_running {
            Color::rgb(1.0, 0.88, 0.25)
        } else {
            Color::WHITE
        };
    }

    for mut text in &mut status_query {
        let status = if !queue.queue.is_empty() && queue.queue.iter().any(|m| m.is_scramble) {
            "EMBARALHANDO..."
        } else if timer_state.is_solved {
            "RESOLVIDO"
        } else if timer_state.is_running {
            "EM ANDAMENTO"
        } else if timer_state.is_scrambled {
            "AGUARDANDO INICIO"
        } else {
            "PRONTO"
        };

        text.sections[0].value = format!("{} MOVS | {}", timer_state.move_count, status);
        text.sections[0].style.color = if timer_state.is_solved {
            Color::rgb(0.2, 1.0, 0.4)
        } else if timer_state.is_running {
            Color::rgb(1.0, 0.85, 0.2)
        } else {
            Color::rgb(0.65, 0.7, 0.78)
        };
    }

    for mut text in &mut scramble_query {
        if !scramble_info.sequence.is_empty() {
            text.sections[0].value = scramble_info.sequence.clone();
        } else {
            text.sections[0].value.clear();
        }
    }
}
