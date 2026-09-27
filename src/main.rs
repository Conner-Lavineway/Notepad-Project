mod editor;
mod markdownreformatter;
use editor::TextEditor;

const PIXEL_POINT: f32 = 1.5; //Default textsize

fn main() -> eframe::Result {
    let options = eframe::NativeOptions{
        viewport: eframe::egui::ViewportBuilder::default()
        .with_inner_size([1000.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Conner's Notepad", 
        options,
        Box::new(|cc| {
        cc.egui_ctx.set_pixels_per_point(PIXEL_POINT);
        Ok(Box::new(TextEditor::default()))
    }))
}