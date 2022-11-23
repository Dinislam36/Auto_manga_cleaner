use anyhow::Result;
use ndarray::ShapeBuilder;
use ndarray::Zip;
use ndarray_vision::morphology::MorphologyExt;
use tract_onnx::prelude::tract_ndarray as ndarray;
use tract_onnx::prelude::*;

// TODO: add a mechanism to download a model during build time or smth
// otherwise building becomes a pain
// const MODEL_ONNX: &'static [u8] = include_bytes!("../model.onnx");

// TODO: this will not work, lol
const MODEL_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/model.onnx");

type OnnxModel = RunnableModel<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

const IMAGE_WIDTH: usize = 828;
const IMAGE_HEIGHT: usize = 1176;
const MODEL_INPUT_SHAPE: [usize; 4] = [1, 3, IMAGE_HEIGHT, IMAGE_WIDTH];
const MODEL_OUTPUT_SHAPE: [usize; 4] = [1, 1, IMAGE_HEIGHT, IMAGE_WIDTH];

const THRESHOLD: f32 = 0.0005;

pub struct MangaiClean {
    model: OnnxModel,
}

// fn

impl MangaiClean {
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

    pub fn new() -> Result<Self> {
        let bytes = std::fs::read(MODEL_PATH)?;
        Self::new_from_bytes(bytes.as_ref())
    }

    pub fn clean_page(
        &self,
        image_in: ndarray::ArrayView3<u8>,
        mut image_out: ndarray::ArrayViewMut3<u8>,
    ) {
        let mut image_buf = ndarray::Array3::zeros(image_in.dim().into_shape());
        // image_in.mapv()
        // let image_in = image_in.;
        Zip::from(&mut image_buf).and(image_in).for_each(|a, b| {
            let f = *b as f32 / 255.0;
            // normalize
            *a = (f - 0.5) / 0.5;
        });

        // TODO: support other sizes
        assert_eq!(image_in.shape(), &MODEL_INPUT_SHAPE[1..]);
        assert_eq!(image_out.shape(), &MODEL_INPUT_SHAPE[1..]);

        let image_buf = image_buf.into_shape(MODEL_INPUT_SHAPE).unwrap(); // we

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
            .into_shape((1, IMAGE_HEIGHT, IMAGE_WIDTH))
            .unwrap();
        let mut mask = model_output.mapv(|x| x > THRESHOLD);

        let kern = ndarray::arr2(&[[true, true, true], [true, true, true], [true, true, true]]);

        let mut dilating_mask = mask.view_mut();
        dilating_mask.swap_axes(0, 2);
        dilating_mask.dilate_inplace(kern.view());
        dilating_mask.dilate_inplace(kern.view());

        let mask = mask.into_shape((IMAGE_HEIGHT, IMAGE_WIDTH)).unwrap();

        // dbg!(image_in.shape());
        // dbg!(mask.shape());
        // dbg!(image_out.shape());

        Zip::from(image_in)
            .and(mask.broadcast(image_in.dim()).unwrap())
            .and(&mut image_out)
            .for_each(|&value, &mask, out| *out = if mask { 255 } else { value });
    }
}
