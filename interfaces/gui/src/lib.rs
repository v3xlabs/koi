use anyhow::Result;
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder},
    window::WindowBuilder,
};
use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent, menu::MenuEvent};
#[cfg(not(target_os = "linux"))]
use wry::{PageLoadEvent, WebView, WebViewBuilder};

mod icon;
mod tray;
mod webview;

pub struct GuiOptions {
    pub url: String,
}

pub enum UserEvent {
    Tray(TrayIconEvent),
    Menu(MenuEvent),
}

pub fn run(options: GuiOptions) -> Result<()> {
    configure_linux_backend();

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    tray::install_tray_event_handlers(event_loop.create_proxy());

    let window = WindowBuilder::new()
        .with_title("Koi")
        .with_inner_size(tao::dpi::LogicalSize::new(1180.0, 820.0))
        .with_window_icon(Some(icon::koi_window_icon()?))
        .build(&event_loop)?;

    let _webview = webview::build_webview(&window, &options.url)?;
    let tray = tray::Tray::new()?;

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

fn toggle_window(window: &tao::window::Window) {
    let visible = window.is_visible();
    window.set_visible(!visible);
    if visible {
        return;
    }

    let _ = window.set_focus();
}
