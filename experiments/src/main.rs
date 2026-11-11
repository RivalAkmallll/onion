
use dioxus::prelude::*;

fn main() {
    dioxus::launch(app);
}

fn app() -> Element {
    let mut counter = use_signal(|| 0);

    rsx! {
        div {
            style: "
                min-height: 100vh;
                background: #151018;
                color: #f5edf8;
                padding: 48px;
                font-family: sans-serif;
            ",

            h1 {
                style: "color: #bd82d9;",
                "Onion"
            }

            p {
                "Dioxus UI integration experiment"
            }

            p {
                "A desktop UI prototype for the future Onion UI runtime."
            }

            div {
                style: "
                    background: #281c30;
                    border-radius: 12px;
                    padding: 24px;
                    margin-top: 24px;
                ",

                h2 {
                    "Counter: {counter}"
                }

                button {
                    style: "
                        background: #8742ac;
                        border: none;
                        border-radius: 8px;
                        padding: 12px 20px;
                        color: white;
                        cursor: pointer;
                        margin-right: 12px;
                    ",
                    onclick: move |_| counter += 1,
                    "Increment"
                }

                button {
                    style: "
                        background: #483451;
                        border: none;
                        border-radius: 8px;
                        padding: 12px 20px;
                        color: white;
                        cursor: pointer;
                    ",
                    onclick: move |_| counter.set(0),
                    "Reset"
                }
            }
        }
    }
}
