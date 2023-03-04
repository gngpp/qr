# qrcode_gen
Rust QR Code Generic Generator

### Example 1
```rust
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
```

```rust
fn main() {
    // print qrcode
    qrcode_gen::qr_print("https://rust-lang.org/").unwrap();

    // print qrcode unicode string
    let string = qrcode_gen::qr_string("https://rust-lang.org/").unwrap();
    println!("{}", string);

    // print qrcode u8 arr
    let u8_arr = qrcode_gen::qr_bytes("https://rust-lang.org/").unwrap();
    println!("{:?}", u8_arr);

    // print qrcode svg String
    let svg = qrcode_gen::qr_svg("https://rust-lang.org/");
    println!("{:?}", svg);

    // qrcode image write to /tmp/qrcode.png
    qrcode_gen::qr_image("https://rust-lang.org/", "/tmp/qrcode.png");
}

```