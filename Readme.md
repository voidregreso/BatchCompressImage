# Batch Image Compressor

This Android app provides a convenient way to batch compress images in a selected folder for various image formats, including JPEG, PNG, and WebP, and convert all non-JPEG images to JPEG. The app leverages Kotlin for the Android frontend and integrates a Rust JNI module for efficient image compression.

## Features

- Batch compression of images in a selected folder
- Support for JPEG, PNG, and WebP compression algorithms
- Transparent PNG images are identified and excluded from JPEG conversion
- Adjustable compression quality settings for each algorithm
- Effortlessly compress images with no extra setup. Open the app, select the directory, and batch compress images instantly.

## Prerequisites

- Android device running Android 5.0 or higher
- If you need to build the native source code rather than using existing prebuilt dynamic libraries, [Rust](https://www.rust-lang.org/) should be installed on your development machine.

## Getting Started

1. Clone the repository to your local machine.
2. Open the project in Android Studio.
3. Build and run the app on an Android device or emulator.
4. Well, since the main code of libcaesium is written in Rust, getting this project compiled and configured can be a real pain. So, I just went ahead and modified the c_compress() function in the original author's project file, then wrote a JNI function to call it. After all that, I compiled it into a dynamic link library file fer both arm32 and arm64 architectures. You can just use them straight up in your projects. Now, if you wanna improve the JNI part of the function, or if you need ta compile the dynamic link library fer the x86 architecture Android system, you're gonna hafta do it all from scratch. Here are the brief steps:

* Download NDK r21 from repository: https://github.com/android/ndk/wiki/Unsupported-Downloads
* Configure AR and CC path under [target.<arch>] in .cargo/config.toml, where <arch> can be one of them: *armv7-linux-androideabi, aarch64-linux-android, i686-linux-android, x86_64-linux-android*. Once they're got done, modify path after *-L* in *rustflags* to corresponding android library path. 
* Set additional environment variables to prevent the clang compiler from not finding the path to the corresponding toolchain:

```
path D:\android-ndk-r21e\toolchains\llvm\prebuilt\windows-x86_64\bin;%PATH%
```

* Install Install the rust build toolchain corresponding to the target architecture:

```
rustup target install aarch64-linux-android
```

* Go to build:

```
cargo build --target aarch64-linux-android --release
```


## Usage

1. Launch the app on your Android device.
2. Grant necessary permissions for accessing external storage if prompted.
3. Select a target folder for batch image compression.
4. The app will identify image files in the selected folder and initiate the compression process.
5. Upon completion, a toast message will indicate that the compression is finished.

## License

This project is licensed under the [MIT License](LICENSE).

## Acknowledgments

- The Rust JNI module for image compression is based on the [Caesium Rust library](https://github.com/Luis-lp-caesium/caesium).
- Special thanks to contributors and open-source projects that inspired this work.
