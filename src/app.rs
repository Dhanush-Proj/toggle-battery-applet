// SPDX-License-Identifier: MPL-2.0

use cosmic::iced::{window::Id, Limits, Subscription};
use cosmic::iced_winit::commands::popup::{destroy_popup, get_popup};
use cosmic::prelude::*;
use cosmic::widget;
use std::fs;
use std::process::Command;

// Path specific to Lenovo IdeaPad 3 this is my pc you have to change it according to device
const BAT_PATH: &str = "/sys/bus/platform/drivers/ideapad_acpi/VPC2004:00/conservation_mode";

pub struct AppModel {
    core: cosmic::Core,
    popup: Option<Id>,
    conservation_enabled: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    ToggleConservation(bool),
}

impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.github.user.battery-toggle";

    fn core(&self) -> &cosmic::Core { &self.core }
    fn core_mut(&mut self) -> &mut cosmic::Core { &mut self.core }

    fn init(core: cosmic::Core, _flags: Self::Flags) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Read initial state from system hardware
        let initial_state = fs::read_to_string(BAT_PATH)
            .map(|s| s.trim() == "1")
            .unwrap_or(false);

        let app = AppModel {
            core,
            popup: None,
            conservation_enabled: initial_state,
        };

        (app, Task::none())
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    // This is the icon visible on the panel
    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("battery-level-80-symbolic")
            .on_press(Message::TogglePopup)
            .into()
    }

    // This is the menu that opens when you click the panel icon
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let content = widget::list_column()
            .padding(12)
            .add(widget::settings::item(
                "Limit Charge to 80%",
                widget::toggler(self.conservation_enabled)
                    .on_toggle(Message::ToggleConservation),
            ));

        self.core.applet.popup_container(content).into()
    }

    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None, None, None,
                    );
                    settings.positioner.size_limits = Limits::NONE.min_width(300.0).min_height(80.0);
                    get_popup(settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup == Some(id) { self.popup = None; }
            }
            Message::ToggleConservation(enabled) => {
                self.conservation_enabled = enabled;
                let val = if enabled { "1" } else { "0" };

                // Execute the write command via pkexec for root privileges
                let _ = Command::new("pkexec")
                    .arg("sh")
                    .arg("-c")
                    .arg(format!("echo {} > {}", val, BAT_PATH))
                    .spawn();
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}
