use wordlehelper::ui::tui::MainWindow;

fn main() {
    let mut main_window: MainWindow<5, 6> = MainWindow::init();
    main_window.run_main_loop();
}
