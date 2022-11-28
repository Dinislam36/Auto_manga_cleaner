// This place is not a place of honor... no highly esteemed deed is commemorated here... nothing valued is here.
// this code is not rusty at all
// it modifies the global state of the program (by initializing some Win32 Controls)
// it uses unsafe code
// honestly, progress reporting should be redesigned
// but here we are

use crate::PluginBaseParams;
use mangai_clean::{ProgressKind, ProgressReporter};
use nwg::NativeUi;
use std::cell::RefCell;
use std::sync::{Arc, Mutex, Once};
use tracing::info;

#[derive(Default, Clone, Debug)]
enum ProgressStatus {
    #[default]
    Hidden,
    Stop,
    Visible(String, f64),
}

#[derive(Default, nwd::NwgUi)]
pub struct BasicApp {
    #[nwg_control(size: (800, 100), center: true, topmost: true, title: "Basic example", flags: "POPUP")]
    #[nwg_events( OnInit: [BasicApp::setup], OnWindowClose: [BasicApp::quit] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, spacing: 1, max_row: Some(5), margin: [15,15,15,15])]
    grid: nwg::GridLayout,

    #[nwg_control]
    #[nwg_layout_item(layout: grid, row: 1, col: 0)]
    message: nwg::Label,

    func: RefCell<Option<Box<dyn FnOnce(&mut dyn ProgressReporter) + Send>>>,
    status: Arc<Mutex<ProgressStatus>>,

    #[nwg_control]
    #[nwg_layout_item(layout: grid, col: 0, row: 3, row_span: 1)]
    hello_progress: nwg::ProgressBar,

    #[nwg_control]
    #[nwg_events(OnNotice: [BasicApp::update])]
    notice: nwg::Notice,
}

impl BasicApp {
    fn quit(&self) {
        // nwg::stop_thread_dispatch();
    }

    fn setup(&self) {
        let notice_sender = self.notice.sender();
        let status_arc = self.status.clone();

        *status_arc.lock().unwrap() = ProgressStatus::Hidden;
        notice_sender.notice();

        let func = self.func.borrow_mut().take().unwrap();

        std::thread::spawn(move || {
            func(&mut GuiSenderProgress {
                notice_sender,
                status: status_arc.clone(),
                total: None,
                operation: None,
                kind: None,
            });

            *status_arc.lock().unwrap() = ProgressStatus::Stop;
            notice_sender.notice();
        });
    }

    fn update(&self) {
        let status = self.status.lock().unwrap().clone();
        match status {
            ProgressStatus::Hidden => self.window.set_visible(false),
            ProgressStatus::Stop => {
                self.window.set_visible(false);
                nwg::stop_thread_dispatch();
            }
            ProgressStatus::Visible(operation, progress) => {
                self.window.set_visible(true);
                self.message.set_text(&operation);
                self.hello_progress.set_range(0..100);
                self.hello_progress.set_pos((progress * 100.0) as u32);
            }
        }
    }
}

struct GuiSenderProgress {
    notice_sender: nwg::NoticeSender,
    status: Arc<Mutex<ProgressStatus>>,
    total: Option<usize>,
    operation: Option<String>,
    kind: Option<ProgressKind>,
}

impl GuiSenderProgress {
    fn format_message(&self, progress: usize) -> String {
        use human_bytes::human_bytes;

        match self.kind.unwrap() {
            ProgressKind::Items => format!(
                "{}: {}/{}",
                self.operation.as_ref().unwrap(),
                progress,
                self.total.unwrap()
            ),
            ProgressKind::Bytes => format!(
                "{}: {}/{}",
                self.operation.as_ref().unwrap(),
                human_bytes(progress as f64),
                human_bytes(self.total.unwrap() as f64)
            ),
        }
    }
}

impl ProgressReporter for GuiSenderProgress {
    fn init(&mut self, kind: ProgressKind, operation: &str, total: usize) {
        self.total = Some(total);
        self.operation = Some(operation.to_owned());
        self.kind = Some(kind);
    }

    fn progress(&mut self, progress: usize) {
        *self.status.lock().unwrap() = ProgressStatus::Visible(
            self.format_message(progress),
            progress as f64 / self.total.unwrap() as f64,
        );
        self.notice_sender.notice();
    }

    fn finish(&mut self) {
        *self.status.lock().unwrap() = ProgressStatus::Hidden;
        self.notice_sender.notice();
    }
}

static GUI_INIT: Once = Once::new();

pub fn run_with_progress<'a>(
    ps: &PluginBaseParams,
    func: impl FnOnce(&mut dyn ProgressReporter) + Send + 'a,
) {
    // the custom UI is prone to crashing, so use PS's progress bar instead
    // GUI_INIT.call_once(|| {
    //     nwg::init().unwrap();
    //     nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    // });
    //
    // let app = BasicApp::build_ui(Default::default()).expect("Failed to build UI");
    //
    // // I hate GUI I hate GUI I hate GUI
    // let box_: Box<dyn FnOnce(&mut dyn ProgressReporter) + Send + 'a> = Box::new(func);
    // let box_: Box<dyn FnOnce(&mut dyn ProgressReporter) + Send> =
    // // SAFETY: this function __should__ only be able to terminate after the closure is terminated
    //     unsafe { std::mem::transmute(box_) };
    //
    // *app.func.borrow_mut() = Some(box_);
    //
    // nwg::dispatch_thread_events();

    std::thread::scope(|s| {
        s.spawn(|| {
            let mut progress = PsProgressReporter::new(ps);

            info!("!!!!");

            func(&mut progress);
        })
        .join()
        .unwrap();
    })
}

pub struct PsProgressReporter<'a> {
    params: &'a PluginBaseParams,
    total: i32,
}

impl<'a> PsProgressReporter<'a> {
    pub fn new(params: &'a PluginBaseParams) -> Self {
        Self { params, total: 0 }
    }
}

impl<'a> ProgressReporter for PsProgressReporter<'a> {
    fn init(&mut self, _kind: ProgressKind, _operation: &str, total: usize) {
        info!("progress START: {:?} {:?} {}", _kind, _operation, total);
        self.total = total as i32;
    }

    fn progress(&mut self, progress: usize) {
        info!("progress: {}", progress);
        self.params.report_progress(progress as i32, self.total);
    }

    fn finish(&mut self) {
        self.params.report_progress(self.total, self.total);
    }
}
