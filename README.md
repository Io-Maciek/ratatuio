
# ratatuio

**WARNING: This is a very alpha alpha version. NOT for production.**
The API can (and will) change at any time without notice. Expect bugs.

This library expands on [ratatui](https://github.com/ratatui/ratatui) by abstracting main methods to different modules, which can be helpful in making modular applications with many views as well as using architectural patterns like MVVM.

Original crate: [ratatui](https://github.com/ratatui/ratatui) ([crates.io](https://crates.io/crates/ratatui))

## Install

```
cargo add ratatuio
```

## Minimal working example

```rust
use ratatuio::app::{App, Context};
use ratatuio::crossterm::event::KeyEvent;
use ratatuio::ratatui::Frame;
use ratatuio::view::View;
use std::io;

struct Hello;

impl View for Hello {
    fn render(&mut self, frame: &mut Frame) {
        frame.render_widget("Hello World! Press any key to quit.", frame.area());
    }

    fn on_key(&mut self, _: KeyEvent, ctx: &mut Context) -> io::Result<()> {
        ctx.quit();
        Ok(())
    }
}

fn main() -> io::Result<()> {
    App::new(Hello).run()
}
```

Only `render()` is required in a `View`. The `on_key()` is here because the terminal is in raw mode, so without it there would be no way to close the application.

## How it works

Create a struct for each screen and implement the `View` trait. Give the first one to `App` and run it. Views can talk back to the application with `Context`.

### App

- `App::new()` - Creates the application with the starting view.
- `App::tick_rate()` - Sets how often `update()` runs. Default is 16 ms.
- `App::mouse()` - Enables mouse events for `on_mouse()`. Off by default.
- `App::paste()` - Enables paste events for `on_paste()`. Off by default.
- `App::run()` - Starts the main loop. The terminal is always restored on exit, also on error.

### Context

- `Context::change_view()` - Changes the current view to the new one. If called many times in one frame, the last one wins.
- `Context::quit()` - Stops the main loop. Takes precedence over `change_view()`.

### View

- `render()` - Draws the view. You get the whole ratatui `Frame`, so stateful widgets and the cursor work as usual.
- `on_key()` - Called on key press. Key release events are filtered out.
- `on_mouse()` - Called on mouse events. NOTE: Needs `App::mouse(true)`.
- `on_resize()` - Called when the terminal is resized.
- `on_paste()` - Called on pasted text. NOTE: Needs `App::paste(true)`.
- `handle_event()` - Gets the raw event. The default one calls the methods above. Override it only if you need full control.
- `update()` - Runs each tick. You get `dt` since the previous one.
- `on_appear()` - Runs once, right before the view is drawn for the first time.
- `on_disappear()` - Runs once, right before the view is replaced or the application exits.

Views live on one thread, so they do NOT need to be `Send` or `Sync`. You can use `Rc<RefCell<_>>` and friends.

## Examples

Examples are in the `examples` folder. To run one, go to its directory and use `cargo run`.

- `examples/simple` - A few views with navigation between them.
- `examples/measure` - A system monitor in the style of s-tui. Shows shared state between views, `update(dt)` and mouse support.

```
cd examples/simple
cargo run
```

## Notes

- It uses [crossterm](https://github.com/crossterm-rs/crossterm) under the hood, so it should work on Linux, macOS and Windows. For now it was only tested on Linux.
- Use `ratatuio::crossterm` and `ratatuio::ratatui` instead of adding your own versions. Two different versions of crossterm in one project can cause lost events.
- There is no support for other ratatui backends (termion, termwiz).

## License

See [LICENSE](LICENSE).
