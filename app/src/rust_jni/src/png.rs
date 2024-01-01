use std::{io, mem};
use std::num::NonZeroU8;

use image::ImageOutputFormat;
use lodepng::{decode32, Encoder};
use oxipng::Deflaters::{Libdeflater, Zopfli};

use crate::CSParameters;
use crate::resize::resize;

pub fn compress(
    in_file: Vec<u8>,
    out_buffer: &mut Vec<u8>,
    parameters: &CSParameters,
) -> Result<(), io::Error> {
    if parameters.width > 0 || parameters.height > 0 {
        let proc_in = resize(in_file, parameters.width, parameters.height, ImageOutputFormat::Png)?;
        let _ = mem::replace(out_buffer, compress_to_memory(proc_in, parameters)?);
    } else {
        let _ = mem::replace(out_buffer, compress_to_memory(in_file.clone(), parameters)?);
    }

    Ok(())
}

pub fn compress_to_memory(in_file: Vec<u8>, parameters: &CSParameters) -> Result<Vec<u8>, io::Error>
{
    let optimized_png: Vec<u8> = if parameters.optimize {
        lossless(in_file, parameters)?
    } else {
        lossy(in_file, parameters)?
    };

    Ok(optimized_png)
}

fn lossy(in_file: Vec<u8>, parameters: &CSParameters) -> Result<Vec<u8>, io::Error> {
    let rgba_bitmap = match decode32(in_file) {
        Ok(i) => i,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
    };

    let mut liq = imagequant::new();
    match liq.set_quality(0, parameters.png.quality as u8) {
        Ok(()) => {}
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
    }

    let mut liq_image = match liq.new_image(
        rgba_bitmap.buffer.as_slice(),
        rgba_bitmap.width,
        rgba_bitmap.height,
        0.0,
    ) {
        Ok(i) => i,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
    };

    let mut quantization = match liq.quantize(&mut liq_image) {
        Ok(q) => q,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
    };

    let (palette, pixels) = match quantization.remapped(&mut liq_image) {
        Ok((pl, px)) => (pl, px),
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
    };

    let mut encoder = Encoder::new();
    match encoder.set_palette(palette.as_slice()) {
        Ok(_) => {}
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
    }
    let png_vec = match encoder.encode(pixels.as_slice(), rgba_bitmap.width, rgba_bitmap.height) {
        Ok(pv) => pv,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
    };

    Ok(png_vec)
}

fn lossless(in_file: Vec<u8>, parameters: &CSParameters) -> Result<Vec<u8>, io::Error> {
    // let in_file = fs::read(input_path)?;
    let mut oxipng_options = oxipng::Options::default();
    if !parameters.keep_metadata {
        oxipng_options.strip = oxipng::Headers::Safe;
    }

    if parameters.optimize && parameters.png.force_zopfli {
        oxipng_options.deflate = Zopfli {
            iterations: NonZeroU8::new(15).unwrap(),
        };
    } else {
        oxipng_options = oxipng::Options::from_preset(3);
        oxipng_options.deflate = Libdeflater { compression: 6 };
    }

    let optimized_png = match oxipng::optimize_from_memory(in_file.as_slice(), &oxipng_options) {
        Ok(o) => o,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, e)),
    };

    Ok(optimized_png)
}
