#[derive(Debug, Clone, Copy)]
pub struct Point {
    v: i32,
    h: i32,
}

impl From<ps_sdk_sys::VPoint> for Point {
    fn from(point: ps_sdk_sys::VPoint) -> Self {
        Self {
            v: point.v,
            h: point.h,
        }
    }
}

impl From<Point> for ps_sdk_sys::VPoint {
    fn from(point: Point) -> Self {
        Self {
            v: point.v,
            h: point.h,
        }
    }
}
