fn main() {
    // print qrcode
    qr::qr_print("https://rust-lang.org/").unwrap();

    // print qrcode unicode string
    let string = qr::qr_string("https://rust-lang.org/").unwrap();
    println!("{}", string);

    // print qrcode u8 arr
    let u8_arr = qr::qr_bytes("https://rust-lang.org/").unwrap();
    println!("{:?}", u8_arr);

    // print qrcode svg String
    let svg = qr::qr_svg("https://rust-lang.org/");
    println!("{:?}", svg);

    // qrcode image write to /tmp/qrcode.png
    qr::qr_image("https://rust-lang.org/", "/tmp/qrcode.png");
}
