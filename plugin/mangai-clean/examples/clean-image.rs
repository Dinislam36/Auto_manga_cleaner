use camino::Utf8PathBuf;
use clap::Parser;
use mangai_clean::MangaiClean;
use nshare::{MutNdarray2, ToNdarray3};
use tract_onnx::prelude::tract_ndarray as ndarray;

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

fn main() {
    let args = Args::parse();

    println!("Loading the image...");
    let image_image = image::open(args.input).unwrap();
    let image = image_image.to_rgb8().into_ndarray3();

    let mut output_image = image::GrayImage::new(828, 1176);
    println!("Loading the model...");
    let clean = MangaiClean::new().unwrap();

    println!("Cleaning the image...");
    clean.clean_page(image.view(), output_image.mut_ndarray2());

    println!("Saving the image...");
    output_image.save(args.output).unwrap();
}
