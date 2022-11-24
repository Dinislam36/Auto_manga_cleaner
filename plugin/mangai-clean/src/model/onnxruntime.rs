use super::{BATCH_HEIGHT, BATCH_WIDTH, MODEL_INPUT_SHAPE, MODEL_OUTPUT_SHAPE, THRESHOLD};
use anyhow::Result;
use ndarray::prelude::*;
use ndarray::Zip;
use ndarray_vision::morphology::MorphologyExt;
use once_cell::sync::Lazy;
use onnxruntime;
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

    pub fn clean_one_batch(&self, image_in: ArrayView3<u8>, mut mask_out: ArrayViewMut2<bool>) {
        let mut image_buf = Array3::zeros(image_in.dim().into_shape());
        // TODO: most of this code can be shared with the tract version
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
            .into_owned();

        let mut image_buf = onnxruntime::session::NdArray::new(image_buf);

        let input_tensor: Vec<&mut dyn AnyArray> = vec![&mut image_buf];

        let model_output = {
            let mut session = self.session.lock().unwrap();
            let model_output = session.run(input_tensor).unwrap();

            let model_output = model_output.into_iter().next().unwrap().deref().to_owned();
            model_output
        };
        let model_output = model_output.into_shape(MODEL_OUTPUT_SHAPE).unwrap();
        let model_output = model_output
            .into_shape((1, BATCH_HEIGHT, BATCH_WIDTH))
            .unwrap();
        let mut mask = model_output.mapv(|x: f32| x > THRESHOLD);

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
