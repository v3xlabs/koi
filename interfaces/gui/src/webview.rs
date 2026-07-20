use anyhow::Result;
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent, menu::MenuEvent};
#[cfg(not(target_os = "linux"))]
use wry::{PageLoadEvent, WebView, WebViewBuilder};

pub fn build_webview(window: &tao::window::Window, url: &str) -> Result<GuiWebView> {
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

pub enum GuiWebView {
    #[cfg(target_os = "linux")]
    WebKitGtk(()),
    #[cfg(not(target_os = "linux"))]
    Wry(WebView),
}

#[cfg(target_os = "linux")]
pub fn build_webkitgtk_webview(window: &tao::window::Window, url: &str) -> Result<GuiWebView> {
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

pub const WEBVIEW_DIAGNOSTICS: &str = r#"
console.log("Koi GUI diagnostics:", {
  href: window.location.href,
  origin: window.location.origin,
  secureContext: window.isSecureContext,
  RTCPeerConnection: typeof window.RTCPeerConnection,
  webkitRTCPeerConnection: typeof window.webkitRTCPeerConnection,
  mediaDevices: typeof navigator.mediaDevices
});
window.addEventListener("error", (event) => {
  console.error("Koi GUI JavaScript error:", event.message, event.filename, event.lineno, event.colno, event.error);
});
window.addEventListener("unhandledrejection", (event) => {
  console.error("Koi GUI unhandled promise rejection:", event.reason);
});
"#;
