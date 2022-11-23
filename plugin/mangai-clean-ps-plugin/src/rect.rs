#[derive(Debug, Clone, Copy)]
pub struct Rect {
    top: i32,
    left: i32,
    bottom: i32,
    right: i32,
}

impl Rect {
    pub fn empty() -> Self {
        Self {
            top: 0,
            left: 0,
            bottom: 0,
            right: 0,
        }
    }
}

impl From<ps_sdk_sys::VRect> for Rect {
    fn from(rect: ps_sdk_sys::VRect) -> Self {
        Self {
            top: rect.top,
            left: rect.left,
            bottom: rect.bottom,
            right: rect.right,
        }
    }
}

impl From<Rect> for ps_sdk_sys::VRect {
    fn from(rect: Rect) -> Self {
        Self {
            top: rect.top,
            left: rect.left,
            bottom: rect.bottom,
            right: rect.right,
        }
    }
}
