//! The xray crate provides utilities for performing integration tests on 
//! graphical applications, such as games.
//! 
//! For the most basic usage of this libray, you may use one of the utility functions
//! that will perform a screenshot test with the default settings for your library. Currently, the only
//! implemented utility method is `gl_screenshot_test` which will capture a screenshot using OpenGL
//! of the specified region, and compare it to a reference screenshot loaded from 
//! `references/<test name>.png`.
//! 
//! To customise this behaviour, you should call `screenshot_test` (returns a `Result<(), XrayError>`)
//! or `assert_screenshot_test` (panics on failure) with a custom `ScreenshotIo` and `ScreenshotCaptor`.
//! 
//! 1. You may customise the method by which reference images are read, 
//!    or output images are written by providing a custom `ScreenshotIo`. 
//!    * For basic customisation of paths, you may create a new instance of `FsScreenshotIo` and pass
//!      it your paths of choice.
//!    * For more extensive customisation (e.g. using a web service to store screenshots), you may provide
//!      a custom implementation of `ScreenshotIo`.
//! 2. You may customise the method by which screenshots are taken. This is done by providing a custom implementation
//!    of `ScreenshotCaptor`

#[cfg(feature = "gl")]
extern crate gl;
extern crate image;

use std::borrow::ToOwned;
use std::fmt;
use std::fs as fs;
use std::fs::File;
use std::os::raw::c_void;
use std::path::{Path,PathBuf};
use std::result::Result;

use image::{GenericImage, ImageBuffer, ImageError, ImageFormat, Rgba};

pub use image::DynamicImage;

/// Errors that occur while loading reference images
/// or writing the output images.
pub enum IoError {
    OutputLocationUnavailable(String),
    FailedWritingScreenshot(String, String),
    FailedLoadingReferenceImage
}

/// Errors that occur with the screenshot comparison
pub enum ScreenshotError {
    NoReferenceScreenshot(DynamicImage),
    ScreenshotMismatch(DynamicImage, DynamicImage)
}

/// Reasons that a test could fail.
pub enum XrayError {
    Io(IoError),
    CaptureError,
    Screenshot(ScreenshotError)
}

impl fmt::Display for XrayError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = match self {
            XrayError::Io(io_error) => match io_error {
                IoError::OutputLocationUnavailable(location) => format!("Could not write to output location: {}", location),
                IoError::FailedWritingScreenshot(name, reason) => format!("Could not write screenshot {}:\n{}", name, reason),
                FailedLoadingReferenceImage => format!("Reference image could not be loaded or parsed")
            },
            XrayError::CaptureError => "Could not take screenshot.".to_string(),
            XrayError::Screenshot(screenshot_error) => match screenshot_error {
                ScreenshotError::NoReferenceScreenshot(_) => "No reference screenshot found.".to_string(),
                ScreenshotError::ScreenshotMismatch(_, _) => "Actual screenshot did not match expected screenshot.".to_string(),
            }
        };
        write!(f, "{}", text)
    }
}

type XrayResult<T> = Result<T, XrayError>;

/// Load reference images for comparison and store image output in the event of a failed test. 
/// 
/// `xray` ships with one implementation
/// of ScreenshotIo by default, `FsScreenshotIo` which reads and writes screenshots from the filesystem. 
pub trait ScreenshotIo {
    /// Performs any work needed to prepare to store output (e.g. creating directories)
    /// This gets called once per test, before any output is written.
    fn prepare_output(&self) -> XrayResult<()>;
    /// Loads a reference image to compare to the screenshot taken by xray. The actual method
    /// used to load the image depends on your chosen implementation.
    fn load_reference(&self) -> XrayResult<DynamicImage>;
    /// Writes out the screenshot taken during the test in the event of a failed test.
    fn write_actual(&self, &DynamicImage) -> XrayResult<()>;
    /// Writes out the screenshot that was expected during the test in the event of a failed test.
    fn write_expected(&self, &DynamicImage) -> XrayResult<()>;
    /// Writes out an image containing only those pixels that were present in the newly captured image
    /// but not present in the reference image.
    fn write_diff(&self, &DynamicImage) -> XrayResult<()>;

    /// Returns a default implementation of `ScreenshotIo`. 
    /// 
    /// This implementation will look for reference images in
    /// `references/<test_name>.png` at the top level of your crate.
    /// 
    /// In the event of a failed test, it will write out three screenshots.
    /// 
    /// * `test_output/<test_name>/actual.png` containing the screenshot taken during the test.
    /// * `test_output/<test_name>/expected.png` containing a copy of the reference image which the screenshot was compared against.
    /// * `test_output/<test_name>/diff.png` containing those pixels of the newly taken screenshot that did not match the pixels in the reference image.
    fn default(test_name: &str) -> FsScreenshotIo {
        FsScreenshotIo::new(test_name, "references", "test_output")
    }
}

/// Retrieves reference screenshots and stores debugging screenshots using the filesystem.
/// All images are in PNG format.
/// 
/// This implementation will look for reference images in `<references_path>/<test_name>.png` 
/// at the top level of your crate. Either of these may contain slashes to use subdirectories. 
/// For example, for a references_path `tests/reference_images`, and a test_name `basics/menu`
/// the library will look for a reference image in `tests/reference_images/basics/menu.png`.alloc
/// 
/// It will store output images in <output_path>/<test_name> at the top level of your crate. As with 
/// reference images, slashes may be used to use subdirectories. For example, given an output path
/// `target/screenshots` and a test_name `basics/menu`, the following images will be written:
/// 
/// * `target/screenshots/basics/menu/actual.png`
/// * `target/screenshots/basics/menu/expected.png`
/// * `target/screenshots/basics/menu/diff.png`
/// 
/// `actual.png` contains the screenshot taken for the test. 
pub struct FsScreenshotIo {
    references_path: PathBuf,
    output_path: PathBuf,
    test_name: String
}

/// Captures a region of the screen for comparison against a reference image.
pub trait ScreenshotCaptor {
    /// Takes a screenshot of the area (x, y, x + width, y + height)
    /// Returns a ScreenshotError::ErrorCapturingImage if the image could not be captured.
    fn capture_image(&self, x: i32, y: i32, width: u32, height: u32) -> XrayResult<DynamicImage>;
}

#[cfg(feature = "gl")]
/// Captures a screenshot using `gl::ReadPixels`
/// 
/// To use this screenshot captor, OpenGL must be able to 
/// load function pointers. If you use Piston or Glutin, this is likely already the case.
/// 
/// If you use a lower level library like `gl` directly, you may need to call
/// `gl::load_with(|symbol| glfw.get_proc_address(s)))`
/// or similar, depending on your choice of gl library and context library.
pub struct OpenGlScreenshotCaptor {
}

#[cfg(feature = "gl")]
impl ScreenshotCaptor for OpenGlScreenshotCaptor {
    fn capture_image(&self, x: i32, y: i32, width: u32, height: u32) -> XrayResult<DynamicImage> {
        let mut img = DynamicImage::new_rgba8(width, height);
        unsafe {
            let pixels = img.as_mut_rgba8().unwrap();
            gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            let height = height as i32;
            let width = width as i32;
            gl::ReadPixels(x, y, width, height, gl::RGBA, gl::UNSIGNED_BYTE, pixels.as_mut_ptr() as *mut c_void);
            let error_code = gl::GetError();
            if error_code != gl::NO_ERROR {
                return Err(XrayError::CaptureError);
            }
        }

        Ok(img)
    }
}

impl FsScreenshotIo {
    fn new<P: AsRef<Path>>(test_name: &str, references_path: P, output_path: P) -> FsScreenshotIo {
        FsScreenshotIo {
            references_path: references_path.as_ref().to_owned(),
            output_path: output_path.as_ref().to_owned(),
            test_name: test_name.to_string()
        }
    }

    fn write_image(&self, name: &str, img: &DynamicImage) -> XrayResult<()> {
        let filename = self.output_path.join(&self.test_name).join(name);
        let mut file = File::create(&filename).or(Err(XrayError::Io(IoError::FailedWritingScreenshot(
            filename.to_string_lossy().to_string(), 
            "Could not open file for writing".to_string()
        ))))?;
        img.write_to(&mut file, ImageFormat::PNG).or_else(
            |err| Err(XrayError::Io(IoError::FailedWritingScreenshot(
                filename.to_string_lossy().to_string(), 
                err.to_string())))
        )
    }
}

impl ScreenshotIo for FsScreenshotIo {
    fn prepare_output(&self) -> XrayResult<()> {
        fs::create_dir_all(&self.output_path.join(&self.test_name)).or(
            Err(XrayError::Io(IoError::OutputLocationUnavailable(self.output_path.to_string_lossy().to_string())))
        )
    }

    fn load_reference(&self) -> XrayResult<DynamicImage> {
        let full_path = self.references_path.join(format!("{}.png", &self.test_name));
        image::open(full_path).or(Err(XrayError::Io(IoError::FailedLoadingReferenceImage)))
    }

    fn write_actual(&self, actual: &DynamicImage) -> XrayResult<()> {
        self.write_image("actual.png", actual)
    }

    fn write_expected(&self, actual: &DynamicImage) -> XrayResult<()> {
        self.write_image("expected.png", actual)
    }

    fn write_diff(&self, actual: &DynamicImage) ->XrayResult<()> {
        self.write_image("diff.png", actual)
    }
}

fn compare_screenshot_images(reference_image: DynamicImage, actual_image: DynamicImage) -> XrayResult<()> {
    if reference_image.raw_pixels() == actual_image.raw_pixels() { 
        Ok(()) 
    } else { 
        Err(XrayError::Screenshot(ScreenshotError::ScreenshotMismatch(actual_image, reference_image)))
    }
}

/// Creates an image diff between two images.
/// 
/// This is done by creating an image of the size of the `actual` parameter,
/// and copying those pixels that are present in the `actual` image and different
/// to the same pixel in the `expected` image. 
/// 
/// All pixels which match in both input images will be transparent in the output image.
/// If the expected image is smaller than the actual image, all pixels outside the range
/// of the expected image are expected to be transparent.
pub fn diff_images(actual: &DynamicImage, expected: &DynamicImage) -> DynamicImage {
    DynamicImage::ImageRgba8(ImageBuffer::from_fn(
        actual.width(),
        actual.height(),
        |x, y| {
            let actual_pixel = actual.get_pixel(x, y);
            let expected_pixel = if expected.in_bounds(x, y) {
                expected.get_pixel(x, y)
            } else {               
                Rgba {
                    data: [0, 0, 0, 0]
                }
            };
            if actual_pixel == expected_pixel {
                Rgba {
                    data: [0, 0, 0, 0]
                }
            } else {
                actual_pixel
            }
        }
    ))
}

fn handle_screenshot_error<S: ScreenshotIo>(screenshot_io: S, screenshot_error: XrayError) -> XrayResult<()> {
    screenshot_io.prepare_output()?;
    match screenshot_error {
        XrayError::Screenshot(ScreenshotError::NoReferenceScreenshot(ref img)) => {
            screenshot_io.write_actual(&img)?;
        },
        XrayError::Screenshot(ScreenshotError::ScreenshotMismatch(ref actual, ref expected)) => {
            screenshot_io.write_expected(&expected)?;
            screenshot_io.write_actual(&actual)?;
            screenshot_io.write_diff(&diff_images(&actual, &expected))?;
        },
        _ => {}
    }
    Err(screenshot_error)
}

/// Tests the rendered image against the screenshot and returns a Ok(()) if they match, and a Err(ScreenshotError)
/// should the comparison not match or encounter an error.
/// 
/// The reference image is loaded using `screenshot_io.load_reference()`, 
/// while the test image is captured using `screenshot_captor.capture_image(x, y, width, height)`.
pub fn screenshot_test<S: ScreenshotIo, C: ScreenshotCaptor>(screenshot_io: S, screenshot_captor: C, x: i32, y: i32, width: u32, height: u32) -> XrayResult<()> {
    screenshot_captor.capture_image(x, y, width, height)
        .and_then(|captured_image| {
            match screenshot_io.load_reference() {
                Ok(reference_image) => Ok((reference_image, captured_image)),
                Err(_) => Err(XrayError::Screenshot(ScreenshotError::NoReferenceScreenshot(captured_image)))
            }
        })
        .and_then(|images| {
            let (reference_image, captured_image) = images;
            compare_screenshot_images(reference_image, captured_image)
        })
        .or_else(|err| handle_screenshot_error(screenshot_io, err))
        .and(Ok(()))
}

/// Tests the rendered image against a screenshot and panics if the images do
/// not match or are unable to be taken.
/// 
/// The reference image is loaded using `screenshot_io.load_reference()`, 
/// while the test image is captured using `screenshot_captor.capture_image(x, y, width, height)`.
pub fn assert_screenshot_test<S: ScreenshotIo, C: ScreenshotCaptor>(screenshot_io: S, screenshot_captor: C, x: i32, y: i32, width: u32, height: u32) {
    let result = screenshot_test(screenshot_io, screenshot_captor, x, y, width, height);
    if result.is_err() {
        panic!(format!("{}", result.unwrap_err()))
    }
}

/// Takes a screenshot using OpenGL and panics if it does not match a reference image.
/// 
/// The image of the given region is taken using OpenGL's gl::ReadImage.
/// 
/// This screenshot is compared to `references/<test_name>.png`
/// 
/// If the images do not match, or could not be taken, the call panics
/// and the following three screenshots are written out:
/// 
/// * test_output/<test_name>/actual.png containing the screenshot taken during the test
/// * test_output/<test_name>/expected.png containing the reference image the screenshot was compared against.
/// * test_output/<test_name>/diff.png containing the pixels from the screenshot that did not match the pixels in the reference image.
/// 
/// To customise any of this behaviour, create a custom `ScreenshotCaptor` and 
/// `ScreenshotIo` and pass them to `screenshot_test` (returns a `Result<(), ScreenshotError>`) 
/// or `assert_screenshot_test` (panics on fail)
#[cfg(feature = "gl")]
pub fn gl_screenshot_test(test_name: &str, x: i32, y: i32, width: u32, height: u32) {
    let fs_screenshot_io: FsScreenshotIo = FsScreenshotIo::default(test_name);
    let screenshot_captor = OpenGlScreenshotCaptor {};
    let result = assert_screenshot_test(fs_screenshot_io, screenshot_captor, x, y, width, height);
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::cell::RefCell;

    struct FakeScreenshotIo {
        reference_image: DynamicImage,
        actual: RefCell<Option<DynamicImage>>,
        expected: RefCell<Option<DynamicImage>>,
        diff: RefCell<Option<DynamicImage>>
    }

    impl FakeScreenshotIo {
        fn new(reference_image: DynamicImage) -> FakeScreenshotIo {
            FakeScreenshotIo {
                reference_image,
                actual: RefCell::new(None),
                expected: RefCell::new(None),
                diff: RefCell::new(None)
            }
        }
    }

    impl ScreenshotIo for FakeScreenshotIo {
        fn prepare_output(&self) -> XrayResult<()> { 
            Ok(()) 
        }

        fn load_reference(&self) -> XrayResult<DynamicImage> {
            Ok(self.reference_image.clone())
        }

        fn write_actual(&self, image: &DynamicImage) -> XrayResult<()> {
            self.actual.replace(Some(image.clone()));
            Ok(())
        }        
        
        fn write_expected(&self, image: &DynamicImage) -> XrayResult<()> {
            self.expected.replace(Some(image.clone()));
            Ok(())
        }        
        
        fn write_diff(&self, image: &DynamicImage) -> XrayResult<()> {
            self.diff.replace(Some(image.clone()));
            Ok(())
        }
    }

    struct FakeScreenshotCaptor {
        screenshot: DynamicImage
    }

    impl ScreenshotCaptor for FakeScreenshotCaptor {
        fn capture_image(&self, x: i32, y: i32, width: u32, height: u32) -> XrayResult<DynamicImage> {
            return Ok(self.screenshot.clone());
        }
    }

    #[test]
    fn test_diff_images() {
        let rgbw = DynamicImage::ImageRgba8(ImageBuffer::from_vec(2, 2, 
            vec![
                255, 0, 0, 255,
                0, 255, 0, 255,
                0, 0, 255, 255,
                255, 255, 255, 255
            ]
        ).unwrap());
        let rbgw = DynamicImage::ImageRgba8(ImageBuffer::from_vec(2, 2, 
            vec![
                255, 0, 0, 255,
                0, 0, 255, 255,
                0, 255, 0, 255,
                255, 255, 255, 255
            ]
        ).unwrap());
        let expected = DynamicImage::ImageRgba8(ImageBuffer::from_vec(2, 2, 
            vec![
                0, 0, 0, 0,
                0, 0, 255, 255,
                0, 255, 0, 255,
                0, 0, 0, 0
            ]
        ).unwrap());
        assert_eq!(diff_images(&rbgw, &rgbw).to_rgba().into_vec(), expected.to_rgba().into_vec())
    }

    #[test]
    fn test_success() {
        let rgbw = DynamicImage::ImageRgba8(ImageBuffer::from_vec(2, 2, 
            vec![
                255, 0, 0, 255,
                0, 255, 0, 255,
                0, 0, 255, 255,
                255, 255, 255, 255
            ]
        ).unwrap());
        let rbgw = DynamicImage::ImageRgba8(ImageBuffer::from_vec(2, 2, 
            vec![
                255, 0, 0, 255,
                0, 0, 255, 255,
                0, 255, 0, 255,
                255, 255, 255, 255
            ]
        ).unwrap());
        let expected = DynamicImage::ImageRgba8(ImageBuffer::from_vec(2, 2, 
            vec![
                0, 0, 0, 0,
                0, 0, 255, 255,
                0, 255, 0, 255,
                0, 0, 0, 0
            ]
        ).unwrap());
        let screenshot_io = FakeScreenshotIo::new(rgbw.clone());
        let screenshot_captor = FakeScreenshotCaptor { screenshot: rgbw.clone() };
        assert_screenshot_test(screenshot_io, screenshot_captor, 0, 0, 2, 2);
    }

    #[test]
    #[should_panic]
    fn test_fail() {
        let rgbw = DynamicImage::ImageRgba8(ImageBuffer::from_vec(2, 2, 
            vec![
                255, 0, 0, 255,
                0, 255, 0, 255,
                0, 0, 255, 255,
                255, 255, 255, 255
            ]
        ).unwrap());
        let rbgw = DynamicImage::ImageRgba8(ImageBuffer::from_vec(2, 2, 
            vec![
                255, 0, 0, 255,
                0, 0, 255, 255,
                0, 255, 0, 255,
                255, 255, 255, 255
            ]
        ).unwrap());
        let expected = DynamicImage::ImageRgba8(ImageBuffer::from_vec(2, 2, 
            vec![
                0, 0, 0, 0,
                0, 0, 255, 255,
                0, 255, 0, 255,
                0, 0, 0, 0
            ]
        ).unwrap());
        let screenshot_io = FakeScreenshotIo::new(rgbw.clone());
        let screenshot_captor = FakeScreenshotCaptor { screenshot: rbgw.clone() };
        assert_screenshot_test(screenshot_io, screenshot_captor, 0, 0, 2, 2);
    }
}