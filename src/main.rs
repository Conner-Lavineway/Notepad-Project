mod editor;
mod markdownreformatter;
use editor::TextEditor;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions{
        viewport: eframe::egui::ViewportBuilder::default()
        .with_inner_size([1000.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Conner's Notepad", 
        options,
        Box::new(|_cc| Ok(Box::new(TextEditor::default()))))
}
