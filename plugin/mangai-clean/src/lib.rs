use crate::model::{BATCH_HEIGHT, BATCH_WIDTH, MODEL_INPUT_SHAPE, THRESHOLD};
use anyhow::Result;
use ndarray::Zip;
use ndarray::{Array3, ArrayView3, ArrayViewMut2, ShapeBuilder, SliceInfo};
use ndarray_vision::morphology::MorphologyExt;
use std::ops::Deref;
use tracing::info;

mod batcher;
mod model;

// TODO: add a mechanism to download a model during build time or smth
// otherwise building becomes a pain
// const MODEL_ONNX: &'static [u8] = include_bytes!("../model.onnx");

// TODO: this will not work, lol
const MODEL_PATH: &'static str = concat!(env!("CARGO_MANIFEST_DIR"), "/model.onnx");

pub struct MangaiClean {
    model: model::Model,
}

impl MangaiClean {
    pub fn new_from_bytes<B: AsRef<[u8]>>(bytes: B) -> Result<Self> {
        let model = model::Model::new_from_bytes(bytes)?;
        Ok(Self { model })
    }

    pub fn new() -> Result<Self> {
        let bytes = std::fs::read(MODEL_PATH)?;
        Self::new_from_bytes(bytes)
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

        let model_output = self.model.run_model(image_buf);

        let model_output = model_output
            .into_shape((1, BATCH_HEIGHT, BATCH_WIDTH))
            .unwrap();
        let mut mask = model_output.mapv(|x: f32| x > THRESHOLD);

        let kern = ndarray::arr2(&[[true, true, true], [true, true, true], [true, true, true]]);

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

    pub fn clean_page(
        &self,
        image_in: ndarray::ArrayView3<u8>,
        mut image_out: ndarray::ArrayViewMut3<u8>,
    ) {
        assert_eq!(image_in.dim(), image_out.dim());

        let (channels, height, width) = image_in.dim();
        assert_eq!(channels, 3);

        if height < model::BATCH_HEIGHT || width < model::BATCH_WIDTH {
            todo!("handle small images");
        }

        let mut mask = ndarray::Array2::from_shape_fn((height, width), |_| false);

        let batcher = batcher::Batcher::new(height, width);
        for (i, slice) in batcher.iter().enumerate() {
            info!("Processing batch #{}/{}", i + 1, batcher.num_batches());

            let image_in = image_in.slice(slice);
            let mask_slice = [slice.deref()[1], slice.deref()[2]];
            let mask_slice = SliceInfo::try_from(mask_slice).unwrap();

            let mask_out = mask.slice_mut(mask_slice);

            self.clean_one_batch(image_in, mask_out);
        }

        Zip::from(&mut image_out)
            .and(mask.broadcast(image_in.dim()).unwrap())
            .and(image_in)
            .for_each(|out, &mask, &value| {
                if mask {
                    *out = 255;
                } else {
                    *out = value;
                }
            });
    }
}
