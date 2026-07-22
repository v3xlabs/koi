use super::UserEvent;
use anyhow::Result;
use std::io::Cursor;
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
};
use tray_icon::{
    TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuId, MenuItem, PredefinedMenuItem},
};
#[cfg(not(target_os = "linux"))]
use wry::{PageLoadEvent, WebView, WebViewBuilder};

pub struct Tray {
    pub show_id: MenuId,
    pub quit_id: MenuId,
    _icon: tray_icon::TrayIcon,
}

impl Tray {
    pub fn new() -> Result<Self> {
        let menu = Menu::new();
        let show = MenuItem::with_id("show", "Show Koi", true, None);
        let quit = MenuItem::with_id("quit", "Quit Koi", true, None);
        menu.append_items(&[&show, &PredefinedMenuItem::separator(), &quit])?;

        let icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Koi")
            .with_icon(super::icon::koi_tray_icon()?)
            .build()?;

        Ok(Self {
            show_id: show.id().clone(),
            quit_id: quit.id().clone(),
            _icon: icon,
        })
    }
}

pub fn install_tray_event_handlers(proxy: EventLoopProxy<UserEvent>) {
    let tray_proxy = proxy.clone();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = tray_proxy.send_event(UserEvent::Tray(event));
    }));

    MenuEvent::set_event_handler(Some(move |event| {
        let _ = proxy.send_event(UserEvent::Menu(event));
    }));
}
