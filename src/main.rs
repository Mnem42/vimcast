mod config;
mod loader;
mod screenshot;
mod search;

//use crate::screenshot::take_screenshot;
use crate::loader::launch;
use crate::loader::{apps_json_path, update_apps_json_with_installed_apps};
use crate::search::RadixNode;

use dioxus::desktop::tao::platform::macos::WindowExtMacOS;
use dioxus::desktop::tao::{
    event_loop::EventLoopBuilder,
    platform::macos::{ActivationPolicy, EventLoopExtMacOS, WindowBuilderExtMacOS},
};
use dioxus::desktop::trayicon::menu::{Menu, MenuItem};
use dioxus::desktop::trayicon::{Icon, TrayIconBuilder};
use dioxus::desktop::{use_tray_menu_event_handler, Config};
use dioxus::desktop::{use_global_shortcut, window, WindowBuilder};
use dioxus::prelude::*;

const MAIN_CSS: Asset = asset!("/assets/main.css");

fn init_window() -> WindowBuilder {
    dioxus::desktop::tao::window::WindowBuilder::new()
        .with_resizable(false)
        .with_transparent(true)
        .with_decorations(false)
        .with_always_on_top(true)
        .with_has_shadow(false)
        .with_content_protection(true)
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

    let window_ = init_window();
    LaunchBuilder::new()
        .with_cfg(
            Config::new()
                .with_window(window_)
                .with_event_loop(event_loop)
                .with_disable_context_menu(true)
        )
        .launch(App);

    let main_window = window();

    #[cfg(target_os = "macos")]
    use cocoa::appkit::{NSMainMenuWindowLevel, NSWindow, NSWindowCollectionBehavior};
    use cocoa::base::id;
    let ns_win = main_window.ns_window() as id;
    unsafe {
        ns_win.setLevel_(((NSMainMenuWindowLevel + 1) as u64).try_into().unwrap());
        ns_win.setCollectionBehavior_(
            NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces,
        );
        //        ns_win.setCollectionBehavior_(
        //            NSWindowCollectionBehavior::NSWindowCollectionBehaviorMoveToActiveSpace,
        //        );
    }

    if let Some(monitor) = main_window.current_monitor() {
        let monitor_size = monitor.size(); // LogicalSize
        let monitor_position = monitor.position(); // LogicalPosition

        let window_size = main_window.outer_size(); // PhysicalSize

        // Convert everything to physical coordinates
        let center_x = monitor_position.x
            + ((monitor_size.width as f64 - window_size.width as f64) / 2.0) as i32;
        let center_y = monitor_position.y
            + ((monitor_size.height as f64 - window_size.height as f64) / 2.0) as i32;

        main_window.set_outer_position(dioxus::desktop::tao::dpi::PhysicalPosition::new(
            center_x, center_y,
        ));
    }
}

fn load_icon(bytes: &[u8]) -> Icon {
    let image = image::load_from_memory(bytes).expect("Invalid image format");
    let image = image.into_rgba8(); // Convert to RGBA8 format

    let (width, height) = image.dimensions();
    let rgba = image.into_raw(); // Raw RGBA bytes

    Icon::from_rgba(rgba, width, height).expect("Failed to create Icon")
}

#[component]
fn App() -> Element {
    let menu = Menu::new();
    let quit_item = MenuItem::with_id("quit", "Quit", true, None);
    
    let icon = load_icon(include_bytes!("../assets/icon.png"));

    menu.append(&quit_item).unwrap();

    // Create tray icon
    let builder = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("Onecast")
        .with_icon(icon);

    provide_context(builder.build().expect("tray icon builder failed"));

    use_tray_menu_event_handler(move |event| {
            // Potentially there is a better way to do this.
            // The `0` is the id of the menu item
            match event.id.0.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            }
        });

    let mut visibility = use_signal(|| 0);
    let mut db = RadixNode::new();
    crate::loader::load(&mut db);

    let mut input_value = use_signal(|| String::new());

    let main_window = window();

    #[cfg(target_os = "macos")]
    use cocoa::appkit::{NSMainMenuWindowLevel, NSWindow, NSWindowCollectionBehavior};
    use cocoa::base::id;
    let ns_win = main_window.ns_window() as id;
    unsafe {
        ns_win.setLevel_(((NSMainMenuWindowLevel + 1) as u64).try_into().unwrap());
        ns_win.setCollectionBehavior_(
            NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces,
        );
        //        ns_win.setCollectionBehavior_(
        //            NSWindowCollectionBehavior::NSWindowCollectionBehaviorMoveToActiveSpace,
        //        );
    }

    _ = use_global_shortcut("cmd+g", move || {
        visibility.set(visibility() + 1);

        if visibility() % 2 == 0 {
            let window = window();
            let is_visible = window.is_visible();

            if !is_visible {
                window.set_focus();
            }

            // Get monitor and its dimensions
            if let Some(monitor) = window.current_monitor() {
                let monitor_size = monitor.size(); // LogicalSize
                let monitor_position = monitor.position(); // LogicalPosition

                let window_size = window.outer_size(); // PhysicalSize

                // Convert everything to physical coordinates
                let center_x = monitor_position.x
                    + ((monitor_size.width as f64 - window_size.width as f64) / 2.0) as i32;
                let center_y = monitor_position.y
                    + ((monitor_size.height as f64 - window_size.height as f64) / 2.0) as i32;

                window.set_outer_position(dioxus::desktop::tao::dpi::PhysicalPosition::new(
                    center_x, center_y,
                ));
            }

            window.set_visible(!is_visible);
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
