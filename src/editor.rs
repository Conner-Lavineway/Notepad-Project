use crate::markdownreformatter::Reformatter;
use eframe::egui::{self, Color32, FontFamily, FontId, Galley, Key, KeyboardShortcut, Modifiers, TextureHandle};
use std::{collections::HashMap, path::PathBuf};




static PIXEL_POINT: f32 = 1.5; //Default textsize
static BACKGROUND_COLOR: Color32 = Color32::from_rgb(27, 27, 27);
static DEFAULT_ROWS: usize = 10; //starting rows of text editor, affects size of editing area
static FONT_STYLE: FontId = FontId::new(12.0, FontFamily::Proportional);
static FONT_COLOR: Color32 = Color32::WHITE;


pub struct TextEditor {
    notepad: String,
    reformatter: Reformatter,
    file_path: Option<PathBuf>,
    status: String,
    image_cache: HashMap<String, (TextureHandle, egui::Vec2)>,
}

impl Default for TextEditor {
    fn default() -> Self {
        Self {
            notepad: String::new(), 
            reformatter: Reformatter::new(),
            file_path: None,
            status: "Ready".to_string(),
            image_cache: HashMap::new(),
        }
    }
}

impl TextEditor {
    fn open_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text", &["txt", "md"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(text) => {
                    self.notepad = text;
                    self.status = format!("Opened: {}", path.display());
                    self.file_path = Some(path);
                }
                Err(e) => {
                    self.status = format!("Error: {}", e);
                }
            }
        }
    }

    fn save_file(&mut self) {
        if let Some(path) = &self.file_path {
            match std::fs::write(path, &self.notepad) {
                Ok(()) => {
                    self.status = format!("Saved: {}", path.display());
                }
                Err(e) => {
                    self.status = format!("Error: {}", e);
                }
            }
        }
    }

    fn save_file_as(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text", &["txt", "md"])
            .save_file()
        {
            match std::fs::write(&path, &self.notepad) {
                Ok(()) => {
                    self.status = format!("Saved: {}", path.display());
                    self.file_path = Some(path);
                }
                Err(e) => {
                    self.status = format!("Error: {}", e);
                }
            }
        }
    }

    fn new_file(&mut self) {
        self.notepad.clear();
        self.file_path = None;
        self.status = "New File".to_string();
    }


    fn load_image(&mut self, ctx: &egui::Context, path: &str) -> Option<(TextureHandle, egui::Vec2)> {
        if let Some(tex) = self.image_cache.get(path) {
            return Some(tex.clone());
        }
        let img = image::open(path).ok()?.to_rgba8();
        let (w, h) = img.dimensions();

        let color_image = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &img);
        let tex = ctx.load_texture(path, color_image, egui::TextureOptions::default());
        let size = egui::vec2(w as f32, h as f32);
        self.image_cache.insert(path.to_string(), (tex.clone(), size));
        Some((tex, size))
    }

    fn find_row(galley: &Galley, target_line: usize) -> Option<usize> {
        let mut current_line = 0;

        for (row_index, row) in galley.rows.iter().enumerate() {
            if current_line == target_line {
                return Some(row_index);
            }
            if row.ends_with_newline {
                current_line += 1;
            }
        }

        None
    }

    fn get_image_paths(text: &str) -> Vec<String> {
        let mut paths = Vec::new();
        let mut rest = text;

        while let Some(bracket) = rest.find("![") {
            let after_bracket = &rest[&bracket+2 ..];
            let Some(bracket_end) = after_bracket.find("]") else { break; };
            let after_bracket = &after_bracket[bracket_end + 1..];

            if after_bracket.starts_with('(') {
                if let Some(paren_end) = after_bracket.find(')') {
                    paths.push(after_bracket[1..paren_end].to_string());
                    rest = &after_bracket[paren_end + 1..];
                    continue;
                }
            }                 
            rest = after_bracket;

        }

        paths
    }
}

impl eframe::App for TextEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(PIXEL_POINT);

        for path in Self::get_image_paths(&self.notepad) {
            if let Some((_, size)) = self.load_image(ctx, &path) {
                self.reformatter.set_image_size(&path, size);
            }
        }

        egui::TopBottomPanel::top("ToolBar").show(ctx, |ui|{
            ui.horizontal(|ui| {
                if ui.button("New").clicked() {
                    self.new_file();
                }
                if ui.button("Open").clicked() {
                    self.open_file();
                }
                if ui.button("Save").clicked() {
                    self.save_file();
                }
                if ui.button("Save As").clicked() {
                    self.save_file_as();
                }

                ui.separator();

                if let Some(path) = &self.file_path {
                    ui.label(
                        egui::RichText::new(
                            path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default()
                        )
                        .strong(),
                    );
                } else {
                    ui.label("Unititled");
                }
            });
        });

        egui::TopBottomPanel::bottom("Status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label(format!("{} chars", self.notepad.len()));
                    },
                );
            });
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {

            //text edit render control
            egui::ScrollArea::vertical().show(ui, |ui| {
                let output = {
                    let mut render_layer = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                        if self.reformatter.needs_reformat(text) {
                            self.reformatter.reformat(
                                text,
                                FONT_COLOR,
                                FONT_STYLE.clone(),
                            );
                        }

                        let mut job = self.reformatter.formatted().clone();
                        job.wrap.max_width = wrap_width;

                        ui.fonts(|fonts| {
                            fonts.layout_job(job)
                        })
                    };

                    egui::TextEdit::multiline(&mut self.notepad)
                        .desired_width(f32::INFINITY)
                        .desired_rows(DEFAULT_ROWS)
                        .background_color(BACKGROUND_COLOR)
                        .layouter(&mut render_layer)
                        .show(ui)
                };

                let galley_pos = output.text_clip_rect.min;
                
                for(line, path) in self.reformatter.image_lines() {
                    let Some(row_index) = Self::find_row(&output.galley, line) else {continue;};
                    let row = &output.galley.rows[row_index];

                    if let Some((texture, size)) = self.load_image(ctx, &path) {
                        let scale = (crate::markdownreformatter::MAX_IMAGE_WIDTH / size.x).min(1.0);
                        let display_size = size * scale;

                        let rect = egui::Rect::from_min_size(
                            galley_pos + row.rect.min.to_vec2(), 
                            display_size,
                        );

                        ui.painter().image(
                            texture.id(), 
                            rect, 
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), 
                            Color32::WHITE
                        );
                    }
                }


                if let Some(cursor_range) = output.cursor_range {
                    let cursor_position = cursor_range.primary.ccursor.index;

                    self.reformatter.set_cursor_pos(cursor_position, FONT_COLOR, FONT_STYLE.clone());
                }
            });
            
            if ui.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL | Modifiers::SHIFT, Key::S))) {
                self.save_file_as();
            }
            if ui.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::S))) {
                self.save_file();
            }
            if ui.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::O))) {
                self.open_file();
            }
            if ui.input_mut(|i| i.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::N))) {
                self.new_file();
            }
        });

    }


}
