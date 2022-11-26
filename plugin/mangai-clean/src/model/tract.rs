use super::MODEL_OUTPUT_SHAPE;
use anyhow::Result;
use ndarray::prelude::*;
use std::io;
use tracing::info;
use tract_onnx::prelude::*;

type OnnxModel = RunnableModel<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

pub struct Model {
    model: OnnxModel,
}

impl Model {
    pub fn new_from_bytes<B: AsRef<[u8]>>(bytes: B) -> Result<Self> {
        use prost::Message;
        use tract_onnx::pb::ModelProto;

        info!("Creating inference session with tract");

        let bytes = io::Cursor::new(bytes.as_ref());
        let model = ModelProto::decode(bytes)?;
        let model = onnx()
            .model_for_proto_model(&model)?
            .into_optimized()? // TODO: this can clearly be done at compile time
            .into_runnable()?;

        Ok(Self { model })
    }

    pub fn run_model(&self, image_buf: Array4<f32>) -> Array4<f32> {
        let image_buf = image_buf.into();
        let model_output = self.model.run(tvec!(image_buf)).unwrap();
        let model_output = model_output.into_iter().next().unwrap();
        Arc::try_unwrap(model_output)
            .unwrap()
            .into_array::<f32>()
            .unwrap()
            .into_shape(MODEL_OUTPUT_SHAPE)
            .unwrap()
    }
}
