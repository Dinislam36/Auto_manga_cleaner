use super::{BATCH_HEIGHT, BATCH_WIDTH, MODEL_INPUT_SHAPE, MODEL_OUTPUT_SHAPE, THRESHOLD};
use anyhow::Result;
use ndarray::prelude::*;
use ndarray::Zip;
use ndarray_vision::morphology::MorphologyExt;
use tract_onnx::prelude::*;

type OnnxModel = RunnableModel<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

pub struct Model {
    model: OnnxModel,
}

impl Model {
    pub fn new_from_bytes<B: prost::bytes::Buf>(bytes: B) -> Result<Self> {
        use prost::Message;
        use tract_onnx::pb::ModelProto;

        let model = ModelProto::decode(bytes)?;
        let model = onnx()
            .model_for_proto_model(&model)?
            .into_optimized()? // TODO: this can clearly be done at compile time
            .into_runnable()?;

        Ok(Self { model })
    }

    pub fn clean_one_batch(&self, image_in: ArrayView3<u8>, mut mask_out: ArrayViewMut2<bool>) {
        let mut image_buf = Array3::zeros(image_in.dim().into_shape());
        // image_in.mapv()
        // let image_in = image_in.;
        Zip::from(&mut image_buf).and(image_in).for_each(|a, b| {
            let f = *b as f32 / 255.0;
            // normalize
            *a = (f - 0.5) / 0.5;
        });

        assert_eq!(image_in.shape(), &MODEL_INPUT_SHAPE[1..]);
        assert_eq!(mask_out.shape(), &MODEL_INPUT_SHAPE[2..]);

        let image_buf = image_buf
            .into_shape(MODEL_INPUT_SHAPE)
            .unwrap()
            .into_owned(); // we
        let image_buf = image_buf.into();

        let model_output = self.model.run(tvec!(image_buf)).unwrap();
        let model_output = model_output.into_iter().next().unwrap();
        let model_output = Arc::try_unwrap(model_output)
            .unwrap()
            .into_array::<f32>()
            .unwrap()
            .into_shape(MODEL_OUTPUT_SHAPE)
            .unwrap();
        let model_output = model_output
            .into_shape((1, BATCH_HEIGHT, BATCH_WIDTH))
            .unwrap();
        let mut mask = model_output.mapv(|x| x > THRESHOLD);

        let kern = arr2(&[[true, true, true], [true, true, true], [true, true, true]]);

        let mut dilating_mask = mask.view_mut();
        dilating_mask.swap_axes(0, 2);
        dilating_mask.dilate_inplace(kern.view());
        dilating_mask.dilate_inplace(kern.view());

        let mask = mask.into_shape((BATCH_HEIGHT, BATCH_WIDTH)).unwrap();

        // perform OR operation on intersecting areas
        // it's not really clear how this affects the result, but let's try it

        Zip::from(&mut mask_out).and(&mask).for_each(|a, &b| {
            if b {
                *a = true;
            }
        });
    }
}
