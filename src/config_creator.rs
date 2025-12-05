use update_manager::util;

fn main() {
    let _ = color_eyre::install();
    let terminal = ratatui::init();
    let _ = util::tui::tui::run(terminal);
    ratatui::restore();
}