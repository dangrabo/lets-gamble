use bevy::prelude::*;
use bevy_simple_text_input::{
    TextInput, TextInputInactive, TextInputPlaceholder, TextInputPlugin, TextInputSettings,
    TextInputTextColor, TextInputTextFont, TextInputValue,
};
use rand::Rng;

const STARTING_BALANCE: f64 = 10.0;
const FLIP_DURATION: f32 = 1.3;
// Coin images are 3:2 (1536x1024) with the round coin centered, so the node
// must keep that aspect or the circle gets stretched into an oval. The visible
// coin diameter equals the node height.
const COIN_W: f32 = 225.0;
const COIN_H: f32 = 150.0;

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
struct FlipButton;

#[derive(Component)]
struct RestartButton;

#[derive(Resource, Default)]
struct ActiveFlip {
    timer: Timer,
    result: Coin,
}

#[derive(Resource)]
struct CoinAssets {
    heads: Handle<Image>,
    tails: Handle<Image>,
}

impl CoinAssets {
    fn face(&self, coin: Coin) -> Handle<Image> {
        match coin {
            Coin::Heads => self.heads.clone(),
            Coin::Tails => self.tails.clone(),
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Lucky Flip - Coin Toss Gambler".into(),
                resolution: (1280u32, 720u32).into(),
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

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let coin_assets = CoinAssets {
        heads: asset_server.load("coin_heads.png"),
        tails: asset_server.load("coin_tails.png"),
    };
    let initial_coin = coin_assets.heads.clone();

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|root| {
            // Full-window casino scene behind everything.
            root.spawn((
                ImageNode::new(asset_server.load("casino_scene.png")),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    top: Val::Px(0.0),
                    left: Val::Px(0.0),
                    ..default()
                },
                GlobalZIndex(-1),
            ));

            // Top bar: balance (left) and status message (right).
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    padding: UiRect::axes(Val::Px(28.0), Val::Px(14.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            ))
            .with_children(|bar| {
                bar.spawn((
                    Text::new("Balance: $0.00"),
                    TextFont {
                        font_size: 38.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.85, 0.30)),
                    BalanceText,
                ));
                bar.spawn((
                    Text::new("Place your bet!"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.92, 0.92, 0.92)),
                    StatusText,
                ));
            });

            // Middle: the coin sits low on the felt (FlexEnd) so it doesn't
            // cover the dealer's face higher up in the scene.
            root.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                padding: UiRect::bottom(Val::Px(36.0)),
                ..default()
            })
            .with_children(|mid| {
                // Fixed-size slot keeps layout stable while the coin squashes.
                mid.spawn(Node {
                    width: Val::Px(COIN_W),
                    height: Val::Px(COIN_H),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|slot| {
                    slot.spawn((
                        CoinNode,
                        ImageNode::new(initial_coin),
                        Node {
                            width: Val::Px(COIN_W),
                            height: Val::Px(COIN_H),
                            ..default()
                        },
                    ));
                });
            });

            // Bottom HUD: all the betting controls on a dark panel.
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(14.0)),
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.60)),
            ))
            .with_children(|hud| {
                hud.spawn((
                    Text::new("Bet: $1.00 on HEADS"),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.85, 0.95, 0.88)),
                    BetText,
                ));

                // Side selection + custom amount row
                hud.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(10.0),
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
                            width: Val::Px(160.0),
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

                // Bet amount row ($1 / $5 / $10 / All-in)
                hud.spawn(Node {
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

                // FLIP button
                hud.spawn((
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(54.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
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
                hud.spawn((
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
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
        });

    commands.insert_resource(coin_assets);
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
    coin_assets: Res<CoinAssets>,
    mut coin_q: Query<(&mut Node, &mut ImageNode), With<CoinNode>>,
    mut status_q: Query<(&mut Text, &mut TextColor), With<StatusText>>,
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

    if let Ok((mut node, mut image)) = coin_q.single_mut() {
        // Squash the coin's height to fake a vertical spin.
        let squash = if finished {
            1.0
        } else {
            (elapsed * 18.0).cos().abs() * 0.85 + 0.15
        };
        node.height = Val::Px(COIN_H * squash);
        image.image = coin_assets.face(shown);
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
    coin_assets: Res<CoinAssets>,
    mut coin_q: Query<&mut ImageNode, With<CoinNode>>,
    mut status_q: Query<(&mut Text, &mut TextColor), With<StatusText>>,
) {
    let broke = bank.balance <= 0.0;

    for (interaction, mut bg, mut node) in &mut buttons {
        node.display = if broke { Display::Flex } else { Display::None };

        if broke && *interaction == Interaction::Pressed {
            bank.balance = STARTING_BALANCE;
            bet.amount = 1.0;
            bet.side = Coin::Heads;
            state.phase = Phase::Idle;
            if let Ok(mut image) = coin_q.single_mut() {
                image.image = coin_assets.heads.clone();
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
