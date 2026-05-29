use bevy::prelude::*;
use bevy_simple_text_input::{
    TextInput, TextInputInactive, TextInputPlaceholder, TextInputPlugin, TextInputSettings,
    TextInputTextColor, TextInputTextFont, TextInputValue,
};
use rand::Rng;

const STARTING_BALANCE: f64 = 10.0;
const FLIP_DURATION: f32 = 1.3;
const COIN_SIZE: f32 = 120.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Coin {
    #[default]
    Heads,
    Tails,
}

impl Coin {
    fn label(self) -> &'static str {
        match self {
            Coin::Heads => "HEADS",
            Coin::Tails => "TAILS",
        }
    }

    fn short(self) -> &'static str {
        match self {
            Coin::Heads => "H",
            Coin::Tails => "T",
        }
    }
}

#[derive(Resource)]
struct Bank {
    balance: f64,
}

impl Default for Bank {
    fn default() -> Self {
        Bank {
            balance: STARTING_BALANCE,
        }
    }
}

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
enum Phase {
    #[default]
    Idle,
    Flipping,
    Result,
}

#[derive(Resource, Default)]
struct GameState {
    phase: Phase,
}

#[derive(Resource)]
struct CurrentBet {
    side: Coin,
    amount: f64,
}

impl Default for CurrentBet {
    fn default() -> Self {
        CurrentBet {
            side: Coin::Heads,
            amount: 1.0,
        }
    }
}

#[derive(Component)]
struct BalanceText;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct BetText;

#[derive(Component)]
struct SideButton(Coin);

#[derive(Component, Clone, Copy)]
enum BetButton {
    Set(f64),
    AllIn,
}

#[derive(Component)]
struct CustomAmountInput;

#[derive(Component)]
struct CoinNode;

#[derive(Component)]
struct CoinFaceText;

#[derive(Component)]
struct FlipButton;

#[derive(Component)]
struct RestartButton;

#[derive(Resource, Default)]
struct ActiveFlip {
    timer: Timer,
    result: Coin,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Lucky Flip - Coin Toss Gambler".into(),
                resolution: (800u32, 600u32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TextInputPlugin)
        .insert_resource(ClearColor(Color::srgb(0.05, 0.18, 0.10)))
        .init_resource::<Bank>()
        .init_resource::<GameState>()
        .init_resource::<CurrentBet>()
        .init_resource::<ActiveFlip>()
        .add_systems(Startup, (setup_camera, setup_ui))
        .add_systems(
            Update,
            (
                update_balance_text,
                update_bet_text,
                side_button_system,
                bet_button_system,
                custom_amount_system,
                flip_button_system,
                flip_animation_system,
                restart_button_system,
            ),
        )
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn button_node() -> Node {
    Node {
        width: Val::Px(120.0),
        height: Val::Px(48.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        margin: UiRect::all(Val::Px(6.0)),
        ..default()
    }
}

fn button_text(label: &str) -> (Text, TextFont, TextColor) {
    (
        Text::new(label),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.95, 0.95)),
    )
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(14.0),
            ..default()
        })
        .with_children(|root| {
            root.spawn((
                Text::new("Balance: $0.00"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.92, 0.45)),
                BalanceText,
            ));

            root.spawn((
                Text::new("Place your bet!"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                StatusText,
            ));

            root.spawn((
                Text::new("Bet: $1.00 on HEADS"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.95, 0.85)),
                BetText,
            ));

            // Side selection row (Heads / Tails)
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .with_children(|row| {
                for side in [Coin::Heads, Coin::Tails] {
                    row.spawn((
                        Button,
                        button_node(),
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
                        SideButton(side),
                    ))
                    .with_child(button_text(side.label()));
                }
            });

            // Bet amount row ($1 / $5 / $10 / All-in)
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                ..default()
            })
            .with_children(|row| {
                let presets: [(&str, BetButton); 4] = [
                    ("$1", BetButton::Set(1.0)),
                    ("$5", BetButton::Set(5.0)),
                    ("$10", BetButton::Set(10.0)),
                    ("All-in", BetButton::AllIn),
                ];
                for (label, action) in presets {
                    row.spawn((
                        Button,
                        button_node(),
                        BackgroundColor(Color::srgb(0.15, 0.15, 0.18)),
                        action,
                    ))
                    .with_child(button_text(label));
                }
            });

            // Custom amount input row
            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new("Custom $:"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
                row.spawn((
                    TextInput,
                    TextInputValue(String::new()),
                    TextInputTextFont(TextFont {
                        font_size: 20.0,
                        ..default()
                    }),
                    TextInputTextColor(TextColor(Color::WHITE)),
                    TextInputInactive(false),
                    TextInputSettings {
                        max_length: Some(9),
                        ..default()
                    },
                    TextInputPlaceholder {
                        value: "type amount".to_string(),
                        ..default()
                    },
                    CustomAmountInput,
                    Node {
                        width: Val::Px(170.0),
                        height: Val::Px(38.0),
                        padding: UiRect::all(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BorderColor::all(Color::srgb(0.6, 0.6, 0.6)),
                    BackgroundColor(Color::srgb(0.10, 0.10, 0.12)),
                ));
            });

            // The coin: a fixed-size slot reserves constant layout space so the
            // FLIP button below stays put while the inner coin squashes.
            root.spawn(Node {
                width: Val::Px(COIN_SIZE),
                height: Val::Px(COIN_SIZE),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                margin: UiRect::top(Val::Px(8.0)),
                ..default()
            })
            .with_children(|slot| {
                slot.spawn((
                    CoinNode,
                    Node {
                        width: Val::Px(COIN_SIZE),
                        height: Val::Px(COIN_SIZE),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Percent(50.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.95, 0.82, 0.25)),
                ))
                .with_child((
                    Text::new("?"),
                    TextFont {
                        font_size: 56.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.2, 0.15, 0.0)),
                    CoinFaceText,
                ));
            });

            // FLIP button
            root.spawn((
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(56.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.85, 0.65, 0.10)),
                FlipButton,
            ))
            .with_child((
                Text::new("FLIP!"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::BLACK),
            ));

            // Restart button (hidden until the player goes bankrupt)
            root.spawn((
                Button,
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(8.0)),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.70, 0.20, 0.20)),
                RestartButton,
            ))
            .with_child((
                Text::new("Restart ($10)"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn update_balance_text(bank: Res<Bank>, mut query: Query<&mut Text, With<BalanceText>>) {
    if !bank.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = format!("Balance: ${:.2}", bank.balance);
    }
}

fn update_bet_text(bet: Res<CurrentBet>, mut query: Query<&mut Text, With<BetText>>) {
    if !bet.is_changed() {
        return;
    }
    for mut text in &mut query {
        **text = format!("Bet: ${:.2} on {}", bet.amount, bet.side.label());
    }
}

fn side_button_system(
    mut query: Query<(&Interaction, &SideButton, &mut BackgroundColor)>,
    mut bet: ResMut<CurrentBet>,
) {
    for (interaction, side, _) in query.iter() {
        if *interaction == Interaction::Pressed {
            bet.side = side.0;
        }
    }
    for (interaction, side, mut bg) in query.iter_mut() {
        let selected = bet.side == side.0;
        *bg = BackgroundColor(match (*interaction, selected) {
            (Interaction::Pressed, _) => Color::srgb(0.35, 0.35, 0.40),
            (_, true) => Color::srgb(0.18, 0.50, 0.30),
            (Interaction::Hovered, false) => Color::srgb(0.25, 0.25, 0.30),
            (Interaction::None, false) => Color::srgb(0.15, 0.15, 0.18),
        });
    }
}

fn bet_button_system(
    mut query: Query<(&Interaction, &BetButton, &mut BackgroundColor)>,
    mut bet: ResMut<CurrentBet>,
    bank: Res<Bank>,
) {
    for (interaction, button, mut bg) in query.iter_mut() {
        if *interaction == Interaction::Pressed {
            let requested = match button {
                BetButton::Set(value) => *value,
                BetButton::AllIn => bank.balance,
            };
            bet.amount = requested.min(bank.balance).max(0.0);
        }
        *bg = BackgroundColor(match *interaction {
            Interaction::Pressed => Color::srgb(0.35, 0.35, 0.40),
            Interaction::Hovered => Color::srgb(0.25, 0.25, 0.30),
            Interaction::None => Color::srgb(0.15, 0.15, 0.18),
        });
    }
}

fn custom_amount_system(
    query: Query<&TextInputValue, (Changed<TextInputValue>, With<CustomAmountInput>)>,
    mut bet: ResMut<CurrentBet>,
    bank: Res<Bank>,
) {
    for value in &query {
        let trimmed = value.0.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(parsed) = trimmed.parse::<f64>() {
            if parsed.is_finite() && parsed >= 0.0 {
                bet.amount = parsed.min(bank.balance);
            }
        }
    }
}

fn flip_button_system(
    mut buttons: Query<(&Interaction, &mut BackgroundColor), With<FlipButton>>,
    mut state: ResMut<GameState>,
    mut active: ResMut<ActiveFlip>,
    bet: Res<CurrentBet>,
    bank: Res<Bank>,
    mut status: Query<(&mut Text, &mut TextColor), With<StatusText>>,
) {
    let can_flip = state.phase != Phase::Flipping
        && bet.amount > 0.0
        && bet.amount <= bank.balance + 1e-9;

    for (interaction, mut bg) in &mut buttons {
        if can_flip && *interaction == Interaction::Pressed {
            let mut rng = rand::thread_rng();
            active.result = if rng.gen_bool(0.5) {
                Coin::Heads
            } else {
                Coin::Tails
            };
            active.timer = Timer::from_seconds(FLIP_DURATION, TimerMode::Once);
            state.phase = Phase::Flipping;
            if let Ok((mut text, mut color)) = status.single_mut() {
                **text = "Flipping...".to_string();
                *color = TextColor(Color::srgb(0.9, 0.9, 0.9));
            }
        }

        *bg = BackgroundColor(if !can_flip {
            Color::srgb(0.35, 0.35, 0.35)
        } else {
            match *interaction {
                Interaction::Pressed => Color::srgb(0.70, 0.50, 0.05),
                Interaction::Hovered => Color::srgb(0.95, 0.75, 0.20),
                Interaction::None => Color::srgb(0.85, 0.65, 0.10),
            }
        });
    }
}

fn flip_animation_system(
    time: Res<Time>,
    mut state: ResMut<GameState>,
    mut active: ResMut<ActiveFlip>,
    mut bank: ResMut<Bank>,
    bet: Res<CurrentBet>,
    mut coin_q: Query<(&mut Node, &mut BackgroundColor), With<CoinNode>>,
    mut face_q: Query<&mut Text, With<CoinFaceText>>,
    mut status_q: Query<(&mut Text, &mut TextColor), (With<StatusText>, Without<CoinFaceText>)>,
) {
    if state.phase != Phase::Flipping {
        return;
    }

    active.timer.tick(time.delta());
    let elapsed = active.timer.elapsed_secs();
    let finished = active.timer.is_finished();

    let shown = if finished {
        active.result
    } else if (elapsed * 12.0) as i32 % 2 == 0 {
        Coin::Heads
    } else {
        Coin::Tails
    };

    if let Ok(mut text) = face_q.single_mut() {
        **text = shown.short().to_string();
    }

    if let Ok((mut node, mut bg)) = coin_q.single_mut() {
        let squash = if finished {
            1.0
        } else {
            (elapsed * 18.0).cos().abs() * 0.85 + 0.15
        };
        node.height = Val::Px(COIN_SIZE * squash);
        *bg = BackgroundColor(match shown {
            Coin::Heads => Color::srgb(0.95, 0.82, 0.25),
            Coin::Tails => Color::srgb(0.80, 0.80, 0.85),
        });
    }

    if finished {
        let win = active.result == bet.side;
        if win {
            bank.balance += bet.amount;
        } else {
            bank.balance -= bet.amount;
        }
        if bank.balance < 0.0 {
            bank.balance = 0.0;
        }
        if let Ok((mut text, mut color)) = status_q.single_mut() {
            if bank.balance <= 0.0 {
                **text = format!("It's {}. BANKRUPT - you're out of money!", active.result.label());
                *color = TextColor(Color::srgb(1.0, 0.30, 0.30));
            } else if win {
                **text = format!("It's {}! You WON ${:.2}!", active.result.label(), bet.amount);
                *color = TextColor(Color::srgb(0.40, 1.0, 0.50));
            } else {
                **text = format!("It's {}. You lost ${:.2}.", active.result.label(), bet.amount);
                *color = TextColor(Color::srgb(1.0, 0.60, 0.40));
            }
        }
        state.phase = Phase::Result;
    }
}

fn restart_button_system(
    mut buttons: Query<(&Interaction, &mut BackgroundColor, &mut Node), With<RestartButton>>,
    mut bank: ResMut<Bank>,
    mut bet: ResMut<CurrentBet>,
    mut state: ResMut<GameState>,
    mut face_q: Query<&mut Text, With<CoinFaceText>>,
    mut status_q: Query<(&mut Text, &mut TextColor), (With<StatusText>, Without<CoinFaceText>)>,
) {
    let broke = bank.balance <= 0.0;

    for (interaction, mut bg, mut node) in &mut buttons {
        node.display = if broke { Display::Flex } else { Display::None };

        if broke && *interaction == Interaction::Pressed {
            bank.balance = STARTING_BALANCE;
            bet.amount = 1.0;
            bet.side = Coin::Heads;
            state.phase = Phase::Idle;
            if let Ok(mut text) = face_q.single_mut() {
                **text = "?".to_string();
            }
            if let Ok((mut text, mut color)) = status_q.single_mut() {
                **text = "Place your bet!".to_string();
                *color = TextColor(Color::srgb(0.9, 0.9, 0.9));
            }
        }

        *bg = BackgroundColor(match *interaction {
            Interaction::Pressed => Color::srgb(0.50, 0.10, 0.10),
            Interaction::Hovered => Color::srgb(0.85, 0.28, 0.28),
            Interaction::None => Color::srgb(0.70, 0.20, 0.20),
        });
    }
}
