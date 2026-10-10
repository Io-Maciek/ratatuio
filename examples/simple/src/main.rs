mod event_log;
mod navigation;
mod views;

use ratatuio::app::App;
use std::io;
use views::ParagraphView;

fn main() -> io::Result<()> {
    App::new(ParagraphView::new())
        .mouse(true) // needed for View::on_mouse
        .paste(false) // needed for View::on_paste
        .run()
}