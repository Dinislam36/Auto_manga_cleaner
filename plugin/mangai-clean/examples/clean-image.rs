use camino::Utf8PathBuf;
use clap::Parser;
use mangai_clean::MangaiClean;
use nshare::{MutNdarray2, ToNdarray2};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    input: Utf8PathBuf,

    /// Number of times to greet
    #[arg(short, long)]
    output: Utf8PathBuf,
}

struct IndicatifProgress {
    pb: Option<indicatif::ProgressBar>,
}

impl IndicatifProgress {
    fn new() -> Self {
        Self { pb: None }
    }
}

impl mangai_clean::ProgressReporter for IndicatifProgress {
    fn total(&mut self, total: usize) {
        self.pb = Some(indicatif::ProgressBar::new(total as u64));
    }

    fn progress(&mut self, current: usize) {
        if let Some(pb) = &self.pb {
            pb.set_position(current as u64);
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    println!("Loading the image...");
    let image_image = image::open(args.input).unwrap();
    let image = image_image.to_luma8().into_ndarray2();

    let mut output_image = image::GrayImage::new(image_image.width(), image_image.height());
    println!("Loading the model...");
    let clean = MangaiClean::new().unwrap();

    println!("Cleaning the image...");
    clean.clean_grayscale_page(
        image.view(),
        output_image.mut_ndarray2(),
        Box::new(IndicatifProgress::new()),
    );

    println!("Saving the image...");
    output_image.save(args.output).unwrap();
}
