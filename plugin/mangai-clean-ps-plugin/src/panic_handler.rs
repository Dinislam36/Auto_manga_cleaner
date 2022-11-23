use std::sync::Once;

static ONCE: Once = Once::new();

pub fn setup_panic() {
    ONCE.call_once(|| {
        std::panic::set_hook(Box::new(|panic_info| {
            let mut message = String::new();
            if let Some(location) = panic_info.location() {
                message.push_str(&format!("{}:{}: ", location.file(), location.line()));
            }
            if let Some(payload) = panic_info.payload().downcast_ref::<&str>() {
                message.push_str(payload);
            } else if let Some(payload) = panic_info.payload().downcast_ref::<String>() {
                message.push_str(payload);
            } else {
                message.push_str("unknown panic");
            }
            msgbox::create("Panic", &message, msgbox::IconType::Error).unwrap();
        }));
    });
}
