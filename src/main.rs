use eframe::egui;
use chrono::{Local, Timelike};
use std::f32::consts::PI;
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// ─────────────────────────────────────────────
// Enums & Theme definitions
// ─────────────────────────────────────────────

#[derive(PartialEq, Clone, Copy)]
enum AppMode {
    Clock,
    Stopwatch,
    Timer,
    Alarm,
    Settings,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum TimeFormat {
    Hour12,
    Hour24,
}

#[derive(PartialEq, Clone, Copy, Debug)]
enum Theme {
    Midnight,   // deep navy / indigo
    Aurora,     // teal / emerald
    Sakura,     // soft rose / peach
    Dusk,       // amber / warm purple
    Slate,      // cool grey / cyan
}

impl Theme {
    fn name(&self) -> &'static str {
        match self {
            Theme::Midnight => "Midnight",
            Theme::Aurora   => "Aurora",
            Theme::Sakura   => "Sakura",
            Theme::Dusk     => "Dusk",
            Theme::Slate    => "Slate",
        }
    }

    /// Primary accent colour
    fn accent(&self) -> egui::Color32 {
        match self {
            Theme::Midnight => egui::Color32::from_rgb(130, 160, 255),
            Theme::Aurora   => egui::Color32::from_rgb(80,  210, 170),
            Theme::Sakura   => egui::Color32::from_rgb(255, 160, 180),
            Theme::Dusk     => egui::Color32::from_rgb(255, 190,  90),
            Theme::Slate    => egui::Color32::from_rgb(100, 210, 230),
        }
    }

    /// Secondary / softer accent
    fn accent2(&self) -> egui::Color32 {
        match self {
            Theme::Midnight => egui::Color32::from_rgb( 80, 110, 200),
            Theme::Aurora   => egui::Color32::from_rgb( 40, 160, 130),
            Theme::Sakura   => egui::Color32::from_rgb(200, 110, 140),
            Theme::Dusk     => egui::Color32::from_rgb(200, 140,  50),
            Theme::Slate    => egui::Color32::from_rgb( 60, 160, 180),
        }
    }

    /// Card / panel fill (dark base)
    fn panel_fill(&self) -> egui::Color32 {
        match self {
            Theme::Midnight => egui::Color32::from_rgb(12, 16, 38),
            Theme::Aurora   => egui::Color32::from_rgb( 8, 26, 24),
            Theme::Sakura   => egui::Color32::from_rgb(30, 14, 20),
            Theme::Dusk     => egui::Color32::from_rgb(28, 18,  8),
            Theme::Slate    => egui::Color32::from_rgb(14, 20, 26),
        }
    }

    /// Subtle ring / track colour
    fn track_color(&self) -> egui::Color32 {
        let [r, g, b, _] = self.panel_fill().to_array();
        egui::Color32::from_rgb(
            (r as u16 + 30).min(255) as u8,
            (g as u16 + 30).min(255) as u8,
            (b as u16 + 30).min(255) as u8,
        )
    }

    fn text_secondary(&self) -> egui::Color32 {
        egui::Color32::from_rgb(160, 168, 185)
    }

    /// Top bar background
    fn bar_fill(&self) -> egui::Color32 {
        let [r, g, b, _] = self.panel_fill().to_array();
        egui::Color32::from_rgb(
            (r as u16 + 8).min(255) as u8,
            (g as u16 + 8).min(255) as u8,
            (b as u16 + 8).min(255) as u8,
        )
    }
}

// ─────────────────────────────────────────────
// App state
// ─────────────────────────────────────────────

struct ClockApp {
    mode: AppMode,

    // Settings
    time_format: TimeFormat,
    theme: Theme,
    bg_opacity: f32,           // 0.0 – 1.0
    bg_image_path: Option<String>,
    bg_texture: Option<egui::TextureHandle>,
    bg_texture_size: Option<egui::Vec2>,

    // Stopwatch state
    sw_running: bool,
    sw_start: Option<Instant>,
    sw_elapsed: Duration,

    // Timer state
    timer_running: bool,
    timer_minutes: u32,
    timer_seconds: u32,
    timer_duration_secs: u32,
    timer_start: Option<Instant>,
    timer_left: Duration,
    timer_finished: bool,
    sound_playing: bool,

    // Alarm state
    alarm_hour: u32,
    alarm_minute: u32,
    alarm_active: bool,
    alarm_ringing: bool,
}

impl Default for ClockApp {
    fn default() -> Self {
        Self {
            mode: AppMode::Clock,

            time_format: TimeFormat::Hour12,
            theme: Theme::Midnight,
            bg_opacity: 0.35,
            bg_image_path: None,
            bg_texture: None,
            bg_texture_size: None,

            sw_running: false,
            sw_start: None,
            sw_elapsed: Duration::ZERO,

            timer_running: false,
            timer_minutes: 1,
            timer_seconds: 0,
            timer_duration_secs: 60,
            timer_start: None,
            timer_left: Duration::from_secs(60),
            timer_finished: false,
            sound_playing: false,

            alarm_hour: 8,
            alarm_minute: 0,
            alarm_active: false,
            alarm_ringing: false,
        }
    }
}

// ─────────────────────────────────────────────
// Sound
// ─────────────────────────────────────────────

fn play_system_sound() {
    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let script = r#"
        [console]::beep(880, 180); Start-Sleep -m 120
        [console]::beep(880, 180); Start-Sleep -m 120
        [console]::beep(1100, 300)
        "#;
        let _ = std::process::Command::new("powershell")
            .args(&["-c", script])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("sh")
            .args(&["-c", "afplay /System/Library/Sounds/Ping.aiff; sleep 0.3; afplay /System/Library/Sounds/Ping.aiff; sleep 0.3; afplay /System/Library/Sounds/Ping.aiff"])
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("sh")
            .args(&["-c", "canberra-gtk-play -i alarm-clock-elapsed || paplay /usr/share/sounds/freedesktop/stereo/alarm-clock-elapsed.oga || aplay /usr/share/sounds/alsa/Front_Center.wav"])
            .spawn();
    }
}

// ─────────────────────────────────────────────
// Load image helper
// ─────────────────────────────────────────────

fn load_image_from_path(path: &str) -> Option<(egui::ColorImage, egui::Vec2)> {
    let img = image::open(path).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixels: Vec<egui::Color32> = rgba
        .pixels()
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();
    let color_image = egui::ColorImage { size: [w as usize, h as usize], pixels };
    Some((color_image, egui::vec2(w as f32, h as f32)))
}

// ─────────────────────────────────────────────
// eframe::App
// ─────────────────────────────────────────────

impl eframe::App for ClockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        let t = self.theme;

        // ── Global egui visuals ──────────────────
        {
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill              = t.bar_fill();
            visuals.window_fill             = t.panel_fill();
            visuals.faint_bg_color          = t.panel_fill();
            visuals.extreme_bg_color        = t.panel_fill();
            visuals.widgets.noninteractive.bg_fill  = t.panel_fill();
            visuals.widgets.inactive.bg_fill        = t.panel_fill();
            visuals.widgets.inactive.fg_stroke      = egui::Stroke::new(1.0, t.text_secondary());
            visuals.widgets.hovered.bg_fill         = t.accent2();
            visuals.widgets.hovered.fg_stroke       = egui::Stroke::new(1.5, t.accent());
            visuals.widgets.active.bg_fill          = t.accent();
            visuals.selection.bg_fill               = t.accent2();
            visuals.selection.stroke                = egui::Stroke::new(1.0, t.accent());
            visuals.override_text_color = Some(egui::Color32::from_rgb(220, 225, 235));
            ctx.set_visuals(visuals);
        }

        // ── Background wallpaper ─────────────────
        if let Some(tex) = &self.bg_texture {
            let screen = ctx.screen_rect();
            let tex_size = self.bg_texture_size.unwrap_or(egui::vec2(1.0, 1.0));

            // Cover crop: scale to fill, centred
            let scale_x = screen.width()  / tex_size.x;
            let scale_y = screen.height() / tex_size.y;
            let scale   = scale_x.max(scale_y);
            let draw_w  = tex_size.x * scale;
            let draw_h  = tex_size.y * scale;
            let ox      = (screen.width()  - draw_w) / 2.0;
            let oy      = (screen.height() - draw_h) / 2.0;
            let img_rect = egui::Rect::from_min_size(
                screen.min + egui::vec2(ox, oy),
                egui::vec2(draw_w, draw_h),
            );

            let alpha = (self.bg_opacity * 255.0) as u8;
            let tint  = egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha);

            let painter = ctx.layer_painter(egui::LayerId::background());
            painter.image(tex.id(), img_rect, egui::Rect::from_min_max(
                egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0),
            ), tint);
        }

        // ── Top navigation bar ───────────────────
        egui::TopBottomPanel::top("nav")
            .frame(egui::Frame::default()
                .fill(t.bar_fill())
                .inner_margin(egui::Margin::symmetric(12.0, 10.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let tabs = [
                        (AppMode::Clock,     "🕰  Clock"),
                        (AppMode::Stopwatch, "⏱  Stopwatch"),
                        (AppMode::Timer,     "⏳  Timer"),
                        (AppMode::Alarm,     "⏰  Alarm"),
                    ];
                    for (mode, label) in tabs {
                        let active = self.mode == mode;
                        let text = egui::RichText::new(label)
                            .size(13.5)
                            .color(if active { t.accent() } else { t.text_secondary() })
                            .strong();
                        if ui.add(egui::Label::new(text).sense(egui::Sense::click())).clicked() {
                            self.mode = mode;
                        }
                        if active {
                            // underline indicator
                            let r = ui.min_rect();
                            ui.painter().line_segment(
                                [r.left_bottom() + egui::vec2(-4.0, 3.0),
                                 r.right_bottom() + egui::vec2(0.0, 3.0)],
                                egui::Stroke::new(2.0, t.accent()),
                            );
                        }
                        ui.add_space(14.0);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let gear = egui::RichText::new("⚙")
                            .size(18.0)
                            .color(if self.mode == AppMode::Settings { t.accent() } else { t.text_secondary() });
                        if ui.add(egui::Label::new(gear).sense(egui::Sense::click())).clicked() {
                            self.mode = if self.mode == AppMode::Settings {
                                AppMode::Clock
                            } else {
                                AppMode::Settings
                            };
                        }
                    });
                });
            });

        // ── Central panel ────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::TRANSPARENT))
            .show(ctx, |ui| {
                // Dim overlay over wallpaper for readability
                if self.bg_texture.is_some() {
                    let overlay_alpha = ((1.0 - self.bg_opacity) * 160.0) as u8;
                    ui.painter().rect_filled(
                        ui.max_rect(),
                        0.0,
                        egui::Color32::from_rgba_unmultiplied(
                            t.panel_fill().r(),
                            t.panel_fill().g(),
                            t.panel_fill().b(),
                            overlay_alpha,
                        ),
                    );
                }

                ui.vertical_centered(|ui| {
                    ui.add_space(28.0);

                    let card_frame = egui::Frame::default()
                        .fill(egui::Color32::from_rgba_unmultiplied(
                            t.panel_fill().r(),
                            t.panel_fill().g(),
                            t.panel_fill().b(),
                            if self.bg_texture.is_some() { 200 } else { 255 },
                        ))
                        .rounding(egui::Rounding::same(20.0))
                        .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(
                            t.accent().r(), t.accent().g(), t.accent().b(), 40,
                        )))
                        .inner_margin(egui::Margin::same(28.0));

                    card_frame.show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            match self.mode {
                                AppMode::Clock     => self.show_clock(ui),
                                AppMode::Stopwatch => self.show_stopwatch(ui),
                                AppMode::Timer     => self.show_timer(ui, ctx),
                                AppMode::Alarm     => self.show_alarm(ui, ctx),
                                AppMode::Settings  => self.show_settings(ui, ctx),
                            }
                        });
                    });
                });
            });
    }
}

// ─────────────────────────────────────────────
// Panels
// ─────────────────────────────────────────────

impl ClockApp {
    // ── Clock ────────────────────────────────
    fn show_clock(&mut self, ui: &mut egui::Ui) {
        let now = Local::now();
        let t   = self.theme;
        ui.add_space(10.0);

        let time_str = match self.time_format {
            TimeFormat::Hour12 => now.format("%I:%M:%S %p").to_string(),
            TimeFormat::Hour24 => now.format("%H:%M:%S").to_string(),
        };
        let date_str = now.format("%A, %B %-d, %Y").to_string();

        ui.label(egui::RichText::new(&time_str)
            .size(58.0)
            .strong()
            .color(t.accent()));
        ui.add_space(8.0);
        ui.label(egui::RichText::new(&date_str)
            .size(17.0)
            .color(t.text_secondary()));
        ui.add_space(8.0);
    }

    // ── Stopwatch ────────────────────────────
    fn show_stopwatch(&mut self, ui: &mut egui::Ui) {
        let t = self.theme;
        section_heading(ui, "Stopwatch", t);
        ui.add_space(16.0);

        if self.sw_running {
            if let Some(start) = self.sw_start {
                self.sw_elapsed = start.elapsed();
            }
        }

        let secs    = self.sw_elapsed.as_secs();
        let millis  = self.sw_elapsed.subsec_millis() / 10;
        let display = format!("{:02}:{:02}:{:02}.{:02}",
            secs / 3600, (secs % 3600) / 60, secs % 60, millis);

        ui.label(egui::RichText::new(display)
            .size(48.0)
            .strong()
            .color(t.accent()));
        ui.add_space(20.0);

        ui.horizontal(|ui| {
            center_buttons(ui, 160.0, |ui| {
                if self.sw_running {
                    if accent_button(ui, "  Pause  ", t).clicked() {
                        self.sw_running = false;
                    }
                } else {
                    if accent_button(ui, "  Start  ", t).clicked() {
                        self.sw_start   = Some(Instant::now() - self.sw_elapsed);
                        self.sw_running = true;
                    }
                }
                ui.add_space(10.0);
                if ghost_button(ui, "Reset", t).clicked() {
                    self.sw_running = false;
                    self.sw_elapsed = Duration::ZERO;
                }
            });
        });
    }

    // ── Timer ────────────────────────────────
    fn show_timer(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let t = self.theme;
        section_heading(ui, "Timer", t);
        ui.add_space(12.0);

        if self.timer_running {
            if let Some(start) = self.timer_start {
                let elapsed = start.elapsed();
                let total   = Duration::from_secs(self.timer_duration_secs as u64);
                if elapsed >= total {
                    self.timer_running = false;
                    self.timer_left    = Duration::ZERO;
                    self.timer_finished = true;
                    if !self.sound_playing {
                        play_system_sound();
                        self.sound_playing = true;
                    }
                } else {
                    self.timer_left = total - elapsed;
                }
            }
        }

        let progress = if self.timer_duration_secs > 0 {
            self.timer_left.as_secs_f32() / self.timer_duration_secs as f32
        } else { 0.0 };

        draw_ring(ui, progress, self.timer_finished, ctx.input(|i| i.time), t);
        ui.add_space(14.0);

        let secs = self.timer_left.as_secs();
        ui.label(egui::RichText::new(format!("{:02}:{:02}", secs / 60, secs % 60))
            .size(38.0)
            .strong()
            .color(t.accent()));

        ui.add_space(16.0);

        if !self.timer_running && !self.timer_finished {
            ui.horizontal(|ui| {
                center_buttons(ui, 260.0, |ui| {
                    ui.label(egui::RichText::new("min").color(t.text_secondary()));
                    ui.add(egui::DragValue::new(&mut self.timer_minutes).range(0..=99).speed(1.0));
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("sec").color(t.text_secondary()));
                    ui.add(egui::DragValue::new(&mut self.timer_seconds).range(0..=59).speed(1.0));
                });
            });
            ui.add_space(12.0);
        }

        ui.horizontal(|ui| {
            center_buttons(ui, 170.0, |ui| {
                if self.timer_finished {
                    if accent_button(ui, " Dismiss ", t).clicked() {
                        self.timer_finished   = false;
                        self.sound_playing    = false;
                        self.timer_left       = Duration::from_secs(self.timer_duration_secs as u64);
                        self.timer_minutes    = self.timer_duration_secs / 60;
                        self.timer_seconds    = self.timer_duration_secs % 60;
                    }
                } else if self.timer_running {
                    if accent_button(ui, " Pause ", t).clicked() {
                        self.timer_running = false;
                        let s = self.timer_left.as_secs();
                        self.timer_minutes = (s / 60) as u32;
                        self.timer_seconds = (s % 60) as u32;
                    }
                } else {
                    if accent_button(ui, " Start ", t).clicked() {
                        let total = self.timer_minutes * 60 + self.timer_seconds;
                        if total > 0 {
                            self.timer_duration_secs = total;
                            self.timer_left          = Duration::from_secs(total as u64);
                            self.timer_start         = Some(Instant::now());
                            self.timer_running       = true;
                            self.timer_finished      = false;
                        }
                    }
                }
                ui.add_space(10.0);
                if ghost_button(ui, "Reset", t).clicked() {
                    self.timer_running  = false;
                    self.timer_finished = false;
                    self.sound_playing  = false;
                    self.timer_left     = Duration::from_secs(self.timer_duration_secs as u64);
                }
            });
        });
    }

    // ── Alarm ────────────────────────────────
    fn show_alarm(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let t   = self.theme;
        let now = Local::now().time();
        section_heading(ui, "Alarm", t);
        ui.add_space(12.0);

        if self.alarm_active {
            if now.hour() == self.alarm_hour
                && now.minute() == self.alarm_minute
                && now.second() < 2
            {
                self.alarm_active  = false;
                self.alarm_ringing = true;
                play_system_sound();
            }
        }

        let progress = if self.alarm_active {
            let ms      = now.nanosecond() as f32 / 1_000_000.0;
            let total   = (now.second() as f32 * 1000.0) + ms;
            (total % 60_000.0) / 60_000.0
        } else if self.alarm_ringing { 0.0 } else { 1.0 };

        draw_ring(ui, progress, self.alarm_ringing, ctx.input(|i| i.time), t);
        ui.add_space(16.0);

        if self.alarm_ringing {
            ui.label(egui::RichText::new("⏰  WAKE UP!")
                .size(36.0)
                .strong()
                .color(egui::Color32::from_rgb(255, 110, 110)));
            ui.add_space(12.0);
            if accent_button(ui, "Turn Off", t).clicked() {
                self.alarm_ringing = false;
            }
        } else {
            ui.horizontal(|ui| {
                center_buttons(ui, 210.0, |ui| {
                    ui.label(egui::RichText::new("Hour").color(t.text_secondary()));
                    ui.add(egui::DragValue::new(&mut self.alarm_hour).range(0..=23).speed(1.0));
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Min").color(t.text_secondary()));
                    ui.add(egui::DragValue::new(&mut self.alarm_minute).range(0..=59).speed(1.0));
                });
            });
            ui.add_space(12.0);

            let btn_label = if self.alarm_active { " Cancel " } else { "Set Alarm" };
            ui.horizontal(|ui| {
                center_buttons(ui, 100.0, |ui| {
                    if accent_button(ui, btn_label, t).clicked() {
                        self.alarm_active = !self.alarm_active;
                    }
                });
            });

            if self.alarm_active {
                ui.add_space(10.0);
                let alarm_time = match self.time_format {
                    TimeFormat::Hour12 => {
                        let h12 = if self.alarm_hour == 0 { 12 }
                                  else if self.alarm_hour > 12 { self.alarm_hour - 12 }
                                  else { self.alarm_hour };
                        let am  = if self.alarm_hour < 12 { "AM" } else { "PM" };
                        format!("{:02}:{:02} {}", h12, self.alarm_minute, am)
                    }
                    TimeFormat::Hour24 =>
                        format!("{:02}:{:02}", self.alarm_hour, self.alarm_minute),
                };
                ui.label(egui::RichText::new(format!("Alarm set for {}", alarm_time))
                    .size(14.0)
                    .color(t.text_secondary()));
            }
        }
    }

    // ── Settings ─────────────────────────────
    fn show_settings(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let t = self.theme;
        section_heading(ui, "Settings", t);
        ui.add_space(16.0);

        // ── Time format ──────────────────────
        setting_label(ui, "Time Format", t);
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            center_buttons(ui, 190.0, |ui| {
                let active12 = self.time_format == TimeFormat::Hour12;
                if toggle_btn(ui, "12-hour", active12, t).clicked() {
                    self.time_format = TimeFormat::Hour12;
                }
                ui.add_space(8.0);
                let active24 = self.time_format == TimeFormat::Hour24;
                if toggle_btn(ui, "24-hour", active24, t).clicked() {
                    self.time_format = TimeFormat::Hour24;
                }
            });
        });

        ui.add_space(18.0);
        divider(ui, t);
        ui.add_space(14.0);

        // ── Theme picker ─────────────────────
        setting_label(ui, "Colour Theme", t);
        ui.add_space(8.0);

        let themes = [Theme::Midnight, Theme::Aurora, Theme::Sakura, Theme::Dusk, Theme::Slate];
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(8.0, 8.0);
            for th in themes {
                let selected = self.theme == th;
                let swatch_size = 34.0;
                let (rect, resp) = ui.allocate_exact_size(
                    egui::vec2(swatch_size, swatch_size + 18.0),
                    egui::Sense::click(),
                );
                if resp.clicked() { self.theme = th; }

                let swatch_rect = egui::Rect::from_min_size(
                    rect.min, egui::vec2(swatch_size, swatch_size),
                );
                ui.painter().rect_filled(swatch_rect, egui::Rounding::same(8.0), th.accent());
                if selected {
                    ui.painter().rect_stroke(
                        swatch_rect.expand(2.5),
                        egui::Rounding::same(10.0),
                        egui::Stroke::new(2.0, th.accent()),
                    );
                    // tick
                    ui.painter().circle_filled(swatch_rect.center(), 7.0,
                        egui::Color32::from_rgba_unmultiplied(0, 0, 0, 160));
                    ui.painter().text(
                        swatch_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "✓",
                        egui::FontId::proportional(13.0),
                        egui::Color32::WHITE,
                    );
                }
                ui.painter().text(
                    egui::pos2(rect.center().x, rect.bottom() - 2.0),
                    egui::Align2::CENTER_BOTTOM,
                    th.name(),
                    egui::FontId::proportional(10.5),
                    if selected { th.accent() } else { t.text_secondary() },
                );
            }
        });

        ui.add_space(18.0);
        divider(ui, t);
        ui.add_space(14.0);

        // ── Wallpaper ────────────────────────
        setting_label(ui, "Background Wallpaper", t);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            center_buttons(ui, 260.0, |ui| {
                if accent_button(ui, " Choose Image… ", t).clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp"])
                        .pick_file()
                    {
                        let path_str = path.to_string_lossy().to_string();
                        if let Some((color_image, size)) = load_image_from_path(&path_str) {
                            self.bg_texture = Some(ctx.load_texture(
                                "bg",
                                color_image,
                                egui::TextureOptions::LINEAR,
                            ));
                            self.bg_texture_size = Some(size);
                            self.bg_image_path   = Some(path_str);
                        }
                    }
                }
                if self.bg_texture.is_some() {
                    ui.add_space(8.0);
                    if ghost_button(ui, "Remove", t).clicked() {
                        self.bg_texture      = None;
                        self.bg_texture_size = None;
                        self.bg_image_path   = None;
                    }
                }
            });
        });

        if let Some(p) = &self.bg_image_path {
            ui.add_space(5.0);
            let short: String = p.chars().rev().take(40).collect::<String>()
                .chars().rev().collect();
            ui.label(egui::RichText::new(format!("…{}", short))
                .size(11.0)
                .color(t.text_secondary()));
        }

        ui.add_space(14.0);

        // ── Wallpaper opacity ─────────────────
        let opacity_enabled = self.bg_texture.is_some();
        setting_label(ui, "Wallpaper Opacity", t);
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            center_buttons(ui, 240.0, |ui| {
                let slider = egui::Slider::new(&mut self.bg_opacity, 0.05_f32..=1.0_f32)
                    .show_value(true)
                    .custom_formatter(|v, _| format!("{:.0}%", v * 100.0));
                ui.add_enabled(opacity_enabled, slider);
            });
        });
        if !opacity_enabled {
            ui.add_space(4.0);
            ui.label(egui::RichText::new("Load a wallpaper to adjust opacity")
                .size(11.5)
                .color(t.text_secondary()));
        }

        ui.add_space(6.0);
    }
}

// ─────────────────────────────────────────────
// Circular ring animation
// ─────────────────────────────────────────────

fn draw_ring(
    ui: &mut egui::Ui,
    progress: f32,
    finished: bool,
    time: f64,
    theme: Theme,
) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(200.0, 200.0), egui::Sense::hover());
    let center    = rect.center();
    let painter   = ui.painter();
    let radius    = 78.0_f32;
    let track_w   = 11.0_f32;

    if finished {
        // ── Finished: expanding ripple rings ──
        let pulse = ((time * 6.0).sin() as f32 * 0.5 + 0.5);
        let head_r = radius + pulse * 8.0;

        for i in 0..4 {
            let phase = (time * 1.6 + i as f64 * 0.28) % 1.0;
            let rr    = radius + phase as f32 * 55.0;
            let alpha = ((1.0 - phase as f32) * 200.0) as u8;
            let ac    = theme.accent();
            painter.circle_stroke(center, rr,
                egui::Stroke::new(2.5, egui::Color32::from_rgba_unmultiplied(ac.r(), ac.g(), ac.b(), alpha)));
        }
        // Filled glow
        for i in 1..=4_u8 {
            let ac = theme.accent();
            let alpha = 40 / i;
            painter.circle_stroke(center, head_r + i as f32 * 5.0,
                egui::Stroke::new(track_w + i as f32 * 3.0,
                    egui::Color32::from_rgba_unmultiplied(ac.r(), ac.g(), ac.b(), alpha)));
        }
        painter.circle_stroke(center, head_r,
            egui::Stroke::new(track_w, theme.accent()));

        // Centre label
        painter.text(center, egui::Align2::CENTER_CENTER, "✓",
            egui::FontId::proportional(36.0), theme.accent());
    } else {
        // ── Track background ──────────────────
        painter.circle_stroke(center, radius,
            egui::Stroke::new(track_w, theme.track_color()));

        // Subtle glow on track
        let ac = theme.accent();
        painter.circle_stroke(center, radius,
            egui::Stroke::new(track_w + 6.0,
                egui::Color32::from_rgba_unmultiplied(ac.r(), ac.g(), ac.b(), 18)));

        if progress > 0.001 {
            let segments    = 160_usize;
            let start_angle = -PI / 2.0_f32;
            let sweep       = progress * 2.0 * PI;

            // ── Draw arc segments with gradient brightness ──
            for i in 0..segments {
                let t0 = i       as f32 / segments as f32;
                let t1 = (i + 1) as f32 / segments as f32;
                if t0 >= progress { break; }

                let frac  = t0 / progress; // 0 = start, 1 = head
                // Brightness: dim at tail, bright at head
                let bright = (frac * frac * 0.75 + 0.25).min(1.0);
                let alpha   = (bright * 255.0) as u8;

                let color = egui::Color32::from_rgba_unmultiplied(
                    ((ac.r() as f32 * bright) as u8),
                    ((ac.g() as f32 * bright) as u8),
                    ((ac.b() as f32 * bright) as u8),
                    alpha,
                );

                let a0 = start_angle + t0 * sweep;
                let a1 = start_angle + t1 * sweep;
                let p0 = center + egui::vec2(a0.cos() * radius, a0.sin() * radius);
                let p1 = center + egui::vec2(a1.cos() * radius, a1.sin() * radius);

                painter.line_segment([p0, p1], egui::Stroke::new(track_w, color));

                // Glow pass — only near the head (last 20% of arc)
                if frac > 0.80 {
                    let glow_alpha = ((frac - 0.80) / 0.20 * 60.0) as u8;
                    painter.line_segment([p0, p1],
                        egui::Stroke::new(track_w + 6.0,
                            egui::Color32::from_rgba_unmultiplied(ac.r(), ac.g(), ac.b(), glow_alpha)));
                }
            }

            // ── Bright dot head ───────────────
            let head_angle = start_angle + progress * sweep;
            let head_pos   = center + egui::vec2(head_angle.cos() * radius, head_angle.sin() * radius);

            // Outer glow
            for i in 1..=3_u8 {
                painter.circle_filled(head_pos, track_w / 2.0 + 2.0 + i as f32 * 4.0,
                    egui::Color32::from_rgba_unmultiplied(ac.r(), ac.g(), ac.b(), 35 / i));
            }
            // Core dot
            painter.circle_filled(head_pos, track_w / 2.0 + 2.5, egui::Color32::WHITE);
            // Accent ring
            painter.circle_stroke(head_pos, track_w / 2.0 + 5.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(ac.r(), ac.g(), ac.b(), 180)));
        }

        // ── Percentage in centre ──────────────
        let pct_text = format!("{:.0}%", progress * 100.0);
        painter.text(center, egui::Align2::CENTER_CENTER, pct_text,
            egui::FontId::proportional(20.0),
            theme.text_secondary());
    }
}

// ─────────────────────────────────────────────
// UI helpers
// ─────────────────────────────────────────────

fn section_heading(ui: &mut egui::Ui, text: &str, t: Theme) {
    ui.label(egui::RichText::new(text)
        .size(15.0)
        .strong()
        .color(t.text_secondary()));
}

fn setting_label(ui: &mut egui::Ui, text: &str, t: Theme) {
    ui.label(egui::RichText::new(text)
        .size(13.5)
        .strong()
        .color(t.text_secondary()));
}

fn divider(ui: &mut egui::Ui, t: Theme) {
    let ac = t.accent();
    let (r, _) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter().line_segment(
        [r.left_center(), r.right_center()],
        egui::Stroke::new(0.5, egui::Color32::from_rgba_unmultiplied(ac.r(), ac.g(), ac.b(), 40)),
    );
}

fn center_buttons<R>(ui: &mut egui::Ui, width: f32, f: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let space = (ui.available_width() - width).max(0.0) / 2.0;
    ui.add_space(space);
    f(ui)
}

fn accent_button(ui: &mut egui::Ui, label: &str, t: Theme) -> egui::Response {
    let btn = egui::Button::new(
        egui::RichText::new(label).color(egui::Color32::from_rgb(15, 15, 20)).strong()
    )
    .fill(t.accent())
    .rounding(egui::Rounding::same(8.0));
    ui.add(btn)
}

fn ghost_button(ui: &mut egui::Ui, label: &str, t: Theme) -> egui::Response {
    let ac = t.accent();
    let btn = egui::Button::new(
        egui::RichText::new(label).color(t.text_secondary())
    )
    .fill(egui::Color32::TRANSPARENT)
    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(ac.r(), ac.g(), ac.b(), 70)))
    .rounding(egui::Rounding::same(8.0));
    ui.add(btn)
}

fn toggle_btn(ui: &mut egui::Ui, label: &str, active: bool, t: Theme) -> egui::Response {
    if active {
        accent_button(ui, label, t)
    } else {
        ghost_button(ui, label, t)
    }
}

// ─────────────────────────────────────────────
// main
// ─────────────────────────────────────────────

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([430.0, 580.0])
            .with_min_inner_size([340.0, 420.0])
            .with_title("Clocky"),
        ..Default::default()
    };

    eframe::run_native(
        "Clocky",
        options,
        Box::new(|_cc| Ok(Box::<ClockApp>::default())),
    )
}
