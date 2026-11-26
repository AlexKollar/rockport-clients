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
//
// v1.0.2 learns to fetch its own successor (Alex: "unless we build an
// autoupdater into the next deb" — so this is the last manual install).
// On startup the client asks the release shelf, quietly, whether a
// newer signed client exists; if one does, a native dialog offers the
// update, the download is signature-verified by the updater plugin,
// and the install lands in place: the plugin's own path on Windows
// and AppImage, and — because a package install needs root that the
// plugin does not arrange — pkexec on package installs (dpkg on the
// Debian family, rpm on the Fedora/SUSE family), with the system's
// own password prompt. The city is never involved; only the door
// changes.
//
// v1.0.3 puts an RPM on the shelf beside the .deb and the AppImage
// (Alex: "we also need to add RPM builds for Linux"), and the updater
// covers it the same way: the manifest carries the rpm artifact, and
// the install step picks the package tool the system actually runs.

use tauri::webview::NewWindowResponse;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_updater::UpdaterExt;

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

async fn check_and_offer(handle: AppHandle) {
    let updater = match handle.updater() {
        Ok(u) => u,
        Err(_) => return,
    };
    let update = match updater.check().await {
        Ok(Some(u)) => u,
        _ => return,
    };
    let version = update.version.clone();
    let h = handle.clone();
    handle
        .dialog()
        .message(format!(
            "Rockport {} is ready — a newer client than the one you are \
             running. Update and restart now? The city itself won't \
             blink; only the door changes.",
            version
        ))
        .title("Rockport — update available")
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Update and restart".to_string(),
            "Later".to_string(),
        ))
        .show(move |confirmed| {
            if !confirmed {
                return;
            }
            let h2 = h.clone();
            tauri::async_runtime::spawn(async move {
                let bytes = match update.download(|_, _| {}, || {}).await {
                    Ok(b) => b,
                    Err(_) => return,
                };
                install_downloaded(&h2, &update, bytes);
            });
        });
}

/// The Arch line (Omarchy and kin) runs pacman, and Rockport
/// ships there as a package — updates arrive as packages too.
/// Detected by the tool itself: pacman present, and neither of
/// the other lines' tools beside it.
#[cfg(target_os = "linux")]
fn pacman_system() -> bool {
    std::path::Path::new("/usr/bin/pacman").exists()
        && !std::path::Path::new("/usr/bin/dpkg").exists()
        && !std::path::Path::new("/usr/bin/rpm").exists()
}

#[cfg(target_os = "linux")]
fn package_update_dialog(handle: &AppHandle) {
    handle
        .dialog()
        .message(
            "On this system, Rockport updates arrive as a package: \
             the newest Arch package (.pkg.tar.zst) is waiting at the \
             city's front door, and pacman -U installs it in one line.",
        )
        .title("Rockport — update via package")
        .buttons(MessageDialogButtons::Ok)
        .show(|_| {});
}

fn install_downloaded(
    handle: &AppHandle,
    update: &tauri_plugin_updater::Update,
    bytes: Vec<u8>,
) {
    #[cfg(not(target_os = "linux"))]
    {
        if update.install(bytes).is_ok() {
            handle.restart();
        }
    }
    #[cfg(target_os = "linux")]
    {
        // An AppImage replaces its own file — the plugin's path works
        // there. A .deb needs root for dpkg, which the plugin does
        // not arrange; pkexec does, with the system's own prompt.
        if std::env::var_os("APPIMAGE").is_some() {
            if update.install(bytes).is_ok() {
                handle.restart();
            } else if pacman_system() {
                // The packaged AppImage lives on root-owned ground;
                // the swap cannot land there. On the Arch line the
                // package is the update — say so, plainly.
                package_update_dialog(handle);
            }
            return;
        }
        // A bare binary on the Arch line: the downloaded bytes are
        // the AppImage build, which neither dpkg nor rpm can eat.
        // Do not feed them to the wrong tool.
        if pacman_system() {
            package_update_dialog(handle);
            return;
        }
        // The package tool follows the system family: rpm where the
        // os-release names the Fedora/SUSE line (or where dpkg simply
        // isn't), dpkg everywhere else.
        let osr = std::fs::read_to_string("/etc/os-release")
            .unwrap_or_default()
            .to_lowercase();
        let rpm_family = ["fedora", "rhel", "centos", "rocky", "almalinux", "suse"]
            .iter()
            .any(|k| osr.contains(k))
            || (std::path::Path::new("/usr/bin/rpm").exists()
                && !std::path::Path::new("/usr/bin/dpkg").exists());
        let (tool, flag, ext): (&str, &str, &str) = if rpm_family {
            ("rpm", "-U", "rpm")
        } else {
            ("dpkg", "-i", "deb")
        };
        let path = std::env::temp_dir()
            .join(format!("rockport-update-{}.{}", update.version, ext));
        if std::fs::write(&path, &bytes).is_err() {
            return;
        }
        let mut cmd = std::process::Command::new("pkexec");
        cmd.arg(tool).arg(flag);
        if rpm_family {
            cmd.arg("--replacepkgs");
        }
        let status = cmd.arg(&path).status();
        match status {
            Ok(s) if s.success() => handle.restart(),
            _ => {
                handle
                    .dialog()
                    .message(format!(
                        "The update downloaded, but the system installer \
                         didn't finish — the new package is waiting at {}. \
                         You can install it by hand, or grab the newest \
                         client from the city's front door.",
                        path.display()
                    ))
                    .title("Rockport — update paused")
                    .buttons(MessageDialogButtons::Ok)
                    .show(|_| {});
            }
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
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
            // The update check rides a plain thread for its delay so
            // startup never waits on it; the check itself is async.
            let h = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(5));
                tauri::async_runtime::block_on(check_and_offer(h));
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Rockport");
}
