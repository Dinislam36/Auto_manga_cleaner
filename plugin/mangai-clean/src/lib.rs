use crate::model::{BATCH_HEIGHT, BATCH_WIDTH, MODEL_INPUT_SHAPE, THRESHOLD};
use anyhow::Result;
use ndarray::{
    s, Array2, Array3, ArrayView2, ArrayView3, ArrayViewMut2, CowArray, ShapeBuilder, SliceInfo,
};
use ndarray::{ArrayViewMut3, Zip};
use ndarray_vision::morphology::MorphologyExt;
use std::ops::Deref;
use tracing::info;

mod batcher;
mod model;
mod model_registry;

pub enum ProgressKind {
    Items,
    Bytes,
}

pub trait ProgressReporter {
    fn init(&mut self, kind: ProgressKind, operation: &str, total: usize);
    fn progress(&mut self, progress: usize);
    fn finish(&mut self);
}

pub struct MangaiClean {
    model: model::Model,
}

impl MangaiClean {
    pub fn new_from_bytes<B: AsRef<[u8]>>(bytes: B) -> Result<Self> {
        let model = model::Model::new_from_bytes(bytes)?;
        Ok(Self { model })
    }

    pub fn new(progress: &mut dyn ProgressReporter) -> Result<Self> {
        let bytes = model_registry::get_model(progress)?;
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
        image_in: ArrayView3<u8>,
        mut image_out: ArrayViewMut3<u8>,
        progress_reporter: &mut dyn ProgressReporter,
    ) {
        assert_eq!(image_in.dim(), image_out.dim());

        let (channels, orig_height, orig_width) = image_in.dim();
        assert_eq!(channels, 3);

        // pad the image if it's too small
        let (image_in, height, width) = if orig_height < BATCH_HEIGHT || orig_width < BATCH_WIDTH {
            info!(
                "Padding the image to fit the batch size ({}x{})",
                BATCH_WIDTH, BATCH_HEIGHT
            );
            let mut padded_image = Array3::from_elem((3, BATCH_HEIGHT, BATCH_WIDTH), 255u8);
            padded_image
                .slice_mut(s![.., ..orig_height, ..orig_width])
                .assign(&image_in);

            (CowArray::from(padded_image), BATCH_HEIGHT, BATCH_WIDTH)
        } else {
            (CowArray::from(image_in), orig_height, orig_width)
        };

        let mut mask = ndarray::Array2::from_shape_fn((height, width), |_| false);

        let batcher = batcher::Batcher::new(height, width);
        progress_reporter.init(ProgressKind::Items, "Cleaning manga", batcher.num_batches());
        for (i, slice) in batcher.iter().enumerate() {
            progress_reporter.progress(i);
            info!("Processing batch #{}/{}", i + 1, batcher.num_batches());

            let image_in = image_in.slice(slice);
            let mask_slice = [slice.deref()[1], slice.deref()[2]];
            let mask_slice = SliceInfo::try_from(mask_slice).unwrap();

            let mask_out = mask.slice_mut(mask_slice);

            self.clean_one_batch(image_in, mask_out);
        }

        progress_reporter.finish();

        // slice the image to undo the padding
        let image_in = image_in.slice(s![.., ..orig_height, ..orig_width]);
        let mask = mask.slice(s![..orig_height, ..orig_width]);

        Zip::from(&mut image_out)
            .and(mask.broadcast(image_in.dim()).unwrap())
            .and(&image_in)
            .for_each(|out, &mask, &value| {
                if mask {
                    *out = 255;
                } else {
                    *out = value;
                }
            });
    }

    pub fn clean_grayscale_page(
        &self,
        image_in: ArrayView2<u8>,
        mut image_out: ArrayViewMut2<u8>,
        progress_reporter: &mut dyn ProgressReporter,
    ) {
        assert_eq!(image_in.dim(), image_out.dim());

        let (orig_height, orig_width) = image_in.dim();

        // pad the image if it's too small
        let (image_in, height, width) = if orig_height < BATCH_HEIGHT || orig_width < BATCH_WIDTH {
            info!(
                "Padding the image to fit the batch size ({}x{})",
                BATCH_WIDTH, BATCH_HEIGHT
            );
            let mut padded_image = Array2::from_elem((BATCH_HEIGHT, BATCH_WIDTH), 255u8);
            padded_image
                .slice_mut(s![..orig_height, ..orig_width])
                .assign(&image_in);

            (CowArray::from(padded_image), BATCH_HEIGHT, BATCH_WIDTH)
        } else {
            (CowArray::from(image_in), orig_height, orig_width)
        };

        let mut mask = ndarray::Array2::from_shape_fn((height, width), |_| false);

        let batcher = batcher::Batcher::new(height, width);
        progress_reporter.init(ProgressKind::Items, "Cleaning Manga", batcher.num_batches());
        for (i, slice) in batcher.iter().enumerate() {
            progress_reporter.progress(i);
            info!("Processing batch #{}/{}", i + 1, batcher.num_batches());

            let image_in = image_in.broadcast((3, height, width)).unwrap();
            let image_in = image_in.slice(slice);

            let mask_slice = [slice.deref()[1], slice.deref()[2]];
            let mask_slice = SliceInfo::try_from(mask_slice).unwrap();

            let mask_out = mask.slice_mut(mask_slice);

            self.clean_one_batch(image_in, mask_out);
        }

        progress_reporter.progress(batcher.num_batches());

        // slice the image to undo the padding
        let image_in = image_in.slice(s![..orig_height, ..orig_width]);
        let mask = mask.slice(s![..orig_height, ..orig_width]);

        Zip::from(&mut image_out)
            .and(mask)
            .and(&image_in)
            .for_each(|out, &mask, &value| {
                if mask {
                    *out = 255;
                } else {
                    *out = value;
                }
            });
    }
}
