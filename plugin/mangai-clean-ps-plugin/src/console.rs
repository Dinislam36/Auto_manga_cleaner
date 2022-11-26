use std::sync::Once;

static ONCE: Once = Once::new();

pub fn setup_console() {
    ONCE.call_once(|| {
        // ignore the result, as the console may already be allocated
        // unsafe { windows::Win32::System::Console::AllocConsole() };

        // #[cfg(windows)]
        // ansi_term::enable_ansi_support().unwrap();

        tracing_subscriber::fmt::fmt().pretty().init();

        tracing::info!("Console initialized");
    });
}
