use dioxus::prelude::*;

#[component]
pub fn Settings() -> Element {
    let mut count = use_signal(|| 0);
    rsx! {
        div {
            h1 { "Popup Window" }
            p { "Count: {count}" }
            button { onclick: move |_| count += 1, "Increment" }
        }
    }
}
