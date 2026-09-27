// Sürüm derlemesinde konsol penceresi açılmasın.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod win;

use std::fs;
use std::path::{Path, PathBuf};

use tao::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use tao::event::{Event, StartCause, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use tao::platform::windows::IconExtWindows;
use tao::window::{Icon, Theme, Window, WindowBuilder};
use tray_icon::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use wry::{
    MemoryUsageLevel, NewWindowResponse, WebContext, WebView, WebViewBuilder,
    WebViewBuilderExtWindows, WebViewExtWindows,
};

const APP_TITLE: &str = "YT Music";
const HOME: &str = "https://music.youtube.com";

// wry'nin varsayılan argümanları + otomatik oynatma + önceden açılan yedek
// işlemciyi (spare renderer) kapatarak boşta bekleyen bir işlemden kurtuluyoruz.
const BROWSER_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection,SpareRendererForSitePerProcess \
     --autoplay-policy=no-user-gesture-required";

const INTERNAL_DOMAINS: &[&str] = &[
    "youtube.com",
    "youtu.be",
    "youtube-nocookie.com",
    "google.com",
    "gstatic.com",
    "googleusercontent.com",
    "ggpht.com",
    "googleapis.com",
    "googlevideo.com",
];

// Mac sürümündeki Bridge.js ile aynı yaklaşım: YouTube'un oynatıcı API'si, yoksa butonlar.
const JS_TOGGLE: &str = r#"(() => {
  const p = document.querySelector('#movie_player');
  const v = document.querySelector('video');
  if (p && typeof p.playVideo === 'function') { (v && !v.paused) ? p.pauseVideo() : p.playVideo(); }
  else { const b = document.querySelector('ytmusic-player-bar #play-pause-button'); if (b) b.click(); }
})()"#;
const JS_NEXT: &str = "document.querySelector('ytmusic-player-bar .next-button')?.click()";
const JS_PREVIOUS: &str = "document.querySelector('ytmusic-player-bar .previous-button')?.click()";

enum UserEvent {
    Tray(TrayIconEvent),
    Menu(MenuEvent),
    Title(String),
    OpenInApp(String),
}

fn main() {
    if !win::claim_single_instance(APP_TITLE) {
        return;
    }

    let data_dir = app_data_dir();
    let state_path = data_dir.join("window.txt");

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let window = build_window(&event_loop, &state_path);

    let proxy = event_loop.create_proxy();
    let title_proxy = proxy.clone();
    let new_window_proxy = proxy.clone();

    let mut context = WebContext::new(Some(data_dir.join("WebView2")));
    let webview = WebViewBuilder::new_with_web_context(&mut context)
        .with_url(HOME)
        .with_background_color((0, 0, 0, 255))
        .with_autoplay(true)
        .with_devtools(cfg!(debug_assertions))
        .with_hotkeys_zoom(true)
        .with_additional_browser_args(BROWSER_ARGS)
        .with_theme(wry::Theme::Dark)
        // YouTube/Google dışındaki bağlantılar varsayılan tarayıcıda açılır.
        .with_navigation_handler(|url| {
            if is_internal(&url) {
                true
            } else {
                win::open_in_browser(&url);
                false
            }
        })
        .with_new_window_req_handler(move |url, _features| {
            if is_internal(&url) {
                let _ = new_window_proxy.send_event(UserEvent::OpenInApp(url));
            } else {
                win::open_in_browser(&url);
            }
            NewWindowResponse::Deny
        })
        .with_document_title_changed_handler(move |title| {
            let _ = title_proxy.send_event(UserEvent::Title(title));
        })
        .build(&window);

    let webview = match webview {
        Ok(webview) => webview,
        Err(err) => {
            win::error_box(
                APP_TITLE,
                &format!(
                    "YT Music başlatılamadı.\n\nMicrosoft Edge WebView2 Runtime kurulu olmayabilir:\n\
                     https://go.microsoft.com/fwlink/p/?LinkId=2124703\n\nHata: {err}"
                ),
            );
            return;
        }
    };

    let tray_proxy = proxy.clone();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = tray_proxy.send_event(UserEvent::Tray(event));
    }));
    let menu_proxy = proxy;
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_proxy.send_event(UserEvent::Menu(event));
    }));

    let mut tray: Option<TrayIcon> = None;
    let mut in_background = false;

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            // Tepsi simgesi, olay döngüsü çalışmaya başladıktan sonra oluşturulmalı.
            Event::NewEvents(StartCause::Init) => tray = build_tray(),

            // Kapat = tepsiye gizle; müzik çalmaya devam eder.
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                save_window_state(&window, &state_path);
                window.set_visible(false);
                set_background(&webview, true);
                in_background = true;
            }

            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
                let minimized = window.is_minimized();
                if minimized && !in_background {
                    set_background(&webview, true);
                    in_background = true;
                } else if !minimized && window.is_visible() && in_background {
                    set_background(&webview, false);
                    in_background = false;
                }
            }

            // İkinci kez açılmaya çalışılınca pencere öne gelir; o zaman da uyanıyoruz.
            Event::WindowEvent { event: WindowEvent::Focused(true), .. } => {
                if in_background {
                    set_background(&webview, false);
                    in_background = false;
                }
            }

            Event::UserEvent(UserEvent::Tray(TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            })) => {
                if window.is_visible() && !window.is_minimized() {
                    save_window_state(&window, &state_path);
                    window.set_visible(false);
                    set_background(&webview, true);
                    in_background = true;
                } else {
                    show_window(&window, &webview);
                    in_background = false;
                }
            }

            Event::UserEvent(UserEvent::Menu(event)) => match event.id.0.as_str() {
                "show" => {
                    show_window(&window, &webview);
                    in_background = false;
                }
                "toggle" => {
                    let _ = webview.evaluate_script(JS_TOGGLE);
                }
                "next" => {
                    let _ = webview.evaluate_script(JS_NEXT);
                }
                "previous" => {
                    let _ = webview.evaluate_script(JS_PREVIOUS);
                }
                "quit" => {
                    if window.is_visible() {
                        save_window_state(&window, &state_path);
                    }
                    tray = None;
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            },

            Event::UserEvent(UserEvent::Title(title)) => {
                if let Some(tray) = &tray {
                    let _ = tray.set_tooltip(Some(tooltip_for(&title)));
                }
            }

            Event::UserEvent(UserEvent::OpenInApp(url)) => {
                let _ = webview.load_url(&url);
            }

            _ => {}
        }
    });
}

/// Arka plana geçince çizimi durdurup WebView2'den belleği bırakmasını istiyoruz.
/// Ses çalmaya devam eder; bu, uygulamanın Chrome sekmesine göre asıl kazancı.
fn set_background(webview: &WebView, background: bool) {
    let _ = webview.set_visible(!background);
    let level = if background { MemoryUsageLevel::Low } else { MemoryUsageLevel::Normal };
    let _ = webview.set_memory_usage_level(level);
}

fn show_window(window: &Window, webview: &WebView) {
    window.set_visible(true);
    window.set_minimized(false);
    window.set_focus();
    set_background(webview, false);
}

fn build_tray() -> Option<TrayIcon> {
    let menu = Menu::with_items(&[
        &MenuItem::with_id("show", "YT Music'i Göster", true, None),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("toggle", "Oynat / Duraklat", true, None),
        &MenuItem::with_id("next", "Sonraki", true, None),
        &MenuItem::with_id("previous", "Önceki", true, None),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id("quit", "Çık", true, None),
    ])
    .ok()?;

    let mut builder = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip(APP_TITLE);
    if let Ok(icon) = tray_icon::Icon::from_resource(1, None) {
        builder = builder.with_icon(icon);
    }
    builder.build().ok()
}

/// "Şarkı | YouTube Music" → "YT Music\nŞarkı" (Windows ipucu sınırı 127 karakter).
fn tooltip_for(title: &str) -> String {
    let song = title.trim().trim_end_matches("| YouTube Music").trim();
    if song.is_empty() || song == "YouTube Music" {
        return APP_TITLE.to_string();
    }
    let text = format!("{APP_TITLE}\n{song}");
    text.chars().take(120).collect()
}

fn is_internal(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")) else {
        return true; // about:blank, data: vb.
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.rsplit('@').next().unwrap_or(host);
    let host = host.split(':').next().unwrap_or(host).to_ascii_lowercase();
    if host.starts_with("accounts.google.") {
        return true;
    }
    INTERNAL_DOMAINS
        .iter()
        .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
}

fn app_data_dir() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let dir = base.join("YTMusic");
    let _ = fs::create_dir_all(&dir);
    dir
}

// MARK: - Pencere konumu ve boyutu

struct WindowState {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    maximized: bool,
}

fn build_window(event_loop: &EventLoop<UserEvent>, state_path: &Path) -> Window {
    let mut builder = WindowBuilder::new()
        .with_title(APP_TITLE)
        .with_window_icon(Icon::from_resource(1, None).ok())
        .with_inner_size(LogicalSize::new(1200.0, 800.0))
        .with_min_inner_size(LogicalSize::new(800.0, 500.0))
        .with_theme(Some(Theme::Dark))
        .with_background_color((0, 0, 0, 255));

    if let Some(state) = load_window_state(state_path) {
        builder = builder
            .with_inner_size(PhysicalSize::new(state.width, state.height))
            .with_maximized(state.maximized);
        // Kayıtlı konum artık bağlı olmayan bir ekrandaysa varsayılan konumu kullan.
        let on_screen = event_loop.available_monitors().any(|monitor| {
            let pos = monitor.position();
            let size = monitor.size();
            state.x >= pos.x - 50
                && state.y >= pos.y - 50
                && state.x < pos.x + size.width as i32 - 100
                && state.y < pos.y + size.height as i32 - 100
        });
        if on_screen {
            builder = builder.with_position(PhysicalPosition::new(state.x, state.y));
        }
    }

    builder.build(event_loop).expect("pencere oluşturulamadı")
}

fn load_window_state(path: &Path) -> Option<WindowState> {
    let text = fs::read_to_string(path).ok()?;
    let mut parts = text.split_whitespace();
    let state = WindowState {
        x: parts.next()?.parse().ok()?,
        y: parts.next()?.parse().ok()?,
        width: parts.next()?.parse().ok()?,
        height: parts.next()?.parse().ok()?,
        maximized: parts.next()? == "1",
    };
    (state.width >= 400 && state.height >= 300).then_some(state)
}

fn save_window_state(window: &Window, path: &Path) {
    if window.is_minimized() {
        return;
    }
    let maximized = window.is_maximized();
    // Büyütülmüşken boyutu değil, yalnızca büyütülmüş olduğunu kaydet.
    let previous = load_window_state(path);
    let (x, y, width, height) = match (maximized, window.outer_position(), previous) {
        (true, _, Some(prev)) => (prev.x, prev.y, prev.width, prev.height),
        (_, Ok(pos), _) => {
            let size = window.inner_size();
            (pos.x, pos.y, size.width, size.height)
        }
        _ => return,
    };
    let _ = fs::write(path, format!("{x} {y} {width} {height} {}", maximized as u8));
}
