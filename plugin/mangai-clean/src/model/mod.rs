use cfg_if::cfg_if;

pub const BATCH_WIDTH: usize = 828;
pub const BATCH_HEIGHT: usize = 1176;
pub const MODEL_INPUT_SHAPE: [usize; 4] = [1, 3, BATCH_HEIGHT, BATCH_WIDTH];
pub const MODEL_OUTPUT_SHAPE: [usize; 4] = [1, 1, BATCH_HEIGHT, BATCH_WIDTH];

pub const THRESHOLD: f32 = 0.0005;

cfg_if!(
    if #[cfg(feature = "onnxruntime-backend")] {
        mod onnxruntime;
        pub use self::onnxruntime::*;
    } else if #[cfg(feature = "tract-backend")] {
        mod tract;
        pub use self::tract::*;
    } else {
        compile_error!("No backend selected. Please select one with --features onnxruntime-backend or --features tract-backend");
    }
);
