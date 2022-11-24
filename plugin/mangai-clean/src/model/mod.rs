pub const BATCH_WIDTH: usize = 828;
pub const BATCH_HEIGHT: usize = 1176;
pub const MODEL_INPUT_SHAPE: [usize; 4] = [1, 3, BATCH_HEIGHT, BATCH_WIDTH];
pub const MODEL_OUTPUT_SHAPE: [usize; 4] = [1, 1, BATCH_HEIGHT, BATCH_WIDTH];

pub const THRESHOLD: f32 = 0.0005;

mod tract;

pub use tract::Model;
