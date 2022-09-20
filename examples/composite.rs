use gazou::{composite, CompositeOperator};
use image::io::Reader as ImageReader;

fn main() {
    let mut img1 = ImageReader::open("assets/red.png")
        .unwrap()
        .decode()
        .unwrap();
    let img2 = ImageReader::open("assets/green.png")
        .unwrap()
        .decode()
        .unwrap();

    composite(&mut img1, &img2, 0, 0, CompositeOperator::SrcOver);
    img1.save("assets/result.png").unwrap();
}
