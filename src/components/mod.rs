//! Declarative Dioxus component facade for canvas presentation.

use dioxus::prelude::*;

/// Properties for the `<TrioxusCanvas>` component.
#[derive(Clone, PartialEq, Props)]
pub struct TrioxusCanvasProps {
    #[props(default)]
    pub id: String,
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub style: String,
}

/// Declarative cross-platform graphics canvas component for Dioxus applications.
#[component]
pub fn TrioxusCanvas(props: TrioxusCanvasProps) -> Element {
    #[cfg(target_arch = "wasm32")]
    {
        rsx! {
            canvas {
                id: "{props.id}",
                class: "{props.class}",
                style: "{props.style}",
                tabindex: "0",
                "data-tricore-viewport": "active",
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        rsx! {
            div {
                id: "{props.id}",
                class: "{props.class}",
                style: "{props.style}",
                "data-tricore-viewport": "active",
            }
        }
    }
}
