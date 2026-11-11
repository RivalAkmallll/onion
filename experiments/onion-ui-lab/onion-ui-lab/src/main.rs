use dioxus::prelude::*;

const STYLES: &str = include_str!("../assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
#[allow(non_snake_case)]
fn App() -> Element {
    let mut page = use_signal(|| "Overview".to_string());
    let mut counter = use_signal(|| 0_i32);
    let mut dark_mode = use_signal(|| true);
    let selected_page = page();
    let is_dark = dark_mode();

    rsx! {
        document::Style { "{STYLES}" }
        div { class: if is_dark { "shell dark" } else { "shell light" },
            aside { class: "sidebar",
                div { class: "brand",
                    div { class: "brand-mark", "O" }
                    div {
                        strong { "Onion UI" }
                        span { "Developer preview" }
                    }
                }
                nav { aria_label: "Main navigation",
                    for item in ["Overview", "Projects", "Components", "Settings"] {
                        button {
                            key: "{item}",
                            class: if selected_page == item { "nav-item active" } else { "nav-item" },
                            onclick: move |_| page.set(item.to_string()),
                            "{item}"
                        }
                    }
                }
                div { class: "sidebar-bottom", "Preview build • Dioxus 0.7" }
            }
            main { class: "content",
                header { class: "topbar",
                    span { class: "eyebrow", "WORKSPACE / {selected_page}" }
                    button {
                        class: "theme-toggle",
                        onclick: move |_| dark_mode.set(!is_dark),
                        if is_dark { "☀ Light mode" } else { "☾ Dark mode" }
                    }
                }
                section { class: "hero",
                    div {
                        span { class: "pill", "ONION DESIGN SYSTEM" }
                        h1 { "Build something beautiful." }
                        p { "A responsive, editable UI experiment for the future Onion language." }
                        div { class: "hero-actions",
                            button {
                                class: "primary-btn",
                                onclick: move |_| counter += 1,
                                "Try interaction  ↗"
                            }
                            a { class: "secondary-btn", href: "https://dioxuslabs.com/learn/0.7/", "Dioxus docs" }
                        }
                    }
                    div { class: "hero-orb", "🧅" }
                }
                section { class: "stats-grid", aria_label: "Example dashboard metrics",
                    article { class: "stat-card",
                        span { class: "stat-label", "UI actions" }
                        strong { "{counter}" }
                        span { class: "stat-foot", "Local interactive counter" }
                    }
                    article { class: "stat-card",
                        span { class: "stat-label", "Components" }
                        strong { "04" }
                        span { class: "stat-foot", "Reusable visual patterns" }
                    }
                    article { class: "stat-card",
                        span { class: "stat-label", "Theme" }
                        strong { if is_dark { "Dark" } else { "Light" } }
                        span { class: "stat-foot", "Switch themes instantly" }
                    }
                }
                section { class: "panels",
                    article { class: "panel",
                        h2 { "Your canvas" }
                        p { "Start with a layout, replace components, and design with standard CSS." }
                        div { class: "preview-list",
                            div { span { class: "dot green" } "Responsive layout" span { "Ready" } }
                            div { span { class: "dot purple" } "State management" span { "Active" } }
                            div { span { class: "dot blue" } "Theme tokens" span { "Enabled" } }
                        }
                    }
                    article { class: "panel side-panel",
                        h2 { "Quick actions" }
                        p { "This demo is Rust + Dioxus, not native Onion syntax yet." }
                        button { class: "outline-btn", onclick: move |_| counter.set(0), "Reset counter" }
                    }
                }
                footer { "Onion UI Lab • Experimental design preview" }
            }
        }
    }
}
