use dioxus::prelude::*;

const CELLS: usize = 12;

/// Milliseconds between one cell being attended and the next.
const CHASE_STEP_MS: usize = 150;

/// A row of twelve square cells. With a percentage the row fills `fg` and the
/// next cell is attended in `blue`; without one a single `blue` cell travels
/// along it.
#[component]
pub fn Cells(progress: Option<f32>) -> Element {
    let lit = progress.map(|p| (p.clamp(0.0, 100.0) / 100.0 * CELLS as f32) as usize);

    rsx! {
        div { class: if lit.is_none() { "cells chase" } else { "cells" },
            for i in 0..CELLS {
                span {
                    class: match lit {
                        Some(n) if i < n => "cell lit",
                        Some(n) if i == n => "cell attended",
                        _ => "cell",
                    },
                    style: if lit.is_none() {
                        format!("animation-delay: -{}ms;", (CELLS - i) * CHASE_STEP_MS)
                    } else {
                        String::new()
                    },
                }
            }
        }
    }
}
