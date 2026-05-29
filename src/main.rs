use bevy::prelude::*;

const STARTING_BALANCE: f64 = 10.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Coin {
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
        .insert_resource(ClearColor(Color::srgb(0.05, 0.18, 0.10)))
        .init_resource::<Bank>()
        .init_resource::<GameState>()
        .init_resource::<CurrentBet>()
        .add_systems(Startup, (setup_camera, setup_ui))
        .add_systems(
            Update,
            (
                update_balance_text,
                update_bet_text,
                side_button_system,
                bet_button_system,
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
