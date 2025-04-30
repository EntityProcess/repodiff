# Product Requirements Document (PRD) for RepoDiff UI (Dioxus)

## 1. Overview
This PRD outlines the requirements for a **modern, dark-mode enabled UI** for RepoDiff, built using **Dioxus and Rust**. The UI should provide an intuitive interface for users to configure diff settings, select commits, and generate a structured diff file for LLM consumption.

## 2. Key Objectives
1. **Provide an intuitive UI** for configuring file filters, diff settings, and context rules.
2. **Support dark mode** with modern aesthetics using Dioxus's styling capabilities.
3. **Enable commit selection** from a dropdown or interactive selection.
4. **Allow easy export** of the processed diff to a file.
5. **Ensure responsiveness** and usability across different screen sizes.
6. **Keep UI simple** without embedding a diff viewer, as users will edit the output in their own text editor.

## 3. UI Design
### 3.1. Main Window Components
1. **Commit Selection Panel**
   ```rust
   rsx! {
       select { 
           name: "commit1",
           option { value: "HEAD", "Current" }
           option { value: "HEAD~1", "Previous" }
       }
   }
   ```
2. **Configuration Panel**
   ```rust
   rsx! {
       div { class: "config-panel",
           input { 
               r#type: "text",
               placeholder: "File pattern (e.g., *.cs)",
           }
           input {
               r#type: "number",
               placeholder: "Context lines",
           }
           toggle {
               label: "Include signatures",
           }
       }
   }
   ```
3. **Export Options**
   ```rust
   rsx! {
       button {
           onclick: move |_| export_diff(),
           "Pack"
       }
   }
   ```
4. **Dark Mode Toggle**
   ```rust
   rsx! {
       button {
           onclick: move |_| toggle_theme(),
           class: "theme-toggle",
           "Toggle Theme"
       }
   }
   ```

## 4. Dark Mode Implementation
- Use **Dioxus's CSS-in-Rust** capabilities for theming:
  ```rust
  let theme = use_state(cx, || Theme::Dark);
  
  cx.render(rsx! {
      div {
          class: format!("app-container {}", theme.get()),
          // UI components
      }
  })
  ```
- Implement system theme detection using Rust's system APIs
- Persist theme preference using `serde` for configuration storage

## 5. Interaction Flow
1. **State Management**
   ```rust
   #[derive(Props)]
   struct AppState {
       commit1: String,
       commit2: String,
       file_pattern: String,
       context_lines: u32,
       include_signatures: bool,
   }
   ```

2. **Event Handlers**
   ```rust
   fn handle_pack(state: &AppState) -> Result<(), RepoDiffError> {
       let diff = generate_diff(&state.commit1, &state.commit2)?;
       export_to_file(&diff, "output.md")?;
       Ok(())
   }
   ```

## 6. Sample Implementation
```rust
use dioxus::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Config {
    file_pattern: String,
    context_lines: u32,
    include_signatures: bool,
}

fn App(cx: Scope) -> Element {
    let config = use_state(cx, || Config::default());
    
    cx.render(rsx! {
        div { class: "app-container",
            header {
                h1 { "RepoDiff" }
            }
            
            div { class: "commit-selection",
                CommitSelector {}
            }
            
            div { class: "config-panel",
                ConfigurationPanel {
                    config: config
                }
            }
            
            button {
                class: "pack-button",
                onclick: move |_| handle_pack(&config),
                "Pack"
            }
        }
    })
}

fn main() {
    dioxus_desktop::launch(App);
}
```

## 7. Expected Outcome
- A modern UI with a **clean, structured layout** using Rust's type safety
- **Dark mode support** with CSS-in-Rust styling
- **No embedded diff viewer**, only file export
- **Seamless interaction** for configuring and generating diffs
- **Type-safe state management** using Rust's strong type system
- **Cross-platform compatibility** through Dioxus's desktop support

## 8. Technical Requirements
- **Dependencies**
  ```toml
  [dependencies]
  dioxus = "0.4"
  dioxus-desktop = "0.4"
  serde = { version = "1.0", features = ["derive"] }
  serde_json = "1.0"
  thiserror = "1.0"
  ```
- **Build Requirements**
  - Rust 1.75 or later
  - Cargo package manager
  - Desktop development prerequisites for target platform

## 9. Summary
RepoDiff's UI will be developed using **Dioxus and Rust**, providing a **modern, user-friendly interface** with dark mode support. The UI will leverage Rust's strong type system and Dioxus's reactive components to create a reliable and efficient tool for code review workflows. The implementation will focus on maintainability, type safety, and cross-platform compatibility while keeping the interface simple and focused on its core functionality.

