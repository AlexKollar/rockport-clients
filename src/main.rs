#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Rockport — the standalone client.
//
// One native window, no browser chrome: the webview opens the city's own
// homepage (with the web player inside it) first-party, exactly as served —
// the game stays single-sourced on the server, so every deploy is instantly
// this client's content. No local frontend: the client is a door, the city
// is on the other side of it.
//
// The window is built in code (not in tauri.conf.json) for the sake of
// the admin wing. In v1.0.0 the shell Admin pane's links — Builder's
// Desk, Django admin, characters, changelog, all target=_blank — died
// in the client: a bare webview has no second window to give them.
// Alex's ruling (2026-11-25): pull the admin pages INTO the app. So
// now a new-window ask for a city page becomes this window's own
// journey instead, and an initialization script rides every page to
// (a) turn target=_blank / window.open city links into same-window
// navigation at the source, and (b) furnish the admin pages with
// client chrome: a way back to the city, and an Admin dropdown —
// the dashboard and its menu, inside the app. Links that leave the
// city entirely still belong to the real browser. (v1.0.1.)

use tauri::webview::NewWindowResponse;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

const CITY: &str = "https://rockport.nocturnerow.com:4043/";
const CITY_HOST: &str = "rockport.nocturnerow.com";

const INIT_SCRIPT: &str = r##"
(function () {
  if (window.__rpClient) return;
  window.__rpClient = true;
  var ORIGIN = "https://rockport.nocturnerow.com:4043";
  function cityUrl(u) {
    try {
      var x = new URL(u, window.location.href);
      return x.origin === ORIGIN ? x.href : null;
    } catch (e) { return null; }
  }
  document.addEventListener("click", function (e) {
    var a = e.target && e.target.closest ? e.target.closest("a") : null;
    if (!a) return;
    if ((a.getAttribute("target") || "").toLowerCase() !== "_blank") return;
    var href = cityUrl(a.getAttribute("href") || "");
    if (href) { e.preventDefault(); window.location.href = href; }
  }, true);
  var realOpen = window.open;
  window.open = function (u) {
    var href = u ? cityUrl(u) : null;
    if (href) { window.location.href = href; return null; }
    return realOpen ? realOpen.apply(window, arguments) : null;
  };
  var path = window.location.pathname || "/";
  var ADMIN = /^\/desk(\/|$)|\/admin(\/|$)|\/characters(\/|$)|\/changelog(\/|$)/;
  if (!ADMIN.test(path)) return;
  function chrome() {
    if (document.getElementById("rp-client-bar")) return;
    var css = document.createElement("style");
    css.textContent = "body{padding-top:46px!important}"
      + "#rp-client-bar{position:fixed;top:0;left:0;right:0;height:46px;z-index:2147483000;display:flex;align-items:center;gap:12px;padding:0 12px;background:#05080a;border-bottom:1px solid #12333d;font:13px/1.4 system-ui,sans-serif;color:#cfe}"
      + "#rp-client-bar a{color:#39d5e8;text-decoration:none}"
      + "#rp-client-bar .rp-back{font-weight:600}"
      + "#rp-client-bar .rp-title{opacity:.55;letter-spacing:.14em;font-size:11px}"
      + "#rp-client-menu{margin-left:auto;position:relative}"
      + "#rp-client-menu button{background:#0a161c;color:#cfe;border:1px solid #12333d;border-radius:8px;padding:6px 12px;cursor:pointer;font:inherit}"
      + "#rp-client-list{display:none;position:absolute;right:0;top:40px;min-width:220px;background:#07131a;border:1px solid #12333d;border-radius:10px;padding:6px;box-shadow:0 12px 32px rgba(0,0,0,.5)}"
      + "#rp-client-list a{display:block;padding:7px 10px;border-radius:6px;color:#cfe}"
      + "#rp-client-list a:hover{background:#0d212b}"
      + "#rp-client-menu.open #rp-client-list{display:block}";
    document.head.appendChild(css);
    var bar = document.createElement("div");
    bar.id = "rp-client-bar";
    bar.innerHTML = "<a class=\"rp-back\" href=\"#\" id=\"rp-client-back\">&larr; Back to the city</a>"
      + "<span class=\"rp-title\">ROCKPORT &middot; ADMIN</span>"
      + "<div id=\"rp-client-menu\"><button type=\"button\" id=\"rp-client-btn\">Admin &#9662;</button>"
      + "<div id=\"rp-client-list\">"
      + "<a href=\"/desk/\">Builder's Desk — the board</a>"
      + "<a href=\"/desk/map/\">Desk — city map</a>"
      + "<a href=\"/desk/economy/\">Desk — economy dials</a>"
      + "<a href=\"/desk/players/\">Desk — players</a>"
      + "<a href=\"/desk/audit/\">Desk — audit trail</a>"
      + "<a href=\"/admin/\">Django admin — the deep levers</a>"
      + "<a href=\"/characters/\">Character list</a>"
      + "<a href=\"/changelog/\">City changelog</a>"
      + "<a href=\"/webclient/\">The player</a>"
      + "<a href=\"/\">City homepage</a>"
      + "</div></div>";
    document.body.appendChild(bar);
    document.getElementById("rp-client-back").addEventListener("click", function (e) {
      e.preventDefault();
      if (window.history.length > 1) { window.history.back(); }
      else { window.location.href = ORIGIN + "/webclient/"; }
    });
    var menu = document.getElementById("rp-client-menu");
    document.getElementById("rp-client-btn").addEventListener("click", function (e) {
      e.stopPropagation(); menu.classList.toggle("open");
    });
    document.addEventListener("click", function () { menu.classList.remove("open"); });
  }
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", chrome);
  } else { chrome(); }
})();
"##;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let new_handle = app.handle().clone();
            WebviewWindowBuilder::new(
                app,
                "main",
                WebviewUrl::External(CITY.parse().expect("city URL parses")),
            )
            .title("Rockport")
            .inner_size(1280.0, 800.0)
            .min_inner_size(900.0, 600.0)
            .resizable(true)
            .center()
            .focused(true)
            .initialization_script(INIT_SCRIPT)
            .on_new_window(move |url, _features| {
                if url.host_str() == Some(CITY_HOST) {
                    // A city page asked for a new window: it gets
                    // this one. The admin wing lives in the app.
                    if let Some(w) = new_handle.get_webview_window("main") {
                        let _ = w.navigate(url);
                    }
                } else {
                    let _ = open::that(url.as_str());
                }
                NewWindowResponse::Deny
            })
            .on_navigation(move |url| {
                if url.host_str() == Some(CITY_HOST) {
                    true
                } else {
                    let _ = open::that(url.as_str());
                    false
                }
            })
            .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Rockport");
}
