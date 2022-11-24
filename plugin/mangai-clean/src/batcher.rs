use crate::model::{BATCH_HEIGHT, BATCH_WIDTH};
use ndarray::{s, Ix3, SliceInfo, SliceInfoElem};

#[derive(Debug, Clone, Copy)]
pub struct Batcher {
    v_batches: usize,
    h_batches: usize,
    v_offset: f64,
    h_offset: f64,
}

impl Batcher {
    pub fn new(height: usize, width: usize) -> Self {
        assert!(height >= BATCH_HEIGHT);
        assert!(width >= BATCH_WIDTH);

        let v_batches = (height + BATCH_HEIGHT - 1) / BATCH_HEIGHT;
        let h_batches = (width + BATCH_WIDTH - 1) / BATCH_WIDTH;

        fn map_inf_to_zero(x: f64) -> f64 {
            if x.is_infinite() || x.is_nan() {
                0.0
            } else {
                x
            }
        }

        let v_offset = map_inf_to_zero((height - BATCH_HEIGHT) as f64 / (v_batches - 1) as f64);
        let h_offset = map_inf_to_zero((width - BATCH_WIDTH) as f64 / (h_batches - 1) as f64);

        Self {
            v_batches,
            h_batches,
            v_offset,
            h_offset,
        }
    }

    pub fn iter(&self) -> BatcherIter {
        BatcherIter {
            batcher: *self,
            v: 0,
            h: 0,
        }
    }

    pub fn num_batches(&self) -> usize {
        self.v_batches * self.h_batches
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BatcherIter {
    batcher: Batcher,
    v: usize,
    h: usize,
}

impl Iterator for BatcherIter {
    type Item = SliceInfo<[SliceInfoElem; 3], Ix3, Ix3>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.v >= self.batcher.v_batches {
            return None;
        }

        let v_start = self.v as f64 * self.batcher.v_offset;
        let v_end = v_start + BATCH_HEIGHT as f64;
        let h_start = self.h as f64 * self.batcher.h_offset;
        let h_end = h_start + BATCH_WIDTH as f64;

        self.h += 1;
        if self.h >= self.batcher.h_batches {
            self.h = 0;
            self.v += 1;
        }

        Some(s![
            ..,
            v_start as usize..v_end as usize,
            h_start as usize..h_end as usize
        ])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.batcher.v_batches - self.v) * self.batcher.h_batches - self.h;
        (remaining, Some(remaining))
    }
}

#[cfg(test)]
mod test {
    use crate::ndarray;
    use ndarray::{Ix3, SliceInfo};
    use std::ops::Deref;

    fn deref_slice_info<T: Clone>(info: SliceInfo<T, Ix3, Ix3>) -> T {
        info.deref().clone()
    }

    #[test]
    fn test_batcher_single() {
        use super::*;

        let batcher = Batcher::new(BATCH_HEIGHT, BATCH_WIDTH);
        assert_eq!(batcher.v_batches, 1);
        assert_eq!(batcher.h_batches, 1);
        assert_eq!(batcher.v_offset, 0.0);
        assert_eq!(batcher.h_offset, 0.0);

        let slices = batcher.iter().map(deref_slice_info).collect::<Vec<_>>();
        assert_eq!(
            &slices,
            &[s![.., 0..BATCH_HEIGHT, 0..BATCH_WIDTH]].map(deref_slice_info)
        );
    }

    #[test]
    fn test_batcher_mostly_intersects() {
        use super::*;

        let batcher = Batcher::new(BATCH_HEIGHT + 1, BATCH_WIDTH);
        assert_eq!(batcher.v_batches, 2);
        assert_eq!(batcher.h_batches, 1);
        assert_eq!(batcher.v_offset, 1.0);
        assert_eq!(batcher.h_offset, 0.0);

        let slices = batcher.iter().map(deref_slice_info).collect::<Vec<_>>();
        assert_eq!(
            &slices,
            &[
                s![.., 0..BATCH_HEIGHT, 0..BATCH_WIDTH],
                s![.., 1..BATCH_HEIGHT + 1, 0..BATCH_WIDTH]
            ]
            .map(deref_slice_info)
        );
    }

    #[test]
    fn test_batcher_mostly_intersects2() {
        use super::*;

        let batcher = Batcher::new(BATCH_HEIGHT + 1, BATCH_WIDTH + 1);
        assert_eq!(batcher.v_batches, 2);
        assert_eq!(batcher.h_batches, 2);
        assert_eq!(batcher.v_offset, 1.0);
        assert_eq!(batcher.h_offset, 1.0);

        let slices = batcher.iter().map(deref_slice_info).collect::<Vec<_>>();
        assert_eq!(
            &slices,
            &[
                s![.., 0..BATCH_HEIGHT, 0..BATCH_WIDTH],
                s![.., 0..BATCH_HEIGHT, 1..BATCH_WIDTH + 1],
                s![.., 1..BATCH_HEIGHT + 1, 0..BATCH_WIDTH],
                s![.., 1..BATCH_HEIGHT + 1, 1..BATCH_WIDTH + 1],
            ]
            .map(deref_slice_info)
        );
    }

    #[test]
    fn test_batcher_mostly_not_intersects() {
        use super::*;

        let batcher = Batcher::new(BATCH_HEIGHT * 2 - 1, BATCH_WIDTH * 2);
        assert_eq!(batcher.v_batches, 2);
        assert_eq!(batcher.h_batches, 2);
        assert_eq!(batcher.v_offset, (BATCH_HEIGHT - 1) as f64);
        assert_eq!(batcher.h_offset, BATCH_WIDTH as f64);

        let mut slices = batcher.iter().map(deref_slice_info).collect::<Vec<_>>();
        assert_eq!(
            &slices,
            &[
                s![.., 0..BATCH_HEIGHT, 0..BATCH_WIDTH],
                s![.., 0..BATCH_HEIGHT, BATCH_WIDTH..BATCH_WIDTH * 2],
                s![.., BATCH_HEIGHT - 1..BATCH_HEIGHT * 2 - 1, 0..BATCH_WIDTH],
                s![
                    ..,
                    BATCH_HEIGHT - 1..BATCH_HEIGHT * 2 - 1,
                    BATCH_WIDTH..BATCH_WIDTH * 2
                ],
            ]
            .map(deref_slice_info)
        );
    }

    #[test]
    fn test_batcher_not_intersects() {
        use super::*;

        let batcher = Batcher::new(BATCH_HEIGHT * 2, BATCH_WIDTH * 2);
        assert_eq!(batcher.v_batches, 2);
        assert_eq!(batcher.h_batches, 2);
        assert_eq!(batcher.v_offset, BATCH_HEIGHT as f64);
        assert_eq!(batcher.h_offset, BATCH_WIDTH as f64);

        let slices = batcher.iter().map(deref_slice_info).collect::<Vec<_>>();
        assert_eq!(
            &slices,
            &[
                s![.., 0..BATCH_HEIGHT, 0..BATCH_WIDTH],
                s![.., 0..BATCH_HEIGHT, BATCH_WIDTH..BATCH_WIDTH * 2],
                s![.., BATCH_HEIGHT..BATCH_HEIGHT * 2, 0..BATCH_WIDTH],
                s![
                    ..,
                    BATCH_HEIGHT..BATCH_HEIGHT * 2,
                    BATCH_WIDTH..BATCH_WIDTH * 2
                ],
            ]
            .map(deref_slice_info)
        );
    }
}
