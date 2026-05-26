use anyhow::Result;
use std::io::Cursor;
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::{Icon as WindowIcon, WindowBuilder},
};
use tray_icon::{
    Icon as TrayIconImage, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem},
};
#[cfg(not(target_os = "linux"))]
use wry::{PageLoadEvent, WebView, WebViewBuilder};

const KOI_ICON_PNG: &[u8] = include_bytes!("../../web/public/favicon_x64.png");

pub struct GuiOptions {
    pub url: String,
}

enum UserEvent {
    Tray(TrayIconEvent),
    Menu(MenuEvent),
}

pub fn run(options: GuiOptions) -> Result<()> {
    configure_linux_backend();

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    install_tray_event_handlers(event_loop.create_proxy());

    let window = WindowBuilder::new()
        .with_title("Koi")
        .with_inner_size(tao::dpi::LogicalSize::new(1180.0, 820.0))
        .with_window_icon(Some(koi_window_icon()?))
        .build(&event_loop)?;

    let _webview = build_webview(&window, &options.url)?;
    let tray = Tray::new()?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                window.set_visible(true);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                window.set_visible(false);
            }
            Event::UserEvent(UserEvent::Tray(TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            })) => {
                toggle_window(&window);
            }
            Event::UserEvent(UserEvent::Menu(event)) if event.id == tray.show_id => {
                window.set_visible(true);
                let _ = window.set_focus();
            }
            Event::UserEvent(UserEvent::Menu(event)) if event.id == tray.quit_id => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn build_webview(window: &tao::window::Window, url: &str) -> Result<GuiWebView> {
    #[cfg(target_os = "linux")]
    {
        build_webkitgtk_webview(window, url)
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok(GuiWebView::Wry(
            WebViewBuilder::new()
                .with_initialization_script(WEBVIEW_DIAGNOSTICS)
                .with_on_page_load_handler(|event, url| match event {
                    PageLoadEvent::Started => eprintln!("Koi GUI loading {url}"),
                    PageLoadEvent::Finished => eprintln!("Koi GUI loaded {url}"),
                })
                .with_url(url)
                .build(window)?,
        ))
    }
}

enum GuiWebView {
    #[cfg(target_os = "linux")]
    WebKitGtk(()),
    #[cfg(not(target_os = "linux"))]
    Wry(WebView),
}

#[cfg(target_os = "linux")]
fn build_webkitgtk_webview(window: &tao::window::Window, url: &str) -> Result<GuiWebView> {
    use gtk::prelude::*;
    use tao::platform::unix::WindowExtUnix;
    use webkit2gtk::{
        AutoplayPolicy, PermissionRequestExt, Settings, SettingsExt, UserContentInjectedFrames,
        UserContentManager, UserContentManagerExt, UserScript, UserScriptInjectionTime, WebViewExt,
        WebsitePolicies,
    };

    let container = window
        .default_vbox()
        .ok_or_else(|| anyhow::anyhow!("missing GTK container for Koi GUI webview"))?;

    let settings = Settings::builder()
        .enable_media(true)
        .enable_media_capabilities(true)
        .enable_media_stream(true)
        .enable_mediasource(true)
        .enable_webaudio(true)
        .enable_webgl(true)
        .enable_webrtc(true)
        .enable_developer_extras(cfg!(debug_assertions))
        .build();
    eprintln!(
        "Koi GUI WebKit settings: webrtc={} media_stream={} media={}",
        settings.enables_webrtc(),
        settings.enables_media_stream(),
        settings.enables_media()
    );

    let user_content = UserContentManager::new();
    user_content.add_script(&UserScript::new(
        WEBVIEW_DIAGNOSTICS,
        UserContentInjectedFrames::TopFrame,
        UserScriptInjectionTime::Start,
        &[],
        &[],
    ));

    let webview = webkit2gtk::WebView::builder()
        .settings(&settings)
        .user_content_manager(&user_content)
        .website_policies(
            &WebsitePolicies::builder()
                .autoplay(AutoplayPolicy::Allow)
                .build(),
        )
        .build();

    webview.connect_load_changed(|_, event| {
        eprintln!("Koi GUI WebKitGTK load event: {event:?}");
    });

    webview.connect_permission_request(|_, request| {
        eprintln!("Koi GUI allowing WebKit permission request");
        request.allow();
        true
    });

    webview.load_uri(url);
    webview.show_all();
    container.pack_start(&webview, true, true, 0);

    Ok(GuiWebView::WebKitGtk(()))
}

const WEBVIEW_DIAGNOSTICS: &str = r#"
console.log("Koi GUI diagnostics:", {
  href: window.location.href,
  origin: window.location.origin,
  secureContext: window.isSecureContext,
  RTCPeerConnection: typeof window.RTCPeerConnection,
  webkitRTCPeerConnection: typeof window.webkitRTCPeerConnection,
  mediaDevices: typeof navigator.mediaDevices
});
window.addEventListener("error", (event) => {
  document.body.innerHTML = `<pre style="white-space: pre-wrap; padding: 24px; color: #ffb4b4; background: #111; min-height: 100vh;">Koi GUI JavaScript error:
${event.message}
${event.filename}:${event.lineno}:${event.colno}</pre>`;
});
window.addEventListener("unhandledrejection", (event) => {
  document.body.innerHTML = `<pre style="white-space: pre-wrap; padding: 24px; color: #ffb4b4; background: #111; min-height: 100vh;">Koi GUI unhandled promise rejection:
${event.reason?.stack ?? event.reason}</pre>`;
});
"#;

fn configure_linux_backend() {
    #[cfg(target_os = "linux")]
    if std::env::var_os("DISPLAY").is_some() {
        // Wry's WebKitGTK backend currently attaches through X11 handles.
        // Force Tao and GTK onto XWayland before either toolkit initializes.
        unsafe {
            std::env::set_var("WINIT_UNIX_BACKEND", "x11");
            std::env::set_var("GDK_BACKEND", "x11");
        }
    }
}

struct Tray {
    show_id: MenuId,
    quit_id: MenuId,
    _icon: tray_icon::TrayIcon,
}

impl Tray {
    fn new() -> Result<Self> {
        let menu = Menu::new();
        let show = MenuItem::with_id("show", "Show Koi", true, None);
        let quit = MenuItem::with_id("quit", "Quit Koi", true, None);
        menu.append_items(&[&show, &PredefinedMenuItem::separator(), &quit])?;

        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Koi")
            .with_icon(koi_tray_icon()?)
            .build()?;

        Ok(Self {
            show_id: show.id().clone(),
            quit_id: quit.id().clone(),
            _icon: icon,
        })
    }
}

fn install_tray_event_handlers(proxy: EventLoopProxy<UserEvent>) {
    let tray_proxy = proxy.clone();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = tray_proxy.send_event(UserEvent::Tray(event));
    }));

    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::Menu(event));
    }));
}

fn toggle_window(window: &tao::window::Window) {
    let visible = window.is_visible();
    window.set_visible(!visible);
    if visible {
        return;
    }

    let _ = window.set_focus();
}

fn koi_window_icon() -> Result<WindowIcon> {
    let (rgba, width, height) = load_koi_icon_rgba()?;
    Ok(WindowIcon::from_rgba(rgba, width, height)?)
}

fn koi_tray_icon() -> Result<TrayIconImage> {
    let (rgba, width, height) = load_koi_icon_rgba()?;
    Ok(TrayIconImage::from_rgba(rgba, width, height)?)
}

fn load_koi_icon_rgba() -> Result<(Vec<u8>, u32, u32)> {
    let decoder = png::Decoder::new(Cursor::new(KOI_ICON_PNG));
    let mut reader = decoder.read_info()?;
    let output_buffer_size = reader
        .output_buffer_size()
        .ok_or_else(|| anyhow::anyhow!("invalid Koi icon dimensions"))?;
    let mut buffer = vec![0; output_buffer_size];
    let info = reader.next_frame(&mut buffer)?;
    let bytes = &buffer[..info.buffer_size()];

    let rgba = match info.color_type {
        png::ColorType::Rgba => bytes.to_vec(),
        png::ColorType::Rgb => bytes
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 0xff])
            .collect(),
        other => anyhow::bail!("unsupported Koi icon color type: {other:?}"),
    };

    Ok((rgba, info.width, info.height))
}
