fn main() {
    // print qrcode
    qrcoce_gen::qr_print("https://rust-lang.org/").unwrap();

    // print qrcode unicode string
    let string = qrcoce_gen::qr_string("https://rust-lang.org/").unwrap();
    println!("{}", string);

    // print qrcode u8 arr
    let u8_arr = qrcoce_gen::qr_bytes("https://rust-lang.org/").unwrap();
    println!("{:?}", u8_arr);

    // print qrcode svg String
    let svg = qrcoce_gen::qr_svg("https://rust-lang.org/");
    println!("{:?}", svg);

    // qrcode image write to /tmp/qrcode.png
    qrcoce_gen::qr_image("https://rust-lang.org/", "/tmp/qrcode.png");
}
