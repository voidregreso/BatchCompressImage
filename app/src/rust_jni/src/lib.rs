#![allow(non_snake_case)]
extern crate alloc;
use jni::objects::{JClass, JObject};
use jni::sys::{jbyteArray};
use jni::JNIEnv;

#[cfg(target_os = "android")]
extern crate android_logger;
extern crate log;
use log::Level;

use alloc::ffi::CString;
use std::error::Error;
use std::ffi::CStr;
use std::os::raw::c_char;
use jni::signature::{Primitive, ReturnType};
use crate::jpeg::ChromaSubsampling;

use crate::utils::{get_filetype, SupportedFileTypes};

#[cfg(target_os = "android")]
fn init_logger() {
    android_logger::init_once(
        android_logger::Config::default().with_min_level(Level::Info)
    );
}

mod jpeg;
mod png;
mod resize;
mod utils;
mod webp;

#[repr(C)]
pub struct CCSParameters {
    pub keep_metadata: bool,
    pub jpeg_quality: u32,
    pub jpeg_chroma_subsampling: u32,
    pub png_quality: u32,
    pub png_force_zopfli: bool,
    pub webp_quality: u32,
    pub optimize: bool,
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
pub struct CCSResult {
    pub success: bool,
    pub error_message: *const c_char,
}

#[derive(Copy, Clone)]
pub struct JpegParameters {
    pub quality: u32,
    pub chroma_subsampling: ChromaSubsampling
}

#[derive(Copy, Clone)]
pub struct PngParameters {
    pub quality: u32,
    pub force_zopfli: bool,
}

#[derive(Copy, Clone)]
pub struct WebPParameters {
    pub quality: u32,
}

#[derive(Copy, Clone)]
pub struct CSParameters {
    pub jpeg: JpegParameters,
    pub png: PngParameters,
    pub webp: WebPParameters,
    pub keep_metadata: bool,
    pub optimize: bool,
    pub width: u32,
    pub height: u32,
    pub output_size: u32,
}

pub fn initialize_parameters() -> CSParameters {
    let jpeg = JpegParameters {
        quality: 80,
        chroma_subsampling: ChromaSubsampling::Auto
    };

    let png = PngParameters {
        quality: 80,
        force_zopfli: false,
    };

    let webp = WebPParameters { quality: 80 };

    CSParameters {
        jpeg,
        png,
        webp,
        keep_metadata: false,
        optimize: false,
        width: 0,
        height: 0,
        output_size: 0,
    }
}

fn c_return_result(result: Result<(), Box<dyn Error>>) -> CCSResult {
    let mut error_message = CString::new("").unwrap();

    match result {
        Ok(_) => {
            let em_pointer = error_message.as_ptr();
            std::mem::forget(error_message);
            CCSResult {
                success: true,
                error_message: em_pointer,
            }
        }
        Err(e) => {
            error_message = CString::new(e.to_string()).unwrap();
            let em_pointer = error_message.as_ptr();
            std::mem::forget(error_message);
            CCSResult {
                success: false,
                error_message: em_pointer,
            }
        }
    }
}

pub unsafe fn my_compress(
    inputData: Vec<u8>,
    outData: &mut Vec<u8>,
    params: CCSParameters,
) -> CCSResult {
    let mut parameters = initialize_parameters();

    parameters.jpeg.quality = params.jpeg_quality;
    parameters.png.quality = params.png_quality;
    parameters.optimize = params.optimize;
    parameters.keep_metadata = params.keep_metadata;
    parameters.png.force_zopfli = params.png_force_zopfli;
    parameters.webp.quality = params.webp_quality;
    parameters.width = params.width;
    parameters.height = params.height;

    c_return_result(compress(
        inputData,
        outData,
        &parameters,
    ))
}

pub fn compress(
    inputData: Vec<u8>,
    outData: &mut Vec<u8>,
    parameters: &CSParameters,
) -> Result<(), Box<dyn Error>> {
    validate_parameters(parameters)?;
    let file_type = get_filetype(inputData.clone());

    match file_type {
        SupportedFileTypes::Jpeg => {
            jpeg::compress(inputData.clone(), outData, parameters)?;
        }
        SupportedFileTypes::Png => {
            png::compress(inputData.clone(), outData, parameters)?;
        }
        SupportedFileTypes::WebP => {
            webp::compress(inputData.clone(), outData, parameters)?;
        }
        _ => return Err("Unknown file type".into()),
    }

    Ok(())
}

fn validate_parameters(parameters: &CSParameters) -> Result<(), Box<dyn Error>> {
  /*  println!("{}", format!("JPG Quality = {:?}, PNG Quality = {:?}, WEBP Quality = {:?}"
                     , parameters.jpeg.quality, parameters.png.quality, parameters.webp.quality));*/

    if parameters.jpeg.quality == 0 || parameters.jpeg.quality > 100 {
        return Err("Invalid JPEG quality value".into());
    }

    if parameters.png.quality > 100 {
        return Err("Invalid PNG quality value".into());
    }

    if parameters.webp.quality > 100 {
        return Err("Invalid WebP quality value".into());
    }

    Ok(())
}

#[no_mangle]
pub unsafe extern "system" fn  Java_com_luis_bci_CaesiumNative_compressPic(
    env: JNIEnv,
    _clz: JClass,
    inBytes: jbyteArray,
    conf: JObject,
) -> jbyteArray {
    #[cfg(target_os = "android")]
    init_logger();
    #[cfg(not(target_os = "android"))]
    simple_logger::init_with_level(Level::Info).unwrap();

    log::info!("Received bytes from Java with length {}", env.get_array_length(inBytes).unwrap());

    let cl = env.find_class("com/luis/bci/CCSParameter").unwrap();
    // Get field ID from class CCSParameter
    let f_keep_metadata = env.get_field_id(cl, "keep_metadata", "Z").unwrap();
    let f_jpeg_quality = env.get_field_id(cl, "jpeg_qu", "I").unwrap();
    let f_png_quality = env.get_field_id(cl, "png_qu", "I").unwrap();
    let f_png_force_zopfli = env.get_field_id(cl, "png_force_zopfli", "Z").unwrap();
    let f_webp_quality = env.get_field_id(cl, "webp_qu", "I").unwrap();
    let f_optimize = env.get_field_id(cl, "optm", "Z").unwrap();
    let f_width = env.get_field_id(cl, "width", "I").unwrap();
    let f_height = env.get_field_id(cl, "height", "I").unwrap();

    // Read field value from by ID of class CCSParameter
    let km: bool = env.get_field_unchecked(
        conf,
        f_keep_metadata,
        ReturnType::Primitive(Primitive::Boolean),
    ).unwrap().z().unwrap();
    let jq: u32 = env.get_field_unchecked(
        conf,
        f_jpeg_quality,
        ReturnType::Primitive(Primitive::Int),
    ).unwrap().i().unwrap() as u32;
    let pq: u32 = env.get_field_unchecked(
        conf,
        f_png_quality,
        ReturnType::Primitive(Primitive::Int),
    ).unwrap().i().unwrap() as u32;
    let pfz: bool = env.get_field_unchecked(
        conf,
        f_png_force_zopfli,
        ReturnType::Primitive(Primitive::Boolean),
    ).unwrap().z().unwrap();
    let wq: u32 = env.get_field_unchecked(
        conf,
        f_webp_quality,
        ReturnType::Primitive(Primitive::Int),
    ).unwrap().i().unwrap() as u32;
    let opt: bool = env.get_field_unchecked(
        conf,
        f_optimize,
        ReturnType::Primitive(Primitive::Boolean),
    ).unwrap().z().unwrap();
    let ww: u32 = env.get_field_unchecked(
        conf,
        f_width,
        ReturnType::Primitive(Primitive::Int),
    ).unwrap().i().unwrap() as u32;
    let hh: u32 = env.get_field_unchecked(
        conf,
        f_height,
        ReturnType::Primitive(Primitive::Int),
    ).unwrap().i().unwrap() as u32;

    let f_subsamp_mode = env.get_field_id(
        cl, "subsamp_mode", "Lcom/luis/bci/CCSParameter$ChromaSubsampling;").unwrap();

    let subsamp_mode_obj = env.get_field_unchecked(
        conf, f_subsamp_mode, ReturnType::Object).unwrap().l().unwrap();
    let ordinal_method_id = env.get_method_id("java/lang/Enum", "ordinal", "()I").unwrap();
    let subsamp_mode_ord = env.call_method_unchecked(
        subsamp_mode_obj, ordinal_method_id, ReturnType::Primitive(Primitive::Int), &[])
        .unwrap().i()
        .unwrap() as u32;

    // Fill parameters
    let params = CCSParameters {
        keep_metadata: km,
        jpeg_quality: jq,
        jpeg_chroma_subsampling: subsamp_mode_ord,
        png_quality: pq,
        png_force_zopfli: pfz,
        webp_quality: wq,
        optimize: opt,
        width: ww,
        height: hh,
    };

    // Convert Java byte array to Rust Vec<u8>
    let data1 = env.convert_byte_array(inBytes).unwrap();
    let mut data2: Vec<u8> = vec![];
    let res = unsafe {
        my_compress(data1, &mut data2, params)
    };

    if res.success {
        log::info!("Compression succeeded with final size = {}!", data2.len());
        env.byte_array_from_slice(&data2).unwrap()
    } else {
        let msg = CStr::from_ptr(res.error_message).to_str().unwrap();
        log::error!("Compression was not successful because: {}", msg);
        std::ptr::null_mut()
    }
}
