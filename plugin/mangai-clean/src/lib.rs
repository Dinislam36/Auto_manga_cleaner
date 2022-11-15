use std::process::abort;
use ps_sdk_sys::FilterRecord;

#[no_mangle]
pub extern "C" fn PluginMain(selector: u16, filter_param_block: &mut FilterRecord, data: *mut u8, result: &mut u16) {
    let _ = msgbox::create("Hello, world!", "Hello, world!", msgbox::IconType::Info);
    let _ = msgbox::create("FilterRecord", &format!("{:?}", filter_param_block), msgbox::IconType::Info);
    abort();
}