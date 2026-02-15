pub enum ImageMime {
    Jpeg,
    Png,
    WebP,
    Heic,
    Heif,
}
impl ImageMime {
    pub fn name(&self) -> &'static str {
        match self {
            ImageMime::Jpeg => "image/jpeg",
            ImageMime::Png => "image/png",
            ImageMime::WebP => "image/webp",
            ImageMime::Heic => "image/heic",
            ImageMime::Heif => "image/heif",
        }
    }
    ///Try to automatically detect what kind of image it is
    pub fn try_detect_image_mime(bytes: &[u8]) -> Option<ImageMime> {
        if bytes.len() < 12 {
            return None;
        }

        // JPEG
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some(ImageMime::Jpeg);
        }

        // PNG
        if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
            return Some(ImageMime::Png);
        }

        // WebP: RIFF????WEBP
        if bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
            return Some(ImageMime::WebP);
        }

        // HEIC / HEIF (ISO Base Media)
        if &bytes[4..8] == b"ftyp" {
            match &bytes[8..12] {
                b"heic" | b"heix" => return Some(ImageMime::Heic),
                b"heif" | b"mif1" | b"msf1" => return Some(ImageMime::Heif),
                _ => {}
            }
        }

        None
    }
}
