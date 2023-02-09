/*
 * 上应小风筝  便利校园，一步到位
 * Copyright (C) 2020-2023 上海应用技术大学 上应小风筝团队
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use anyhow::{anyhow, Result};
use d4ocr_rust::{ImageSize, TransformationPipeline};
use image::EncodableLayout;
use once_cell::sync::OnceCell;
use tokio::sync::{mpsc, oneshot};

type AsyncChannelType = (Vec<u8>, oneshot::Sender<Result<String>>);

const IMAGE_DEFAULT_WIDTH: usize = 92;
const IMAGE_DEFAULT_HEIGHT: usize = 34;

const QUEUE_SIZE: usize = 20;

static MODEL: OnceCell<TransformationPipeline> = OnceCell::new();
static CHANNEL_SENDER: OnceCell<mpsc::Sender<AsyncChannelType>> = OnceCell::new();

fn get() -> &'static TransformationPipeline {
    MODEL.get().expect("You should call init() or async_init() first.")
}

pub fn init() {
    let image_size = ImageSize {
        width: IMAGE_DEFAULT_WIDTH,
        height: IMAGE_DEFAULT_HEIGHT,
    };

    if let Err(_) = MODEL.set(TransformationPipeline::new(image_size)) {
        panic!("Failed to load OCR model.");
    }
}

pub fn recognize(image: Vec<u8>) -> Result<String> {
    let raw_image = image::load_from_memory(image.as_bytes())?;
    let gray_image = raw_image.to_luma8();

    get().recognize(gray_image).map_err(|e| anyhow!("{e}"))
}

pub async fn async_init() {
    let _ = tokio::task::spawn_blocking(init).await;

    let (tx, mut rx) = mpsc::channel::<AsyncChannelType>(QUEUE_SIZE);
    std::thread::spawn(move || {
        if let Some((image, sender)) = rx.blocking_recv() {
            let result = recognize(image);
            let _ = sender.send(result);
        }
    });

    let _ = CHANNEL_SENDER.set(tx);
}

pub async fn async_recognize(image: Vec<u8>) -> Result<String> {
    let sender = CHANNEL_SENDER
        .get()
        .expect("Please use async_init() to initialize captcha-recognition module.");

    // Oneshot channel for response
    let (tx, rx) = oneshot::channel::<Result<String>>();
    // Maybe sending result to a fail if the thread is busy and queue is full.
    sender.send((image, tx)).await?;
    rx.await?
}
