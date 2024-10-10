//! Developer console for bevy-wormhole.

use bevy::{
    app::AppExit,
    diagnostic::SystemInfo,
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
        mouse::{MouseScrollUnit, MouseWheel},
    },
    ecs::system::SystemId,
    prelude::*,
    utils::HashMap,
};

/// Developer console plugin.
pub struct ConsolePlugin;

impl Plugin for ConsolePlugin {
    fn build(&self, app: &mut App) {
        println!("ConsolePlugin::build()");
        app.add_event::<StdOutEvent>();
        app.add_event::<StdErrEvent>();
        app.init_resource::<CommandMap>();
        app.add_systems(Startup, (setup_console, console_greeter).chain());
        app.add_systems(Update, (cursor_tick, console_input, console_output, console_error, console_scroll));
    }
}

//------------------------------------------------------------------------------

const GREET: &str = "ROBCO INDUSTRIES (TM) TERMLINK PROTOCOL\n";
const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const HELP: &str = r#"debug               Print debug information.
exit                Exit wormhole.
help                Display this help text.
load filename       Load an ESM or BSA file.
system              Print system information.
version             Build information.
"#;

/// Some sort of debug information or mode.
fn command_debug(
    mut stdout: EventWriter<StdOutEvent>,
) {
    stdout.send(StdOutEvent { value: "unimplemented\n".into() });
}

/// Exit the Bevy app.
fn command_exit(
    mut exit: EventWriter<AppExit>,
) {
    exit.send(AppExit::Success);
}

/// Load some asset.
fn command_load(
    mut stdout: EventWriter<StdOutEvent>,
) {
    stdout.send(StdOutEvent { value: "unimplemented\n".into() });
}

/// Print system information.
fn command_system(
    mut stdout: EventWriter<StdOutEvent>,
    system: Res<SystemInfo>,
) {
    let &SystemInfo { os, kernel, cpu, core_count, memory } = &system.into_inner();
    stdout.send(StdOutEvent { value: format!(r#"OS:     {os}
Kernel: {kernel}
CPU:    {cpu}
Cores:  {core_count}
Memory: {memory}
"#) });
}

// Map of all Bevy console commands.
impl FromWorld for CommandMap {
    fn from_world(world: &mut World) -> Self {
        let mut commands = CommandMap(HashMap::new());

        commands.0.insert("debug".into(), world.register_system(command_debug));
        commands.0.insert("exit".into(), world.register_system(command_exit));
        commands.0.insert("load".into(), world.register_system(command_load));
        commands.0.insert("system".into(),  world.register_system(command_system));

        commands
    }
}

/// Simple commands that don't require Bevy systems.
fn shell(
    stdout: &mut EventWriter<StdOutEvent>,
    args: Vec<&str>,
) {
    match args[0] {
        "help" => stdout.send(StdOutEvent { value: HELP.into() }),
        "version" => stdout.send(StdOutEvent { value: format!("{NAME} {VERSION}\n").into() }),
        _ => stdout.send(StdOutEvent { value: format!("unknown command: {}\n", args[0]).into() })
    };
}

//------------------------------------------------------------------------------

/// Custom write event.
#[derive(Event)]
pub struct StdOutEvent {
    pub value: String,
}

/// Custom write event for errors.
#[derive(Event)]
pub struct StdErrEvent {
    pub value: String,
}

/// Marker for input display.
#[derive(Component)]
pub struct StdIn;

/// Marker for output display.
#[derive(Component)]
pub struct StdOut;

/// Marker for error display.
#[derive(Component)]
pub struct StdErr;

/// Marker for scrolling output.
#[derive(Component)]
struct ConsoleScroll;

/// Map of commands indexed by command name, implemented as Bevy systems.
#[derive(Resource)]
struct CommandMap(HashMap<String, SystemId>);

/// Console state.
#[derive(Resource)]
struct ConsoleState {
    stdin: String, // Current input buffer.
    ticker: Timer, // Flashing cursor timer.
    toggle: bool, // Flashing cursor toggle.
    style: TextStyle, // Style used for all text.
    position: f32, // Scroll position.
}

//------------------------------------------------------------------------------

/// Periodic cursor tick.
fn cursor_tick(
    time: Res<Time>,
    mut console: ResMut<ConsoleState>,
    mut query: Query<&mut Text, With<StdIn>>,
) {
    if console.ticker.tick(time.delta()).just_finished() {
        for mut text in &mut query {
            // sections = [ prompt, stdin, cursor ]
            if console.toggle { text.sections[2].value = format!("█"); }
            else { text.sections[2].value = format!(" "); }
        }
        console.toggle = !console.toggle;
    }
}

/// Startup greeter.
fn console_greeter(
    mut stdout: EventWriter<StdOutEvent>,
) {
    stdout.send(StdOutEvent { value: GREET.into() });
}

/// Create console UI and state.
fn setup_console(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // a fallout 3 style terminal
    let style = TextStyle {
        font: asset_server.load("fonts/FSEX300.ttf"), // ye olde font
        font_size: 16.0,
        color: Color::srgb_u8(41, 225, 140),
        ..default()
    };

    // console root node holds everything
    // TODO: alternatively use a flatter layout
    // i.e. remove root node and add background to stdout and stdin
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(33.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::FlexStart,
                padding: UiRect {
                    left: Val::Vw(3.0),
                    right: Val::Vw(3.0),
                    top: Val::Vw(3.0),
                    bottom: Val::Vw(3.0)
                },
                overflow: Overflow::clip_y(),
                ..default()
            },
            background_color: Color::srgb_u8(14, 46, 32).into(),
            ..default()
        },
    )).with_children(|parent| {
        // scrolling output
        parent.spawn(( // container with hidden overflow
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    // TODO: why 94%? 100% causes stdout to knock stdin down a line.
                    // why a line? coincidence that this 94% matches 2 * 3vw at my test resolution? probably
                    // doesn't make sense, especially considering inner height should be 33% - 2 * 3vw
                    // the overflow doesn't work unless it has a fixed height
                    // align_self, justify_self and flex_grow do not help
                    // putting stdin inside a NodeBundle doesn't help
                    height: Val::Percent(94.0),
                    overflow: Overflow::clip_y(),
                    ..default()
                },
                ..default()
            },
        )).with_children(|parent| {
            parent.spawn(( // content wrapper
                NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    ..default()
                },
                ConsoleScroll,
            )).with_children(|parent| {
                parent.spawn(( // content
                    TextBundle::from_section("", style.clone()),
                    StdOut, StdErr, // print output and errors
                ));
            });
        });
        // input
        parent.spawn((
            TextBundle::from_sections([
                TextSection::new(">", style.clone()),
                TextSection::new("", style.clone()),
                TextSection::new("█", style.clone()),
            ]),
            StdIn,
        ));
    });

    // build the resource
    commands.insert_resource(ConsoleState {
        stdin: String::new(),
        ticker: Timer::from_seconds(1.0, TimerMode::Repeating),
        toggle: false,
        style,
        position: 0.0,
    })
}

/// Handle keystrokes.
fn console_input(
    mut commands: Commands,
    mut keyboard_input_events: EventReader<KeyboardInput>,
    mut stdout: EventWriter<StdOutEvent>,
    mut console: ResMut<ConsoleState>,
    mut query: Query<&mut Text, With<StdIn>>,
    binaries: ResMut<CommandMap>,
) {
    for event in keyboard_input_events.read() {
        if event.state == ButtonState::Released { // ignore release events
            continue;
        }

        match &event.logical_key {
            // enter
            Key::Enter => {
                if console.stdin.is_empty() { // ignore empty buffer
                    continue;
                }

                // parse cli input
                let buffer = console.stdin.clone();
                stdout.send(StdOutEvent { value: format!(">{}\n", buffer) });
                let args: Vec<_> = buffer.split_whitespace().collect();

                // try to run a command
                match args[0] {
                    // interactive commands are implemented as Bevy systems
                    "debug" | "exit" | "load" | "system" => {
                        commands.run_system(binaries.0[args[0]]);
                    },
                    _ => shell(&mut stdout, args) // fallback to shell
                }

                // reset input buffer
                console.stdin.clear();
            },

            // delete
            Key::Backspace => {
                console.stdin.pop();
            },

            // every other key
            Key::Character(input) => {
                // ignore control chars
                if input.chars().any(|c| c.is_control()) {
                    continue;
                }
                console.stdin.push_str(&input);
            },

            _ => {}
        }
    }

    // update input ui
    for mut text in &mut query {
        text.sections[1].value = console.stdin.clone();
    }
}

//------------------------------------------------------------------------------

/// Add output to UI.
fn console_output(
    mut stdout: EventReader<StdOutEvent>,
    mut query: Query<&mut Text, With<StdOut>>,
    console: Res<ConsoleState>,
) {
    // TODO: autoscroll
    for event in stdout.read() {
        for mut text in &mut query {
            text.sections.push(TextSection::new(&event.value, console.style.clone()));
        }
    }
}

/// Add errors to UI.
fn console_error(
    mut stderr: EventReader<StdErrEvent>,
    mut query: Query<&mut Text, With<StdErr>>,
    console: Res<ConsoleState>,
) {
    for event in stderr.read() {
        for mut text in &mut query {
            text.sections.push(TextSection::new(&event.value, console.style.clone()));
        }
    }
}

/// Scroll stdout with mouse wheel.
fn console_scroll(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut console: ResMut<ConsoleState>,
    mut query_list: Query<(&mut Style, &Parent, &Node), With<ConsoleScroll>>,
    query_node: Query<&Node>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        for (mut style, parent, list_node) in &mut query_list {
            // calculate maximum scroll
            let items_height = list_node.size().y;
            let container_height = query_node.get(parent.get()).unwrap().size().y;
            let max_scroll = (items_height - container_height).max(0.0);

            // scale different types of mouse scroll
            let dy = match mouse_wheel_event.unit {
                MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
                MouseScrollUnit::Pixel => mouse_wheel_event.y,
            };

            // scroll and clamp
            console.position += dy;
            console.position = console.position.clamp(-max_scroll, 0.0);
            style.top = Val::Px(console.position);
        }
    }
}
