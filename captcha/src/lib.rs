use anyhow::{anyhow, Result};
use d4ocr_rust::{ImageSize, TransformationPipeline};
use image::DynamicImage;
use once_cell::sync::OnceCell;
use tokio::sync::{oneshot, Mutex};

const IMAGE_DEFAULT_WIDTH: usize = 92;
const IMAGE_DEFAULT_HEIGHT: usize = 34;

static MODEL: OnceCell<TransformationPipeline> = OnceCell::new();

pub fn init() {
    let image_size = ImageSize {
        width: IMAGE_DEFAULT_WIDTH,
        height: IMAGE_DEFAULT_HEIGHT,
    };

    if let Err(_) = MODEL.set(TransformationPipeline::new(image_size)) {
        panic!("Failed to load OCR model.");
    }
}

pub async fn async_init() -> Result<()> {
    tokio::task::spawn_blocking(init).await?;
    Ok(())
}

fn get() -> &'static TransformationPipeline {
    MODEL.get().expect("You should call init() or async_init() first.")
}

pub fn recognize(original_image: DynamicImage) -> Result<String> {
    let image = original_image.to_luma8();

    get().recognize(image).map_err(|e| anyhow!("{e}"))
}

pub async fn async_recognize(original_image: DynamicImage) -> Result<String> {
    let (tx, rx) = oneshot::channel();

    tokio::task::spawn_blocking(move || {
        let result = recognize(original_image);
        let _ = tx.send(result);
    })
    .await?;

    rx.await?
}
