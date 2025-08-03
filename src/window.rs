use dioxus::desktop::DesktopContext;

pub(crate) fn reposition_window(window: &DesktopContext){
    if let Some(monitor) = window.current_monitor() {
        let monitor_size = monitor.size();
        let monitor_position = monitor.position();

        let window_size = window.outer_size();

        let center_x = monitor_position.x
            + ((monitor_size.width as f64 - window_size.width as f64) / 2.0) as i32;
        let center_y = monitor_position.y
            + ((monitor_size.height as f64 - window_size.height as f64) / 2.0) as i32;

        window.set_outer_position(dioxus::desktop::tao::dpi::PhysicalPosition::new(
            center_x, center_y,
        ));
    }
}