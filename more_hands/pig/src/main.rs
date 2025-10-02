use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use my_library::RandomNumberGenerator;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
enum GamePhase {
    #[default]
    Player,
    Cpu,
}

#[derive(Resource)]
/// This holds the handle the dice graphics. It represents an index to the
/// stored graphics for reuse
struct GameAssets {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

#[derive(Clone, Copy, Resource)]
/// Current game score
struct Scores {
    player: usize,
    cpu: usize,
}

#[derive(Component)]
/// These is a marker to represent dice on the screen
struct HandDie;

#[derive(Resource)]
/// Wraps our Library's `RandomNumberGenerator` in a bevy resource.
struct Random(RandomNumberGenerator);

/// Wraps `Timer` in a bevy resource.
#[derive(Resource)]
struct HandTImer(Timer);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin {
            enable_multipass_for_primary_context: false,
        })
        .add_systems(Startup, setup)
        .init_state::<GamePhase>()
        .add_systems(Update, display_score)
        // .add_systems(Update, player.run_if(in_state(GamePhase::Player)))
        // .add_systems(Update, cpu.run_if(in_state(GamePhase::Cpu)))
        .run();
}

fn setup(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2d::default());

    // Load dice asset, define a grid of 6 images in a row with 52 pixels in size
    let texture = asset_server.load("dice.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(52), 6, 1, None, None);
    let texture_alias_layout = texture_atlas_layouts.add(layout);

    commands.insert_resource(GameAssets {
        image: texture,
        layout: texture_alias_layout,
    });

    commands.insert_resource(Scores { cpu: 0, player: 0 });
    commands.insert_resource(Random(RandomNumberGenerator::new()));
    commands.insert_resource(HandTImer(Timer::from_seconds(0.5, TimerMode::Repeating)));
}

fn display_score(scores: Res<Scores>, mut egui_context: EguiContexts) {
    egui::Window::new("Total Scores").show(egui_context.ctx_mut(), |ui| {
        ui.label(&format!("Player: {}", scores.player));
        ui.label(&format!("CPU: {}", scores.cpu));
    });
}
