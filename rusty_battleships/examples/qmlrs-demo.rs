#[macro_use]
extern crate qmlrs;

static WINDOW: &'static str = include_str!("assets/main_window.qml");

fn main() {
    let mut engine = qmlrs::Engine::new();

    engine.load_data(WINDOW);

    engine.exec();
}
