// ensure intel_mkl_src is linked in
extern crate intel_mkl_src as _src;

mod console;
mod panic_handler;
mod point;
#[allow(unused)]
mod progress;
mod rect;

use ndarray::{
    ArrayView2, ArrayView3, ArrayViewD, ArrayViewMut2, ArrayViewMut3, ArrayViewMutD, Ix2, Ix3,
    Shape, ShapeBuilder,
};
use num_enum::TryFromPrimitive;
use point::Point;
use ps_sdk_sys::{int16, int32, Boolean, FilterRecord};
use rect::Rect;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use tracing::{error, info};

#[allow(unused)]
pub struct PluginBaseParams {
    test_abort_cb: unsafe extern "C" fn() -> Boolean,
    progress_cb: unsafe extern "C" fn(done: int32, total: int32),
    host_cb: unsafe extern "C" fn(selector: int16, data: *mut isize),
    host_signature: u32,
}

impl PluginBaseParams {
    pub fn from_filter(filter_param_block: &FilterRecord) -> Self {
        Self {
            test_abort_cb: filter_param_block.abortProc.expect("abortProc is null"),
            progress_cb: filter_param_block
                .progressProc
                .expect("progressProc is null"),
            host_cb: filter_param_block.hostProc.expect("hostProc is null"),
            host_signature: filter_param_block.hostSig,
        }
    }

    // TODO: support user abort
    #[allow(unused)]
    pub fn is_aborted(&self) -> bool {
        unsafe { (self.test_abort_cb)() != 0 }
    }

    // We do our own progress reporting
    #[allow(unused)]
    pub fn report_progress(&self, done: i32, total: i32) {
        unsafe { (self.progress_cb)(done, total) }
    }

    #[allow(unused)]
    pub unsafe fn call_host(&self, selector: i16, data: *mut isize) {
        (self.host_cb)(selector, data)
    }
}

#[derive(TryFromPrimitive, Debug)]
#[repr(u16)]
enum FilterSelector {
    About = ps_sdk_sys::filterSelectorAbout as u16,
    Parameters = ps_sdk_sys::filterSelectorParameters as u16,
    Prepare = ps_sdk_sys::filterSelectorPrepare as u16,
    Start = ps_sdk_sys::filterSelectorStart as u16,
    Continue = ps_sdk_sys::filterSelectorContinue as u16,
    Finish = ps_sdk_sys::filterSelectorFinish as u16,
}

#[allow(unused)]
#[derive(Debug)]
struct PluginPrepareParams {
    image_size: Point,
    planes: i16,
    filter_rect: Rect,
}

impl PluginPrepareParams {
    pub fn from_filter(filter_param_block: &FilterRecord) -> Self {
        let big_data = unsafe { &*filter_param_block.bigDocumentData };

        Self {
            image_size: big_data.imageSize32.into(),
            planes: filter_param_block.planes,
            filter_rect: big_data.filterRect32.into(),
        }
    }
}

#[derive(Debug, TryFromPrimitive, Eq, PartialEq, Copy, Clone)]
#[repr(i16)]
enum ImageMode {
    Bitmap = ps_sdk_sys::plugInModeBitmap as i16,
    GrayScale = ps_sdk_sys::plugInModeGrayScale as i16,
    IndexedColor = ps_sdk_sys::plugInModeIndexedColor as i16,
    RGBColor = ps_sdk_sys::plugInModeRGBColor as i16,
    CMYKColor = ps_sdk_sys::plugInModeCMYKColor as i16,
    HSLColor = ps_sdk_sys::plugInModeHSLColor as i16,
    HSBColor = ps_sdk_sys::plugInModeHSBColor as i16,
    Multichannel = ps_sdk_sys::plugInModeMultichannel as i16,
    Duotone = ps_sdk_sys::plugInModeDuotone as i16,
    LabColor = ps_sdk_sys::plugInModeLabColor as i16,
    Gray16 = ps_sdk_sys::plugInModeGray16 as i16,
    RGB48 = ps_sdk_sys::plugInModeRGB48 as i16,
    Lab48 = ps_sdk_sys::plugInModeLab48 as i16,
    CMYK64 = ps_sdk_sys::plugInModeCMYK64 as i16,
    DeepMultichannel = ps_sdk_sys::plugInModeDeepMultichannel as i16,
    Duotone16 = ps_sdk_sys::plugInModeDuotone16 as i16,
    RGB96 = ps_sdk_sys::plugInModeRGB96 as i16,
    Gray32 = ps_sdk_sys::plugInModeGray32 as i16,
}

#[allow(unused)]
#[derive(Debug)]
struct PluginStartParams {
    image_size: Point,
    planes: i16,
    filter_rect: Rect,
    image_mode: ImageMode,
    depth: i32,
}

impl PluginStartParams {
    pub fn from_filter(filter_param_block: &FilterRecord) -> Self {
        let big_data = unsafe { &*filter_param_block.bigDocumentData };

        Self {
            image_size: big_data.imageSize32.into(),
            planes: filter_param_block.planes,
            filter_rect: big_data.filterRect32.into(),
            image_mode: filter_param_block.imageMode.try_into().unwrap(),
            depth: filter_param_block.depth,
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct DataRequest {
    rect: Rect,
    lo_plane: i16,
    hi_plane: i16,
}

impl DataRequest {
    pub fn empty() -> Self {
        Self {
            rect: Rect::empty(),
            lo_plane: 0,
            hi_plane: 0,
        }
    }

    pub fn to_filter_in(&self, filter_param_block: &mut FilterRecord) {
        let big_data = unsafe { &mut *filter_param_block.bigDocumentData };

        big_data.PluginUsing32BitCoordinates = 1;

        big_data.inRect32 = self.rect.into();
        filter_param_block.inLoPlane = self.lo_plane;
        filter_param_block.inHiPlane = self.hi_plane;
    }

    pub fn to_filter_out(&self, filter_param_block: &mut FilterRecord) {
        let big_data = unsafe { &mut *filter_param_block.bigDocumentData };

        big_data.PluginUsing32BitCoordinates = 1;

        big_data.outRect32 = self.rect.into();
        filter_param_block.outLoPlane = self.lo_plane;
        filter_param_block.outHiPlane = self.hi_plane;
    }
}

#[derive(Debug)]
struct InOutRequest {
    rq_in: DataRequest,
    rq_out: DataRequest,
}

impl InOutRequest {
    pub fn empty() -> Self {
        Self {
            rq_in: DataRequest::empty(),
            rq_out: DataRequest::empty(),
        }
    }

    pub fn to_filter(&self, filter_param_block: &mut FilterRecord) {
        self.rq_in.to_filter_in(filter_param_block);
        self.rq_out.to_filter_out(filter_param_block);
    }
}

#[derive(Debug)]
struct PluginStartResult {
    request: InOutRequest,
}

impl PluginStartResult {
    pub fn to_filter(&self, filter_param_block: &mut FilterRecord) {
        self.request.to_filter(filter_param_block);
    }
}

struct PluginContinueParams<'a> {
    in_rect: Rect,
    out_rect: Rect,
    in_data: ArrayViewD<'a, u8>,
    out_data: ArrayViewMutD<'a, u8>,
    phantom: PhantomData<&'a mut ()>,
}

impl<'a> Debug for PluginContinueParams<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PluginContinueParams")
            .field("in_rect", &self.in_rect)
            .field("out_rect", &self.out_rect)
            .field("in_data", &self.in_data.dim())
            .field("out_data", &self.out_data.dim())
            .finish()
    }
}

impl<'a> PluginContinueParams<'a> {
    pub fn from_filter(filter_param_block: &mut FilterRecord) -> Self {
        let big_data = unsafe { &*filter_param_block.bigDocumentData };

        let in_height = (big_data.inRect32.bottom - big_data.inRect32.top) as usize;
        let in_width = (big_data.inRect32.right - big_data.inRect32.left) as usize;
        let in_stride = filter_param_block.inRowBytes as usize;

        let mode: ImageMode = filter_param_block.imageMode.try_into().unwrap();

        let in_data = match mode {
            ImageMode::GrayScale => unsafe {
                ArrayView2::from_shape_ptr(
                    Shape::from(Ix2(in_height, in_width)).strides(Ix2(in_stride, 1)),
                    filter_param_block.inData as *const u8,
                )
            }
            .into_dyn(),
            ImageMode::RGBColor => unsafe {
                ArrayView3::from_shape_ptr(
                    Shape::from(Ix3(3, in_height, in_width)).strides(Ix3(1, in_stride, 3)),
                    filter_param_block.inData as *const u8,
                )
            }
            .into_dyn(),
            _ => unimplemented!(),
        };

        let out_height = (big_data.outRect32.bottom - big_data.outRect32.top) as usize;
        let out_width = (big_data.outRect32.right - big_data.outRect32.left) as usize;
        let out_stride = filter_param_block.outRowBytes as usize;

        let out_data = match mode {
            ImageMode::GrayScale => unsafe {
                ArrayViewMut2::from_shape_ptr(
                    Shape::from(Ix2(out_height, out_width)).strides(Ix2(out_stride, 1)),
                    filter_param_block.outData as *mut u8,
                )
            }
            .into_dyn(),
            ImageMode::RGBColor => unsafe {
                ArrayViewMut3::from_shape_ptr(
                    Shape::from(Ix3(3, out_height, out_width)).strides(Ix3(1, out_stride, 3)),
                    filter_param_block.outData as *mut u8,
                )
            }
            .into_dyn(),
            _ => unimplemented!(),
        };

        Self {
            in_rect: big_data.inRect32.into(),
            out_rect: big_data.outRect32.into(),
            in_data,
            out_data,
            phantom: PhantomData {},
        }
    }
}

#[derive(Debug)]
struct PluginContinueResult {
    request: InOutRequest,
}

impl PluginContinueResult {
    pub fn to_filter(&self, filter_param_block: &mut FilterRecord) {
        self.request.to_filter(filter_param_block);
    }
}

#[allow(unused)]
#[repr(i16)]
enum FilterError {
    Other = -1,

    BadParameters = ps_sdk_sys::filterBadParameters as i16,
    BadMode = ps_sdk_sys::filterBadMode as i16,
}

fn about(_plugin: &PluginBaseParams) -> Result<(), FilterError> {
    msgbox::create("About", "MangaiClean v 0.0.0.0.0.0.0.01-beta (lol, can you even see this window? I didn't find a way to show it)", msgbox::IconType::Info).unwrap();

    Ok(())
}

fn parameters(_plugin: &PluginBaseParams) -> Result<(), FilterError> {
    // here we should show a dialog with parameters
    // we don't have any parameters yet :P

    Ok(())
}

fn prepare(_plugin: &PluginBaseParams, prepare: PluginPrepareParams) -> Result<(), FilterError> {
    // Calculate memory requirements and allocate memory needed.
    // we don't use PS's memory managements, so I don't think we need to do anything here
    info!("{:#?}", prepare);

    Ok(())
}

fn start(
    _plugin: &PluginBaseParams,
    start: PluginStartParams,
) -> Result<PluginStartResult, FilterError> {
    // Check scripting parameters versus our parameters. Update if necessary.
    // Show UI if flagged/needed.
    //
    // Set initial image rectangles to process.
    // and, actually, this is where most of the processing should be done

    if !matches!(
        (start.image_mode, start.depth, start.planes),
        (ImageMode::RGBColor, 8, 3 | 4) | (ImageMode::GrayScale, 8, 1 | 2)
    ) {
        error!(
            "Bad mode: image_mode={:?}, depth={}, planes={} (support only RGB8 and GrayScale8)",
            start.image_mode, start.depth, start.planes
        );
        return Err(FilterError::BadMode);
    }

    info!("{:#?}", start);

    let hi_plane = match (start.image_mode, start.planes) {
        (ImageMode::RGBColor, 3) => 2,
        (ImageMode::RGBColor, 4) => 2, // no alpha!
        (ImageMode::GrayScale, 1) => 0,
        (ImageMode::GrayScale, 2) => 0, // no alpha!
        _ => unreachable!(),
    };

    info!("Requesting planes 0..={}", hi_plane);

    Ok(PluginStartResult {
        // request all the image to be processed
        request: InOutRequest {
            rq_in: DataRequest {
                rect: start.filter_rect,
                lo_plane: 0,
                hi_plane,
            },
            rq_out: DataRequest {
                rect: start.filter_rect,
                lo_plane: 0,
                hi_plane,
            },
        },
    })
}

fn r#continue(
    plugin: &PluginBaseParams,
    r#continue: PluginContinueParams,
) -> Result<PluginContinueResult, FilterError> {
    // Filter a portion of the image.
    // Update image rectangles for next pass.

    info!("{:#?}", r#continue);

    progress::run_with_progress(plugin, |progress| {
        info!("Loading mangai model...");
        let clean = mangai_clean::MangaiClean::new(progress).unwrap();
        info!("Loaded mangai clean model");

        match r#continue.in_data.shape() {
            [_, _] => {
                // grayscale
                let in_data = r#continue.in_data.into_dimensionality::<Ix2>().unwrap();
                let out_data = r#continue.out_data.into_dimensionality::<Ix2>().unwrap();

                clean.clean_grayscale_page(in_data, out_data, progress);
            }
            [3, _, _] => {
                // RGB
                let in_data = r#continue.in_data.into_dimensionality::<Ix3>().unwrap();
                let out_data = r#continue.out_data.into_dimensionality::<Ix3>().unwrap();

                clean.clean_page(in_data, out_data, progress);
            }
            _ => unimplemented!(),
        }

        info!("Cleaned page!");
    });

    Ok(PluginContinueResult {
        request: InOutRequest::empty(),
    })
}

fn finish(_plugin: &PluginBaseParams) -> Result<(), FilterError> {
    // Clean up. Pass back scripting parameters.

    // we don't need to clean up anything
    // msgbox::create("Finish", "Finish", msgbox::IconType::Info).unwrap();

    Ok(())
}

#[no_mangle]
pub extern "C" fn PluginMain(
    selector: u16,
    filter_param_block: &mut FilterRecord,
    plugin_data: &mut u32,
    result: &mut i16,
) {
    // TODO: catch panics, we can't let those escape the FFI boundary

    console::setup_console();
    panic_handler::setup_panic();

    let selector = FilterSelector::try_from(selector).expect("Invalid selector");

    let plugin = PluginBaseParams::from_filter(filter_param_block);

    info!("PluginMain: {:?}; data = {:?}", selector, plugin_data);
    let span = tracing::info_span!("PluginMain", selector = ?selector);
    let _span_guard = span.enter();

    // plugin.report_progress(10, 100);

    let fn_result = match selector {
        FilterSelector::About => about(&plugin),
        FilterSelector::Parameters => parameters(&plugin),
        FilterSelector::Prepare => prepare(
            &plugin,
            PluginPrepareParams::from_filter(filter_param_block),
        ),
        FilterSelector::Start => start(&plugin, PluginStartParams::from_filter(filter_param_block))
            .map(|result| {
                info!("start result: {:#?}", result);
                result.to_filter(filter_param_block);
            }),
        FilterSelector::Continue => r#continue(
            &plugin,
            PluginContinueParams::from_filter(filter_param_block),
        )
        .map(|result| {
            info!("continue result: {:#?}", result);
            result.to_filter(filter_param_block);
        }),
        FilterSelector::Finish => finish(&plugin),
    };

    match fn_result {
        Ok(_) => *result = 0,
        Err(err) => *result = err as i16,
    }

    // let _ = msgbox::create(
    //     "Hello, world!",
    //     &format!("selector = {:?}, data = {:?}", selector, data),
    //     msgbox::IconType::Info,
    // );
    // plugin.report_progress(20, 100);
    // let _ = msgbox::create(
    //     "FilterRecord",
    //     &format!("{:?}", filter_param_block),
    //     msgbox::IconType::Info,
    // );
}
