use std::path::Path;

use eframe::egui::{
    self, Color32, ColorImage, Context, FontFamily, FontId, Pos2, Rect, SidePanel, Stroke,
    TextureHandle, TextureOptions, TopBottomPanel, Vec2, Visuals,
};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "asset_hander - Step 1",
        options,
        Box::new(|_cc| Ok(Box::<AssetHanderApp>::default())),
    )
}

#[derive(Clone, Copy, Debug)]
struct GridConfig {
    rows: u32,
    columns: u32,
    cell_width: u32,
    cell_height: u32,
    offset_x: u32,
    offset_y: u32,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            rows: 4,
            columns: 4,
            cell_width: 0,
            cell_height: 0,
            offset_x: 0,
            offset_y: 0,
        }
    }
}

struct AssetHanderApp {
    image_name: Option<String>,
    image_size: Option<[usize; 2]>,
    image_texture: Option<TextureHandle>,
    draft_grid: GridConfig,
    applied_grid: GridConfig,
    zoom_level: f32,
    canvas_pan: Vec2,
    theme_applied: bool,
}

impl Default for AssetHanderApp {
    fn default() -> Self {
        Self {
            image_name: None,
            image_size: None,
            image_texture: None,
            draft_grid: GridConfig::default(),
            applied_grid: GridConfig::default(),
            zoom_level: 1.0,
            canvas_pan: Vec2::ZERO,
            theme_applied: false,
        }
    }
}

impl eframe::App for AssetHanderApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        if !self.theme_applied {
            apply_theme(ctx);
            self.theme_applied = true;
        }

        TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Image").clicked() {
                        self.open_image(ctx);
                        ui.close_menu();
                    }
                    if ui.button("Exit").clicked() {
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });

        SidePanel::left("left_controls")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Image");
                ui.separator();
                ui.label(format!(
                    "Name: {}",
                    self.image_name.as_deref().unwrap_or("(none)")
                ));
                if let Some([w, h]) = self.image_size {
                    ui.label(format!("Resolution: {}x{}", w, h));
                } else {
                    ui.label("Resolution: (none)");
                }

                ui.add_space(8.0);
                ui.heading("Grid");
                ui.separator();
                ui.label("Rows / Columns");
                ui.add(
                    egui::DragValue::new(&mut self.draft_grid.rows)
                        .speed(1)
                        .range(1..=4096),
                );
                ui.add(
                    egui::DragValue::new(&mut self.draft_grid.columns)
                        .speed(1)
                        .range(1..=4096),
                );
                ui.add_space(4.0);
                ui.label("Cell Size (0 = auto from rows/columns)");
                ui.add(
                    egui::DragValue::new(&mut self.draft_grid.cell_width)
                        .speed(1)
                        .range(0..=16384),
                );
                ui.add(
                    egui::DragValue::new(&mut self.draft_grid.cell_height)
                        .speed(1)
                        .range(0..=16384),
                );
                ui.add_space(4.0);
                ui.label("Offset");
                ui.add(
                    egui::DragValue::new(&mut self.draft_grid.offset_x)
                        .speed(1)
                        .range(0..=16384),
                );
                ui.add(
                    egui::DragValue::new(&mut self.draft_grid.offset_y)
                        .speed(1)
                        .range(0..=16384),
                );

                if ui.button("Apply / Regenerate Grid").clicked() {
                    self.applied_grid = self.draft_grid;
                }

                ui.add_space(8.0);
                ui.heading("Canvas View");
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("-").clicked() {
                        self.zoom_level = (self.zoom_level * 0.9).clamp(0.1, 16.0);
                    }
                    if ui.button("+").clicked() {
                        self.zoom_level = (self.zoom_level * 1.1).clamp(0.1, 16.0);
                    }
                    if ui.button("Reset View").clicked() {
                        self.zoom_level = 1.0;
                        self.canvas_pan = Vec2::ZERO;
                    }
                });
                ui.label(format!("Zoom: {:.0}%", self.zoom_level * 100.0));
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Canvas");
            ui.separator();

            let canvas_size = ui.available_size();
            let (response, painter) = ui.allocate_painter(canvas_size, egui::Sense::drag());

            if response.dragged() {
                let drag_delta = ui.ctx().input(|input| input.pointer.delta());
                self.canvas_pan += drag_delta;
            }

            if response.hovered() {
                let scroll_delta = ui.ctx().input(|input| input.smooth_scroll_delta.y);
                if scroll_delta.abs() > f32::EPSILON {
                    let zoom_factor = (1.0 + (scroll_delta * 0.001)).clamp(0.5, 1.5);
                    self.zoom_level = (self.zoom_level * zoom_factor).clamp(0.1, 16.0);
                }
            }

            if let Some(texture) = &self.image_texture {
                let image_size = texture.size_vec2() * self.zoom_level;
                let top_left = response.rect.center() - (image_size * 0.5) + self.canvas_pan;
                let image_rect = Rect::from_min_size(top_left, image_size);

                painter.image(
                    texture.id(),
                    image_rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
                draw_grid_overlay(&painter, image_rect.min, image_size, self.applied_grid);
            } else {
                painter.text(
                    response.rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Open an image from File -> Open Image.",
                    FontId::new(16.0, FontFamily::Proportional),
                    Color32::LIGHT_GRAY,
                );
            }
        });
    }
}

fn apply_theme(ctx: &Context) {
    let mut style = (*ctx.style()).clone();
    let palette = AppPalette::default();

    style.visuals = Visuals::dark();
    style.visuals.window_fill = palette.panel_fill;
    style.visuals.panel_fill = palette.background_fill;
    style.visuals.extreme_bg_color = palette.widget_bg;
    style.visuals.faint_bg_color = palette.background_fill;
    style.visuals.code_bg_color = palette.widget_bg;
    style.visuals.selection.bg_fill = palette.selection_fill;
    style.visuals.selection.stroke = Stroke::new(1.0, palette.selection_stroke);

    style.visuals.widgets.noninteractive.bg_fill = palette.panel_fill;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, palette.text_primary);
    style.visuals.widgets.inactive.bg_fill = palette.widget_bg;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, palette.text_primary);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, palette.widget_stroke);
    style.visuals.widgets.hovered.bg_fill = palette.widget_hovered_bg;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, palette.text_primary);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, palette.accent);
    style.visuals.widgets.active.bg_fill = palette.widget_active_bg;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, palette.text_primary);
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.2, palette.accent);

    style.spacing.item_spacing = Vec2::new(10.0, 8.0);
    style.spacing.button_padding = Vec2::new(12.0, 8.0);
    style.spacing.interact_size = Vec2::new(0.0, 30.0);
    style.spacing.indent = 16.0;

    style.text_styles.insert(
        egui::TextStyle::Heading,
        FontId::new(24.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Body,
        FontId::new(16.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        FontId::new(15.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        FontId::new(14.0, FontFamily::Monospace),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        FontId::new(13.0, FontFamily::Proportional),
    );

    ctx.set_style(style);
}

struct AppPalette {
    background_fill: Color32,
    panel_fill: Color32,
    widget_bg: Color32,
    widget_hovered_bg: Color32,
    widget_active_bg: Color32,
    widget_stroke: Color32,
    text_primary: Color32,
    accent: Color32,
    selection_fill: Color32,
    selection_stroke: Color32,
}

impl Default for AppPalette {
    fn default() -> Self {
        Self {
            background_fill: Color32::from_rgb(18, 22, 28),
            panel_fill: Color32::from_rgb(24, 30, 37),
            widget_bg: Color32::from_rgb(32, 40, 50),
            widget_hovered_bg: Color32::from_rgb(40, 50, 62),
            widget_active_bg: Color32::from_rgb(50, 64, 78),
            widget_stroke: Color32::from_rgb(62, 76, 93),
            text_primary: Color32::from_rgb(226, 232, 240),
            accent: Color32::from_rgb(0, 176, 168),
            selection_fill: Color32::from_rgba_unmultiplied(0, 176, 168, 80),
            selection_stroke: Color32::from_rgb(88, 234, 219),
        }
    }
}

fn draw_grid_overlay(painter: &egui::Painter, top_left: Pos2, image_size: Vec2, grid: GridConfig) {
    let width = image_size.x.max(1.0);
    let height = image_size.y.max(1.0);

    let rows = grid.rows.max(1);
    let columns = grid.columns.max(1);

    let auto_cell_w = width / columns as f32;
    let auto_cell_h = height / rows as f32;
    let cell_w = if grid.cell_width == 0 {
        auto_cell_w
    } else {
        grid.cell_width as f32
    };
    let cell_h = if grid.cell_height == 0 {
        auto_cell_h
    } else {
        grid.cell_height as f32
    };

    let offset_x = grid.offset_x as f32;
    let offset_y = grid.offset_y as f32;

    let stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(0, 255, 255, 180));
    let right = top_left.x + width;
    let bottom = top_left.y + height;

    for c in 0..=columns {
        let x = top_left.x + offset_x + (c as f32 * cell_w);
        if x >= top_left.x && x <= right {
            painter.line_segment([Pos2::new(x, top_left.y), Pos2::new(x, bottom)], stroke);
        }
    }

    for r in 0..=rows {
        let y = top_left.y + offset_y + (r as f32 * cell_h);
        if y >= top_left.y && y <= bottom {
            painter.line_segment([Pos2::new(top_left.x, y), Pos2::new(right, y)], stroke);
        }
    }
}

impl AssetHanderApp {
    fn open_image(&mut self, ctx: &Context) {
        let file = rfd::FileDialog::new()
            .add_filter("Image", &["png", "jpg", "jpeg", "bmp", "gif"])
            .pick_file();
        let Some(path) = file else {
            return;
        };

        match load_color_image(&path) {
            Ok(image) => {
                let size = image.size;
                self.image_size = Some(size);
                self.image_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(ToOwned::to_owned);
                self.image_texture =
                    Some(ctx.load_texture("uploaded_image", image, TextureOptions::LINEAR));
            }
            Err(error) => {
                self.image_name = Some(format!("Failed to load image: {}", error));
                self.image_size = None;
                self.image_texture = None;
            }
        }
    }
}

fn load_color_image(path: &Path) -> Result<ColorImage, String> {
    let image = image::ImageReader::open(path)
        .map_err(|e| format!("open error: {}", e))?
        .decode()
        .map_err(|e| format!("decode error: {}", e))?;
    let rgba = image.to_rgba8();
    let (w, h) = rgba.dimensions();
    let pixels = rgba.into_raw();
    Ok(ColorImage::from_rgba_unmultiplied(
        [w as usize, h as usize],
        &pixels,
    ))
}
