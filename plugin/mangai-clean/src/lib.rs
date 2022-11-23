mod console;
mod panic_handler;
mod point;
mod rect;

use num_enum::{FromPrimitive, TryFromPrimitive};
use point::Point;
use ps_sdk_sys::{int16, int32, Boolean, FilterRecord};
use rect::Rect;

struct PluginBaseParams {
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

    pub fn is_aborted(&self) -> bool {
        unsafe { (self.test_abort_cb)() != 0 }
    }

    pub fn report_progress(&self, done: i32, total: i32) {
        unsafe { (self.progress_cb)(done, total) }
    }

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

#[derive(Debug, TryFromPrimitive, Eq, PartialEq)]
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

#[derive(Debug)]
struct InOutRequest(Option<(Rect, Rect)>);

impl InOutRequest {
    pub fn to_filter(&self, filter_param_block: &mut FilterRecord) {
        let big_data = unsafe { &mut *filter_param_block.bigDocumentData };

        big_data.PluginUsing32BitCoordinates = 1;

        if let Some((in_rect, out_rect)) = self.0 {
            big_data.inRect32 = in_rect.into();
            big_data.outRect32 = out_rect.into();
        } else {
            big_data.inRect32 = Rect::empty().into();
            big_data.outRect32 = Rect::empty().into();
        }
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

#[derive(Debug)]
struct PluginContinueParams {
    in_rect: Rect,
    out_rect: Rect,
    // TODO: better types
    in_data: *mut u8,
    out_data: *mut u8,
}

impl PluginContinueParams {
    pub fn from_filter(filter_param_block: &FilterRecord) -> Self {
        let big_data = unsafe { &*filter_param_block.bigDocumentData };

        Self {
            in_rect: big_data.inRect32.into(),
            out_rect: big_data.outRect32.into(),
            in_data: filter_param_block.inData as *mut u8,
            out_data: filter_param_block.outData as *mut u8,
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

#[repr(i16)]
enum FilterError {
    BadParameters = ps_sdk_sys::filterBadParameters as i16,
    BadMode = ps_sdk_sys::filterBadMode as i16,
}

fn about(plugin: &PluginBaseParams) -> Result<(), FilterError> {
    msgbox::create("About", "MangaiClean v 0.0.0.0.0.0.0.01-beta (lol, can you even see this window? I didn't find a way to show it)", msgbox::IconType::Info).unwrap();

    Ok(())
}

fn parameters(plugin: &PluginBaseParams) -> Result<(), FilterError> {
    // here we should show a dialog with parameters
    // we don't have any parameters yet :P

    Ok(())
}

fn prepare(plugin: &PluginBaseParams, prepare: &PluginPrepareParams) -> Result<(), FilterError> {
    // Calculate memory requirements and allocate memory needed.
    // we don't use PS's memory managements, so I don't think we need to do anything here
    println!("{:#?}", prepare);

    Ok(())
}

fn start(
    plugin: &PluginBaseParams,
    start: &PluginStartParams,
) -> Result<PluginStartResult, FilterError> {
    // Check scripting parameters versus our parameters. Update if necessary.
    // Show UI if flagged/needed.
    //
    // Set initial image rectangles to process.
    // and, actually, this is where most of the processing should be done

    if start.image_mode != ImageMode::RGBColor || start.depth != 8 {
        println!(
            "Bad mode: {:?} (we only support RGBColor for now)",
            start.image_mode
        );
        return Err(FilterError::BadMode);
    }

    println!("{:#?}", start);

    Ok(PluginStartResult {
        // request all the image to be processed
        request: InOutRequest(Some((start.filter_rect, start.filter_rect))),
    })
}

fn r#continue(
    plugin: &PluginBaseParams,
    r#continue: &PluginContinueParams,
) -> Result<PluginContinueResult, FilterError> {
    // Filter a portion of the image.
    // Update image rectangles for next pass.

    println!("{:#?}", r#continue);

    Ok(PluginContinueResult {
        request: InOutRequest(None),
    })
}

fn finish(plugin: &PluginBaseParams) -> Result<(), FilterError> {
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
    console::setup_console();
    panic_handler::setup_panic();

    let selector = FilterSelector::try_from(selector).expect("Invalid selector");

    let plugin = PluginBaseParams::from_filter(filter_param_block);

    println!("----\nPluginMain: {:?}; data = {:?}", selector, plugin_data);

    // plugin.report_progress(10, 100);

    let fn_result = match selector {
        FilterSelector::About => about(&plugin),
        FilterSelector::Parameters => parameters(&plugin),
        FilterSelector::Prepare => prepare(
            &plugin,
            &PluginPrepareParams::from_filter(filter_param_block),
        ),
        FilterSelector::Start => {
            start(&plugin, &PluginStartParams::from_filter(filter_param_block)).map(|result| {
                println!("start result: {:#?}", result);
                result.to_filter(filter_param_block);
            })
        }
        FilterSelector::Continue => r#continue(
            &plugin,
            &PluginContinueParams::from_filter(filter_param_block),
        )
        .map(|result| {
            println!("continue result: {:#?}", result);
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
