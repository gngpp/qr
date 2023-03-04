use image::Luma;
use qrcode_gen::{
    render::{svg, unicode},
    EcLevel, QrCode, Version,
};

fn main() {
    // Image generation
    // Encode some data into bits.
    let code = QrCode::new(b"01234567").unwrap();

    // Render the bits into an image.
    let image = code.render::<Luma<u8>>().build();

    // Save the image.
    image.save("/tmp/qrcode.png").unwrap();

    // String generation
    let code = QrCode::new(b"Hello").unwrap();
    let string = code
        .render::<char>()
        .quiet_zone(false)
        .module_dimensions(2, 1)
        .build();
    println!("{}", string);

    // SVG generation
    let code = QrCode::with_version(b"123", Version::Normal(5), EcLevel::M).unwrap();
    let image = code
        .render()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#800000"))
        .light_color(svg::Color("#ffff80"))
        .build();
    println!("{}", image);

    let micro_code = QrCode::with_version(b"123", Version::Micro(1), EcLevel::L).unwrap();
    let image = micro_code
        .render()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#800000"))
        .light_color(svg::Color("#ffff80"))
        .build();
    println!("{}", image);

    // Unicode string generation
    let code = QrCode::new("mow mow").unwrap();
    let image = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    println!("{}", image);
}