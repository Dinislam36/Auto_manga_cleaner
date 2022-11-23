use std::sync::Once;

static ONCE: Once = Once::new();

pub fn setup_console() {
    ONCE.call_once(|| {
        // ignore the result, as the console may already be allocated
        unsafe { windows::Win32::System::Console::AllocConsole() };
    });
}
