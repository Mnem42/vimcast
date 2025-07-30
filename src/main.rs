mod config;
mod loader;
mod search;

use crate::loader::launch;
use crate::loader::{apps_json_path, update_apps_json_with_installed_apps};
use crate::search::RadixNode;

use dioxus::desktop::tao::{
    dpi::LogicalPosition,
    event_loop::{EventLoop, EventLoopBuilder},
    platform::macos::{ActivationPolicy, EventLoopExtMacOS, WindowBuilderExtMacOS},
    window::Window,
};
use dioxus::desktop::{use_global_shortcut, window, Config, WindowBuilder};
use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");

#[allow(unused)]
fn move_window_to_monitor(window: &Window, event_loop: &EventLoop<()>, monitor_index: usize) {
    let monitors: Vec<_> = event_loop.available_monitors().collect();

    if let Some(monitor) = monitors.get(monitor_index) {
        let position = monitor.position(); // PhysicalPosition<i32>
        let scale_factor = window.scale_factor();

        // Convert to logical position based on window's scale factor
        let logical_position: LogicalPosition<f64> = position.to_logical(scale_factor);
        window.set_outer_position(logical_position);
    } else {
        eprintln!("Monitor index {} not found!", monitor_index);
    }
}

fn init_window() -> WindowBuilder {
    WindowBuilder::new()
        .with_resizable(false)
        .with_transparent(true)
        .with_decorations(false)
        .with_always_on_top(true)
        .with_movable_by_window_background(false)
        .with_visible_on_all_workspaces(true)
        .with_has_shadow(false)
        .with_content_protection(false)
}
fn main() {
    if let Err(e) = config::initialize_config() {
        eprintln!("⚠️ failed to initialize config: {e}");
    }

    if let Err(e) = config::try_update_apps_json() {
        eprintln!("⚠️ failed to update apps.json: {e}");
    }

    let mut event_loop = EventLoopBuilder::with_user_event().build();

    event_loop.set_activation_policy(ActivationPolicy::Accessory);

    let window = init_window();

    let monitor = window
        .clone()
        .build(&event_loop)
        .expect("Error getting monitor");

    let mon = monitor.current_monitor();

    println!("Monitor ID: {:#?}", monitor.id());

    LaunchBuilder::new()
        .with_cfg(
            Config::new()
                .with_window(window.with_position(mon.as_ref().unwrap().position()))
                .with_event_loop(event_loop)
                .with_disable_context_menu(true),
        )
        .launch(App);
}

#[component]
fn App() -> Element {
    let mut visibility = use_signal(|| 0);
    let mut db = RadixNode::new();
    crate::loader::load(&mut db);

    let mut input_value = use_signal(|| String::new());

    _ = use_global_shortcut("cmd+g", move || {
        visibility.set(visibility() + 1);
        if visibility() % 2 == 0 {
            let visibility = window().is_visible();

            if !visibility {
                window().set_focus();
            }

            window().set_visible(!visibility);
            window().set_outer_position(window().current_monitor().unwrap().position());
        }
    });

    rsx! {
        div { id: "container",
            document::Link { rel: "stylesheet", href: MAIN_CSS }

            input {
                id: "main-input",
                autocomplete: false,
                autocapitalize: false,
                placeholder: "Search for apps and commands",
                value: "{input_value}",
                oninput: move |event| input_value.set(event.value()),


            }
            SResults { query: input_value(), db }
        }
    }
}

#[component]
fn SResults(query: String, db: RadixNode) -> Element {
    let searchresults = if db.starts_with(&query) && !query.is_empty() {
        db.collect(&query.to_lowercase().trim().trim_start())
    } else {
        vec![]
    };

    let render_commands = searchresults.into_iter().map(|command| {
        let command_clone = command.clone(); // move into closure
        rsx! {
            button {
                onclick: move |_| {
                    launch(command_clone.clone());
                    window().set_visible(false);
                },
                class: "command-div",
                "{command_clone}"
            }
        }
    });

    rsx! {
        div { id: "results-container", {render_commands} }
    }
}

#[component]
fn Settings() -> Element {
    rsx! {
        div { id: "settings-container",
            h1 { "Vimcast Settings" }
            p { "Placeholder for preferences UI" }
        }
    }
}
