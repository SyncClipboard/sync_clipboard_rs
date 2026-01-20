use clipboard_rs::{Clipboard, ClipboardContext, common::RustImage};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use image::DynamicImage;

pub struct ClipboardHandler {
    backend: Arc<Mutex<Box<dyn Clipboard>>>,
}

impl ClipboardHandler {
    pub fn new() -> Result<Self> {
        let clipboard = ClipboardContext::new().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(Self {
            backend: Arc::new(Mutex::new(Box::new(clipboard))),
        })
    }

    pub fn get_text(&self) -> Result<String> {
        let clipboard = self.backend.lock().unwrap();
        clipboard.get_text().map_err(|e| anyhow::anyhow!("{}", e))
    }

    pub fn set_text(&self, text: String) -> Result<()> {
        let clipboard = self.backend.lock().unwrap();
        clipboard.set_text(text).map_err(|e| anyhow::anyhow!("{}", e))
    }

    pub fn get_image(&self) -> Result<DynamicImage> {
        let clipboard = self.backend.lock().unwrap();
        let img = clipboard.get_image().map_err(|e| anyhow::anyhow!("{}", e))?;
        // Try to_dynamic_image if available, else usage error will show what methods match
         match img.get_dynamic_image() {
            Ok(i) => Ok(i),
            Err(e) => Err(anyhow::anyhow!("Failed to convert image: {}", e)),
        }
    }

    pub fn set_image(&self, image: DynamicImage) -> Result<()> {
        let clipboard = self.backend.lock().unwrap();
        let rust_image = RustImage::from_dynamic_image(image);
        clipboard.set_image(rust_image).map_err(|e| anyhow::anyhow!("{}", e))
    }

    pub fn get_html(&self) -> Result<String> {
        let clipboard = self.backend.lock().unwrap();
        clipboard.get_html().map_err(|e| anyhow::anyhow!("{}", e))
    }

    pub fn set_html(&self, html: String, alt_text: Option<String>) -> Result<()> {
        let clipboard = self.backend.lock().unwrap();
        if let Some(text) = alt_text {
            let _ = clipboard.set_text(text);
        }
        clipboard.set_html(html).map_err(|e| anyhow::anyhow!("{}", e))
    }
}
