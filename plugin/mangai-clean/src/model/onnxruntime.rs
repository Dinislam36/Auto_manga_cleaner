use super::MODEL_OUTPUT_SHAPE;
use anyhow::Result;
use ndarray::prelude::*;
use once_cell::sync::Lazy;
use onnxruntime::environment::Environment;
use onnxruntime::session::{AnyArray, Session};
use onnxruntime::GraphOptimizationLevel;
use std::ops::Deref;
use std::sync::Mutex;

static ENVIRONMENT: Lazy<Environment> = Lazy::new(|| {
    Environment::builder()
        .with_name("mangai")
        .with_log_level(onnxruntime::LoggingLevel::Warning)
        .build()
        .unwrap()
});

pub struct Model {
    session: Mutex<Session<'static>>,
}

impl Model {
    pub fn new_from_bytes<B: AsRef<[u8]>>(bytes: B) -> Result<Self> {
        let session = ENVIRONMENT
            .new_session_builder()?
            .with_optimization_level(GraphOptimizationLevel::All)?
            // TODO: threads??
            // .with_intra_op_num_threads(1)?
            .with_model_from_memory(bytes)?;
        let session = Mutex::new(session);

        Ok(Self { session })
    }

    pub fn run_model(&self, image_buf: Array4<f32>) -> Array4<f32> {
        let mut image_buf = onnxruntime::session::NdArray::new(image_buf);
        let input_tensor: Vec<&mut dyn AnyArray> = vec![&mut image_buf];

        let model_output = {
            let mut session = self.session.lock().unwrap();
            let model_output = session.run(input_tensor).unwrap();

            let model_output = model_output.into_iter().next().unwrap().deref().to_owned();
            model_output
        };
        model_output.into_shape(MODEL_OUTPUT_SHAPE).unwrap()
    }
}
