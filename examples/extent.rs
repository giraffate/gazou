use gazou::extent;
use image::io::Reader as ImageReader;

fn main() {
    let img = ImageReader::open("assets/green.png")
        .unwrap()
        .decode()
        .unwrap();

    let img = extent(&img, 1024, 512, 0, 0);
    img.save("assets/result.png").unwrap();
}
