use infer::Infer;

pub enum SupportedFileTypes {
    Jpeg,
    Png,
    WebP,
    Unkn,
}

pub fn get_filetype(data: Vec<u8>) -> SupportedFileTypes {
    let infer = Infer::new();
    match infer.get(&data) {
        None => {
            log::info!("No MIME type found, file type is unknown");
            SupportedFileTypes::Unkn
        },
        Some(ft) => {
            let mime_type = ft.mime_type();
            log::info!("Detected MIME type: {}", mime_type);
            match mime_type {
                "image/jpeg" => SupportedFileTypes::Jpeg,
                "image/png" => SupportedFileTypes::Png,
                "image/webp" => SupportedFileTypes::WebP,
                _ => SupportedFileTypes::Unkn,
            }
        }
    }
}