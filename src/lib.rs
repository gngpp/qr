use std::ops::Index;
pub mod bits;
pub mod canvas;
mod cast;
pub mod ec;
pub mod matrix;
pub(crate) mod optimize;
pub mod render;
pub mod render_term;
pub mod types;
pub(crate) mod util;

pub use crate::types::{Color, EcLevel, QrResult, Version};

use crate::cast::As;
use checked_int_cast::CheckedIntCast;
use image::Luma;
use matrix::Matrix;
use render::{svg, unicode, Pixel, Renderer};
use render_term::RendererTerminal;
use types::QrError;

/// The encoded QR code symbol.
#[derive(Clone)]
pub struct QrCode {
    content: Vec<Color>,
    version: Version,
    ec_level: EcLevel,
    width: usize,
}

impl QrCode {
    /// Constructs a new QR code which automatically encodes the given data.
    ///
    /// This method uses the "medium" error correction level and automatically
    /// chooses the smallest QR code.
    ///
    ///     use qrcode::QrCode;
    ///
    ///     let code = QrCode::new(b"Some data").unwrap();
    ///
    /// # Errors
    ///
    /// Returns error if the QR code cannot be constructed, e.g. when the data
    /// is too long.
    pub fn new<D: AsRef<[u8]>>(data: D) -> QrResult<Self> {
        Self::with_error_correction_level(data, EcLevel::M)
    }

    /// Constructs a new QR code which automatically encodes the given data at a
    /// specific error correction level.
    ///
    /// This method automatically chooses the smallest QR code.
    ///
    ///     use qrcode::{QrCode, EcLevel};
    ///
    ///     let code = QrCode::with_error_correction_level(b"Some data", EcLevel::H).unwrap();
    ///
    /// # Errors
    ///
    /// Returns error if the QR code cannot be constructed, e.g. when the data
    /// is too long.
    pub fn with_error_correction_level<D: AsRef<[u8]>>(
        data: D,
        ec_level: EcLevel,
    ) -> QrResult<Self> {
        let bits = bits::encode_auto(data.as_ref(), ec_level)?;
        Self::with_bits(bits, ec_level)
    }

    /// Constructs a new QR code for the given version and error correction
    /// level.
    ///
    ///     use qrcode::{QrCode, Version, EcLevel};
    ///
    ///     let code = QrCode::with_version(b"Some data", Version::Normal(5), EcLevel::M).unwrap();
    ///
    /// This method can also be used to generate Micro QR code.
    ///
    ///     use qrcode::{QrCode, Version, EcLevel};
    ///
    ///     let micro_code = QrCode::with_version(b"123", Version::Micro(1), EcLevel::L).unwrap();
    ///
    /// # Errors
    ///
    /// Returns error if the QR code cannot be constructed, e.g. when the data
    /// is too long, or when the version and error correction level are
    /// incompatible.
    pub fn with_version<D: AsRef<[u8]>>(
        data: D,
        version: Version,
        ec_level: EcLevel,
    ) -> QrResult<Self> {
        let mut bits = bits::Bits::new(version);
        bits.push_optimal_data(data.as_ref())?;
        bits.push_terminator(ec_level)?;
        Self::with_bits(bits, ec_level)
    }

    /// Constructs a new QR code with encoded bits.
    ///
    /// Use this method only if there are very special need to manipulate the
    /// raw bits before encoding. Some examples are:
    ///
    /// * Encode data using specific character set with ECI
    /// * Use the FNC1 modes
    /// * Avoid the optimal segmentation algorithm
    ///
    /// See the `Bits` structure for detail.
    ///
    ///     #![allow(unused_must_use)]
    ///
    ///     use qrcode::{QrCode, Version, EcLevel};
    ///     use qrcode::bits::Bits;
    ///
    ///     let mut bits = Bits::new(Version::Normal(1));
    ///     bits.push_eci_designator(9);
    ///     bits.push_byte_data(b"\xca\xfe\xe4\xe9\xea\xe1\xf2 QR");
    ///     bits.push_terminator(EcLevel::L);
    ///     let qrcode = QrCode::with_bits(bits, EcLevel::L);
    ///
    /// # Errors
    ///
    /// Returns error if the QR code cannot be constructed, e.g. when the bits
    /// are too long, or when the version and error correction level are
    /// incompatible.
    pub fn with_bits(bits: bits::Bits, ec_level: EcLevel) -> QrResult<Self> {
        let version = bits.version();
        let data = bits.into_bytes();
        let (encoded_data, ec_data) = ec::construct_codewords(&*data, version, ec_level)?;
        let mut canvas = canvas::Canvas::new(version, ec_level);
        canvas.draw_all_functional_patterns();
        canvas.draw_data(&*encoded_data, &*ec_data);
        let canvas = canvas.apply_best_mask();
        Ok(Self {
            content: canvas.into_colors(),
            version,
            ec_level,
            width: version.width().as_usize(),
        })
    }

    /// Gets the version of this QR code.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Gets the error correction level of this QR code.
    pub fn error_correction_level(&self) -> EcLevel {
        self.ec_level
    }

    /// Gets the number of modules per side, i.e. the width of this QR code.
    ///
    /// The width here does not contain the quiet zone paddings.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Gets the maximum number of allowed erratic modules can be introduced
    /// before the data becomes corrupted. Note that errors should not be
    /// introduced to functional modules.
    pub fn max_allowed_errors(&self) -> usize {
        ec::max_allowed_errors(self.version, self.ec_level).expect("invalid version or ec_level")
    }

    /// Checks whether a module at coordinate (x, y) is a functional module or
    /// not.
    pub fn is_functional(&self, x: usize, y: usize) -> bool {
        let x = x
            .as_i16_checked()
            .expect("coordinate is too large for QR code");
        let y = y
            .as_i16_checked()
            .expect("coordinate is too large for QR code");
        canvas::is_functional(self.version, self.version.width(), x, y)
    }

    /// Converts the QR code into a human-readable string. This is mainly for
    /// debugging only.
    pub fn to_debug_str(&self, on_char: char, off_char: char) -> String {
        self.render()
            .quiet_zone(false)
            .dark_color(on_char)
            .light_color(off_char)
            .build()
    }

    /// Converts the QR code to a vector of booleans. Each entry represents the
    /// color of the module, with "true" means dark and "false" means light.
    #[deprecated(since = "0.4.0", note = "use `to_colors()` instead")]
    pub fn to_vec(&self) -> Vec<bool> {
        self.content.iter().map(|c| *c != Color::Light).collect()
    }

    /// Converts the QR code to a vector of booleans. Each entry represents the
    /// color of the module, with "true" means dark and "false" means light.
    #[deprecated(since = "0.4.0", note = "use `into_colors()` instead")]
    pub fn into_vec(self) -> Vec<bool> {
        self.content
            .into_iter()
            .map(|c| c != Color::Light)
            .collect()
    }

    /// Converts the QR code to a vector of colors.
    pub fn to_colors(&self) -> Vec<Color> {
        self.content.clone()
    }

    /// Converts the QR code to a vector of colors.
    pub fn into_colors(self) -> Vec<Color> {
        self.content
    }

    /// Renders the QR code into an image. The result is an image builder, which
    /// you may do some additional configuration before copying it into a
    /// concrete image.
    ///
    /// # Examples
    ///
    #[cfg_attr(feature = "image", doc = " ```rust")]
    #[cfg_attr(not(feature = "image"), doc = " ```ignore")]
    /// # use qrcode::QrCode;
    /// # use image::Rgb;
    ///
    /// let image = QrCode::new(b"hello").unwrap()
    ///                     .render()
    ///                     .dark_color(Rgb([0, 0, 128]))
    ///                     .light_color(Rgb([224, 224, 224])) // adjust colors
    ///                     .quiet_zone(false)          // disable quiet zone (white border)
    ///                     .min_dimensions(300, 300)   // sets minimum image size
    ///                     .build();
    /// ```
    ///
    /// Note: the `image` crate itself also provides method to rotate the image,
    /// or overlay a logo on top of the QR code.
    pub fn render<P: Pixel>(&self) -> Renderer<P> {
        let quiet_zone = if self.version.is_micro() { 2 } else { 4 };
        Renderer::new(&self.content, self.width, quiet_zone)
    }
}

impl Index<(usize, usize)> for QrCode {
    type Output = Color;

    fn index(&self, (x, y): (usize, usize)) -> &Color {
        let index = y * self.width + x;
        &self.content[index]
    }
}

const QUIET_ZONE_WIDTH: usize = 2;

/// Print the given `data` as QR code in the terminal.
///
/// Returns an error if generating the QR code failed.
///
/// # Examples
///
/// ```rust
/// qrcode_term::qr_print("https://rust-lang.org/").unwrap();
/// ```
///
/// # Panics
///
/// Panics if printing the QR code to the terminal failed.
pub fn qr_print<D: AsRef<[u8]>>(data: D) -> Result<(), QrError> {
    // Generate QR code pixel matrix
    let mut matrix = Qr::from(data)?.to_matrix();
    matrix.surround(QUIET_ZONE_WIDTH, render_term::QrLight);

    // Render QR code to stdout
    RendererTerminal::default().print_stdout(&matrix);
    Ok(())
}

/// Generate `String` from the given `data` as QR code.
///
/// Returns an error if generating the QR code failed.
///
/// # Examples
///
/// ```rust
/// let qr_string = qrcode_term::qr_string("https://rust-lang.org/").unwrap();
/// print!("{}", qr_string);
/// ```
///
/// # Panics
///
/// Panics if generating the QR code string failed.
pub fn qr_string<D: AsRef<[u8]>>(data: D) -> Result<String, QrError> {
    // Generate QR code pixel matrix
    let mut matrix = Qr::from(data)?.to_matrix();
    matrix.surround(QUIET_ZONE_WIDTH, render_term::QrLight);

    // Render QR code to a String
    let mut buf = Vec::new();
    RendererTerminal::default()
        .render(&matrix, &mut buf)
        .expect("failed to generate QR code string");
    Ok(String::from_utf8(buf).unwrap())
}

/// Generate `String` from the given `data` as QR code.
///
/// Returns an error if generating the QR code failed.
///
/// # Examples
///
/// ```rust
/// let u8_arr = qrcode_term::qr_bytes("https://rust-lang.org/").unwrap();
/// print!("{}", u8_arr);
/// ```
///
/// # Panics
///
/// Panics if generating the QR code string failed.
pub fn qr_bytes<D: AsRef<[u8]>>(data: D) -> Result<Vec<u8>, QrError> {
    let code = QrCode::new(data).unwrap();
    // unicode string qrcode
    let unicode_qrcode = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    Ok(Vec::from(unicode_qrcode.as_bytes()))
}

/// Generate `String` from the given `data` as QR code.
///
/// Returns an error if generating the QR code failed.
///
/// # Examples
///
/// ```rust
/// let svg_str = qrcode_term::qr_svg("https://rust-lang.org/").unwrap();
/// print!("{}", svg_str);
/// ```
///
/// # Panics
///
/// Panics if generating the QR code string failed.
pub fn qr_svg<D: AsRef<[u8]>>(data: D) -> Result<String, QrError> {
    let code = QrCode::with_version(data, Version::Normal(5), EcLevel::M).unwrap();
    let svg = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#800000"))
        .light_color(svg::Color("#ffff80"))
        .build();
    Ok(svg)
}

pub fn qr_image<D: AsRef<[u8]>>(data: D, path: &str) {
    // Image generation
    // Encode some data into bits.
    let code = QrCode::new(data).unwrap();
    // Render the bits into an image.
    let image = code.render::<Luma<u8>>().build();

    // Save the image.
    image.save(path).unwrap();
}

/// Raw QR code.
#[allow(missing_debug_implementations)]
pub struct Qr {
    code: QrCode,
}

impl Qr {
    /// Construct a new QR code.
    pub fn from<D: AsRef<[u8]>>(data: D) -> Result<Self, QrError> {
        Ok(Self {
            // TODO: error handle here!
            code: QrCode::new(data.as_ref())?,
        })
    }

    /// Create pixel matrix from this QR code.
    pub fn to_matrix(&self) -> Matrix<Color> {
        Matrix::new(self.code.to_colors())
    }
}

#[cfg(test)]
mod tests {
    use crate::{EcLevel, QrCode, Version};

    use super::*;

    #[test]
    fn test_api() {
        // 终端打印二维码
        qr_print("https://github.com/zf1976/pancli").unwrap();

        // 二维码字节数组
        let qrcode_bytes = qr_bytes("https://github.com/zf1976/pancli").unwrap();
        println!("{:?}", qrcode_bytes.as_slice());
    }

    /// Generating QR codes for text that is too large should fail.
    #[test]
    #[should_panic]
    fn print_qr_too_long() {
        Qr::from(&String::from_utf8(vec![b'a'; 8000]).unwrap()).unwrap();
    }

    #[test]
    fn test_annex_i_qr() {
        // This uses the ISO Annex I as test vector.
        let code = QrCode::with_version(b"01234567", Version::Normal(1), EcLevel::M).unwrap();
        assert_eq!(
            &*code.to_debug_str('#', '.'),
            "\
             #######..#.##.#######\n\
             #.....#..####.#.....#\n\
             #.###.#.#.....#.###.#\n\
             #.###.#.##....#.###.#\n\
             #.###.#.#.###.#.###.#\n\
             #.....#.#...#.#.....#\n\
             #######.#.#.#.#######\n\
             ........#..##........\n\
             #.#####..#..#.#####..\n\
             ...#.#.##.#.#..#.##..\n\
             ..#...##.#.#.#..#####\n\
             ....#....#.....####..\n\
             ...######..#.#..#....\n\
             ........#.#####..##..\n\
             #######..##.#.##.....\n\
             #.....#.#.#####...#.#\n\
             #.###.#.#...#..#.##..\n\
             #.###.#.##..#..#.....\n\
             #.###.#.#.##.#..#.#..\n\
             #.....#........##.##.\n\
             #######.####.#..#.#.."
        );
    }

    #[test]
    fn test_annex_i_micro_qr() {
        let code = QrCode::with_version(b"01234567", Version::Micro(2), EcLevel::L).unwrap();
        assert_eq!(
            &*code.to_debug_str('#', '.'),
            "\
             #######.#.#.#\n\
             #.....#.###.#\n\
             #.###.#..##.#\n\
             #.###.#..####\n\
             #.###.#.###..\n\
             #.....#.#...#\n\
             #######..####\n\
             .........##..\n\
             ##.#....#...#\n\
             .##.#.#.#.#.#\n\
             ###..#######.\n\
             ...#.#....##.\n\
             ###.#..##.###"
        );
    }
}
