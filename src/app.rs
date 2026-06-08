// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::{futures, window::Id, Limits, Subscription};
use cosmic::prelude::*;
use cosmic::widget;
use futures::SinkExt;

/// ### The Application Model (State)
/// This struct maintains the entire reactive state of the applet.
pub struct AppModel {
    /// Core context required by the COSMIC SDK to hook into panel/desktop operations.
    core: cosmic::Core,
    /// Stores the unique Wayland window identifier for the active applet popup, if open.
    popup: Option<Id>,
    /// System configuration settings handle managed by the `cosmic-config` service.
    config: Config,
    /// The current display color temperature value in Kelvin (e.g., 3000K - 6500K).
    temperature: u16, 
}

impl Default for AppModel {
    /// Provides standard fallback state initialization metrics when the applet launches.
    fn default() -> Self {
        Self {
            core: cosmic::Core::default(),
            popup: None,
            config: Config::default(),
            temperature: 6500, // Default to clean daylight white
        }
    }
}

/// ### Application Messages (Events)
/// Enum tracking all user actions, asynchronous events, and state mutations.
#[derive(Debug, Clone)]
pub enum Message {
    /// User clicked the applet icon; toggles visibility of the popup container.
    TogglePopup,
    /// Event fired by the compositor indicating a popup window was closed.
    PopupClosed(Id),
    /// Asynchronous signaling channel message to initialize custom stream routines.
    SubscriptionChannel,
    /// Fired whenever the underlying `cosmic-config` file changes on the disk.
    UpdateConfig(Config),
    /// Emitted continuously when the user drags the temperature slider.
    TemperatureChanged(u16), 
}

/// ### COSMIC Application Engine Blueprint
/// Implements the main Application runtime lifecycle using System76's reactive design rules.
impl cosmic::Application for AppModel {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = "com.github.sajiiiid.CosmicNightlight";

    /// Read-only accessor exposing core context metadata to the layout runtime.
    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    /// Mutable accessor allowing the runtime engine to alter layout context values.
    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// #### 1. Initialization
    /// Configures the initial application space, parsing localized config layers upon launch.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        let app = AppModel {
            core,
            // Safely verify and parse user-space configuration settings via the desktop daemon
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => config,
                })
                .unwrap_or_default(),
            temperature: 6500,
            ..Default::default()
        };

        (app, Task::none())
    }

    /// Intercepts window/popup termination requests issued by the underlying window manager.
    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// #### 2. View Layer (The Inline Applet Icon)
    /// Renders what users physically see inside the permanent COSMIC Panel interface.
    fn view(&self) -> Element<'_, Self::Message> {
        self.core
            .applet
            .icon_button("display-symbolic") // Pulls standard display icon from active system theme
            .on_press(Message::TogglePopup)  // Dispatches the Toggle message upon clicking
            .into()
    }

    /// #### 3. Window View Layer (The Popup Tray Slider)
    /// Handles visual construction of the anchoring sub-surface layout when the popup is toggled open.
    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let content_list = widget::list_column()
            .add(widget::text(format!("Temperature: {}K", self.temperature)))
            .add(
                widget::slider(
                    3000..=6500, // Restricts user boundaries from amber-warm up to neutral daylight
                    self.temperature,
                    Message::TemperatureChanged, // Emits message updates continuously on slide adjustments
                )
            );

        // Decorates components nicely within the official system applet container wrapper
        self.core.applet.popup_container(content_list).into()
    }

    /// #### 4. Subscription Event Listeners
    /// Watches persistent system event channels, config daemons, and reactive data streams.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            // Spawns a multi-threaded channel handler stream to watch structural execution patterns
            Subscription::run(|| {
                cosmic::iced::stream::channel(4, move |mut channel: futures::channel::mpsc::Sender<_>| async move {
                    _ = channel.send(Message::SubscriptionChannel).await;
                    futures::future::pending().await
                })
            }),
            // Hooks straight into the DBus configuration manager to listen to live property tweaks
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| Message::UpdateConfig(update.config)),
        ])
    }

    /// #### 5. The State Update Handler
    /// Decodes messages and mutates variables, then returns sub-surface commands to the compositor.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::SubscriptionChannel => {}
            
            Message::UpdateConfig(config) => {
                self.config = config;
            }
            
            Message::TemperatureChanged(temp) => {
                self.temperature = temp;
                
                // Native system call: Pipes temperature parameters down into the gammastep executable.
                // Note: Actual desktop tint shifts require cosmic-comp's Wayland protocols to process.
                let _ = std::process::Command::new("gammastep")
                    .arg("-O")
                    .arg(temp.to_string())
                    .spawn();

                println!("Slider moved. Target temperature sent to gammastep: {}K", self.temperature);
            }
            
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    // If window state contains a valid ID, it's open: instruct Wayland to destroy it
                    destroy_popup(p)
                } else {
                    // Otherwise, register a fresh unique ID to map a new sub-surface menu layer
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    
                    // Constrain popup window sizing boundaries explicitly to avoid screen stretching
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(372.0)
                        .min_width(300.0)
                        .min_height(100.0)
                        .max_height(1080.0);
                        
                    get_popup(popup_settings)
                }
            }
            
            Message::PopupClosed(id) => {
                // Garbage-collect references if the compositor closes the window behind the scenes
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
        }
        Task::none()
    }

    /// Standardizes the CSS/Theming layer to align with System76 global desktop styling options.
    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}