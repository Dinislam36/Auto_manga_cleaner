use anyhow::Result;
use ndarray::SliceInfo;
use ndarray::Zip;
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

            self.model.clean_one_batch(image_in, mask_out);
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
