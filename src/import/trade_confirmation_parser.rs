use pdf2image::{PDF, PDF2ImageError, RenderOptionsBuilder, image};
use base64::{engine::general_purpose, Engine as _};
use std::io::Cursor;

pub fn parse_trade_confirmation(content: &[u8]) -> Result<Vec<String>, ConfirmationParseError> {
    let pdf = PDF::from_bytes(content.to_vec())?;
    let pages = pdf.render(
        pdf2image::Pages::Range(1..=8),
        RenderOptionsBuilder::default().build()
            .map_err(|e| ConfirmationParseError::ImageRenderError(e.to_string()))?,
    )?;

    let mut images_encoded: Vec<String> = vec![];
    for page in pages.iter() {
        let mut image_data: Vec<u8> = Vec::new();
        page.write_to(&mut Cursor::new(&mut image_data), image::ImageFormat::Jpeg)?;
        images_encoded.push(general_purpose::STANDARD.encode(&image_data));
    }
    Ok(images_encoded)
}

#[derive(Debug, thiserror::Error)]
pub enum ConfirmationParseError {
    #[error("Pdf image conversion error: {0}")]
    PDFRenderError(#[from] PDF2ImageError),
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),
    #[error("Image render options error: {0}")]
    ImageRenderError(String),
}
