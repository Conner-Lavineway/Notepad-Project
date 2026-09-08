use crate::markdownreformatter::Reformatter;
use eframe::egui::{self, Color32, FontId, FontFamily};
use std::path::PathBuf;




static PIXEL_POINT: f32 = 1.5; //Default textsize
static BACKGROUND_COLOR: Color32 = Color32::from_rgb(27, 27, 27);
static DEFAULT_ROWS: usize = 100; //starting rows of text editor, affects size of editing area
static FONT_STYLE: FontId = FontId::new(12.0, FontFamily::Proportional);
static FONT_COLOR: Color32 = Color32::WHITE;


pub struct TextEditor {
    notepad: String,
    reformatter: Reformatter,
    file_path: Option<PathBuf>,
    status: String,
}

impl Default for TextEditor {
    fn default() -> Self {
        Self {
            notepad: String::new(), 
            reformatter: Reformatter::new(),
            file_path: None,
            status: "Ready".to_string(),
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
}

impl eframe::App for TextEditor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(PIXEL_POINT);

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
            let mut render_layer = |ui: &egui::Ui, text: &str, wrap_width: f32| {
                if self.reformatter.needs_reformat(text) {
                    self.reformatter.reformat(text, FONT_COLOR, FONT_STYLE.clone());
                }
                let mut job = self.reformatter.formatted().clone();
                job.wrap.max_width = wrap_width;
                ui.fonts(|fonts| {
                    fonts.layout_job(job)
                })
            };
            

            //main text editor
            egui::ScrollArea::vertical().show(ui,|ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.notepad)
                    .desired_width(f32::INFINITY)
                    .desired_rows(DEFAULT_ROWS)
                    .background_color(BACKGROUND_COLOR)
                    .layouter(&mut render_layer)
                );
            });

        });

    }
}
