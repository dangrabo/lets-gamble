use bevy::prelude::*;

const STARTING_BALANCE: f64 = 10.0;

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

#[derive(Component)]
struct BalanceText;

#[derive(Component)]
struct StatusText;

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
        .add_systems(Startup, (setup_camera, setup_ui))
        .add_systems(Update, update_balance_text)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn setup_ui(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(16.0),
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
