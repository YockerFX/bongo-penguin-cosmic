use std::time::{Duration, Instant};

use cosmic::Element;
use cosmic::app::{self, Core};
use cosmic::iced::{self, Alignment, Length, Rectangle, Subscription, window::Id};
use cosmic::surface::action::{app_popup, destroy_popup};
use cosmic::widget::{button, column, row, segmented_button, tab_bar, text};

use crate::input::{self, InputEvent, Side};
use crate::persistence;

pub const APP_ID: &str = "io.github.yockerfx.CosmicAppletBongoPenguin";
const SAVE_INTERVAL: Duration = Duration::from_secs(5);
const ANIM_TICK: Duration = Duration::from_millis(50);
const DECAY: Duration = Duration::from_millis(120);

const SVG_NONE: &[u8] = include_bytes!("../assets/none.svg");
const SVG_LEFT: &[u8] = include_bytes!("../assets/left.svg");
const SVG_RIGHT: &[u8] = include_bytes!("../assets/right.svg");
const SVG_BOTH: &[u8] = include_bytes!("../assets/both.svg");

const SKINS: &[&str] = &["Classic", "Cosmic", "Retro"];
const ACHIEVEMENTS: &[(u64, &str)] = &[
    (100, "Warming up"),
    (1_000, "Getting there"),
    (10_000, "Keyboard warrior"),
    (100_000, "Bongo master"),
];
const GITHUB_URL: &str = "https://github.com/YockerFX/bongo-penguin-cosmic";
// TODO: replace with real invite
const DISCORD_URL: &str = "https://discord.gg/your-invite";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AnimState {
    None,
    Left,
    Right,
    Both,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Tab {
    Cosmetics,
    Achievements,
    About,
}

pub struct BongoPenguin {
    core: Core,
    count: u64,
    last_saved: u64,
    left_held: u32,
    right_held: u32,
    shown: AnimState,
    last_active_at: Option<Instant>,
    popup: Option<Id>,
    tabs: segmented_button::SingleSelectModel,
    selected_skin: usize,
}

#[derive(Clone, Debug)]
pub enum Message {
    PopupClosed(Id),
    Surface(cosmic::surface::Action),
    TabActivated(segmented_button::Entity),
    SkinSelected(usize),
    OpenUrl(&'static str),
    Input(InputEvent),
    SaveTick,
    AnimTick,
}

impl cosmic::Application for BongoPenguin {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _flags: Self::Flags) -> (Self, app::Task<Self::Message>) {
        let count = persistence::load().unwrap_or_else(|| {
            tracing::info!("no existing count file (or tamper) — starting at 0");
            0
        });
        tracing::info!(count, "loaded persisted count");
        let tabs = segmented_button::Model::builder()
            .insert(|b| b.text("Cosmetics").data(Tab::Cosmetics).activate())
            .insert(|b| b.text("Achievements").data(Tab::Achievements))
            .insert(|b| b.text("About").data(Tab::About))
            .build();
        (
            Self {
                core,
                count,
                last_saved: count,
                left_held: 0,
                right_held: 0,
                shown: AnimState::None,
                last_active_at: None,
                popup: None,
                tabs,
                selected_skin: 0,
            },
            app::Task::none(),
        )
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch([
            input::subscription().map(Message::Input),
            iced::time::every(SAVE_INTERVAL).map(|_| Message::SaveTick),
            iced::time::every(ANIM_TICK).map(|_| Message::AnimTick),
        ])
    }

    fn update(&mut self, message: Self::Message) -> app::Task<Self::Message> {
        match message {
            Message::PopupClosed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
            Message::TabActivated(entity) => self.tabs.activate(entity),
            Message::SkinSelected(i) => {
                tracing::info!(
                    index = i,
                    name = SKINS.get(i).copied().unwrap_or(""),
                    "skin selected"
                );
                self.selected_skin = i;
            }
            Message::OpenUrl(url) => {
                if let Err(e) = std::process::Command::new("xdg-open").arg(url).spawn() {
                    tracing::warn!(%e, url, "failed to open url");
                }
            }
            Message::Input(ev) => self.on_input(ev, Instant::now()),
            Message::SaveTick => {
                if self.count != self.last_saved {
                    match persistence::save(self.count) {
                        Ok(()) => {
                            tracing::debug!(count = self.count, "persisted");
                            self.last_saved = self.count;
                        }
                        Err(e) => tracing::warn!(%e, "failed to persist count"),
                    }
                }
            }
            Message::AnimTick => self.recompute(Instant::now()),
        }
        app::Task::none()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        let bytes: &[u8] = match self.shown {
            AnimState::None => SVG_NONE,
            AnimState::Left => SVG_LEFT,
            AnimState::Right => SVG_RIGHT,
            AnimState::Both => SVG_BOTH,
        };
        let (_w, h) = self.core.applet.suggested_size(false);
        let (_pad_major, pad_minor) = self.core.applet.suggested_padding(false);
        let icon_height = (h as f32) + (pad_minor as f32) * 2.0;
        let icon_width = icon_height * (1152.0 / 768.0);
        let svg_handle = cosmic::widget::svg::Handle::from_memory(bytes);
        let icon = cosmic::widget::svg(svg_handle)
            .width(Length::Fixed(icon_width))
            .height(Length::Fixed(icon_height))
            .content_fit(iced::ContentFit::Contain);
        let counter = self.core.applet.text(self.count.to_string());

        let content: Element<'_, Message> = if self.core.applet.is_horizontal() {
            cosmic::widget::row::with_children(vec![icon.into(), counter.into()])
                .spacing(6)
                .align_y(Alignment::Center)
                .into()
        } else {
            cosmic::widget::column::with_children(vec![icon.into(), counter.into()])
                .spacing(2)
                .align_x(Alignment::Center)
                .into()
        };

        let popup_id = self.popup;
        let trigger = button::custom(content)
            .class(cosmic::theme::Button::AppletIcon)
            .padding([2, 6])
            .on_press_with_rectangle(move |offset, bounds| {
                if let Some(id) = popup_id {
                    Message::Surface(destroy_popup(id))
                } else {
                    Message::Surface(app_popup::<BongoPenguin>(
                        move |state: &mut BongoPenguin| {
                            let new_id = Id::unique();
                            state.popup = Some(new_id);
                            let mut settings = state.core.applet.get_popup_settings(
                                state.core.main_window_id().unwrap(),
                                new_id,
                                None,
                                None,
                                None,
                            );
                            settings.positioner.anchor_rect = Rectangle {
                                x: (bounds.x - offset.x) as i32,
                                y: (bounds.y - offset.y) as i32,
                                width: bounds.width as i32,
                                height: bounds.height as i32,
                            };
                            settings
                        },
                        Some(Box::new(|state: &BongoPenguin| {
                            Element::from(state.core.applet.popup_container(state.popup_view()))
                                .map(cosmic::Action::App)
                        })),
                    ))
                }
            });

        self.core.applet.autosize_window(trigger).into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        self.popup_view()
    }

    fn style(&self) -> Option<cosmic::iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}

impl BongoPenguin {
    fn on_input(&mut self, ev: InputEvent, now: Instant) {
        match ev {
            InputEvent::Down(side) => {
                self.count = self.count.saturating_add(1);
                tracing::trace!(?side, count = self.count, "down");
                match side {
                    Some(Side::Left) => self.left_held = self.left_held.saturating_add(1),
                    Some(Side::Right) => self.right_held = self.right_held.saturating_add(1),
                    None => {}
                }
            }
            InputEvent::Up(side) => match side {
                Some(Side::Left) => self.left_held = self.left_held.saturating_sub(1),
                Some(Side::Right) => self.right_held = self.right_held.saturating_sub(1),
                None => {}
            },
        }
        self.recompute(now);
    }

    fn recompute(&mut self, now: Instant) {
        let active = match (self.left_held, self.right_held) {
            (0, 0) => None,
            (l, 0) if l > 0 => Some(AnimState::Left),
            (0, r) if r > 0 => Some(AnimState::Right),
            _ => Some(AnimState::Both),
        };
        match active {
            Some(s) => {
                self.shown = s;
                self.last_active_at = Some(now);
            }
            None => {
                if let Some(t) = self.last_active_at {
                    if now.duration_since(t) >= DECAY {
                        self.shown = AnimState::None;
                        self.last_active_at = None;
                    }
                } else {
                    self.shown = AnimState::None;
                }
            }
        }
    }

    fn popup_view(&self) -> Element<'_, Message> {
        let tabs = tab_bar::horizontal(&self.tabs)
            .on_activate(Message::TabActivated)
            .width(Length::Fill);

        let body: Element<'_, Message> = match self.tabs.active_data::<Tab>().copied() {
            Some(Tab::Cosmetics) => self.view_cosmetics(),
            Some(Tab::Achievements) => self.view_achievements(),
            Some(Tab::About) | None => self.view_about(),
        };

        column::with_children(vec![tabs.into(), body])
            .spacing(12)
            .padding(12)
            .into()
    }

    fn view_cosmetics(&self) -> Element<'_, Message> {
        let title = text("Skin").size(14);
        let hint = text("Selection is a placeholder — SVGs don't change yet.").size(11);

        let mut choices = row::with_capacity(SKINS.len()).spacing(6);
        for (i, name) in SKINS.iter().enumerate() {
            let btn = if i == self.selected_skin {
                button::suggested(*name).on_press(Message::SkinSelected(i))
            } else {
                button::standard(*name).on_press(Message::SkinSelected(i))
            };
            choices = choices.push(btn);
        }

        column::with_children(vec![title.into(), choices.into(), hint.into()])
            .spacing(8)
            .into()
    }

    fn view_achievements(&self) -> Element<'_, Message> {
        let mut col = column::with_capacity(ACHIEVEMENTS.len() + 1).spacing(6);
        col = col.push(text(format!("Total keystrokes: {}", self.count)).size(14));
        for (threshold, label) in ACHIEVEMENTS {
            let unlocked = self.count >= *threshold;
            let marker = if unlocked { "✓" } else { "·" };
            col = col.push(text(format!("{marker}  {label} ({threshold})")).size(13));
        }
        col.into()
    }

    fn view_about(&self) -> Element<'_, Message> {
        let github = button::link("GitHub repository").on_press(Message::OpenUrl(GITHUB_URL));
        let discord = button::link("Join the Discord").on_press(Message::OpenUrl(DISCORD_URL));
        column::with_children(vec![
            text("Bongo Penguin").size(14).into(),
            text(concat!("Version ", env!("CARGO_PKG_VERSION")))
                .size(11)
                .into(),
            github.into(),
            discord.into(),
        ])
        .spacing(8)
        .into()
    }
}
