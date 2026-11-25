#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Rockport — the standalone client.
//
// One native window, no browser chrome: the webview opens the city's own
// homepage (with the web player inside it) first-party, exactly as served —
// the game stays single-sourced on the server, so every deploy is instantly
// this client's content. No local frontend, no Tauri API surface for the
// page: the client is a door, the city is on the other side of it.

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running Rockport");
}
