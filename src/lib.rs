use image::{GenericImage, GenericImageView, ImageBuffer, Pixel, Primitive, Rgba};
use num_traits::NumCast;

#[derive(Clone, Copy)]
pub enum CompositeOperator {
    Blend,
    // Porter-Duff
    Clear,
    Copy,
    Dest,
    SrcOver,
    DestOver,
    SrcIn,
    DestIn,
    SrcOut,
    DestOut,
    SrcAtop,
    DestAtop,
    Xor,
}

pub fn composite<I, J>(bottom: &mut I, top: &J, offset_x: i64, offset_y: i64, op: CompositeOperator)
where
    I: GenericImage,
    J: GenericImage<Pixel = I::Pixel>,
{
    fn porter_duff_composite<T: Primitive>(
        dest: &mut Rgba<T>,
        src: &Rgba<T>,
        op: CompositeOperator,
    ) {
        let (rs, gs, bs, alphas) = (
            src.0[0].to_f32().unwrap(),
            src.0[1].to_f32().unwrap(),
            src.0[2].to_f32().unwrap(),
            src.0[3].to_f32().unwrap() / 255.0,
        );
        let (rd, gd, bd, alphad) = (
            dest.0[0].to_f32().unwrap(),
            dest.0[1].to_f32().unwrap(),
            dest.0[2].to_f32().unwrap(),
            dest.0[3].to_f32().unwrap() / 255.0,
        );

        let (fs, fd) = match op {
            CompositeOperator::Clear => (0.0, 0.0),
            CompositeOperator::Copy => (1.0, 0.0),
            CompositeOperator::Dest => (0.0, 1.0),
            CompositeOperator::SrcOver => (1.0, 1.0 - alphas),
            CompositeOperator::DestOver => (1.0 - alphad, 1.0),
            CompositeOperator::SrcIn => (alphad, 0.0),
            CompositeOperator::DestIn => (0.0, alphas),
            CompositeOperator::SrcOut => (1.0 - alphad, 0.0),
            CompositeOperator::DestOut => (0.0, 1.0 - alphas),
            CompositeOperator::SrcAtop => (alphad, 1.0 - alphas),
            CompositeOperator::DestAtop => (1.0 - alphad, alphas),
            CompositeOperator::Xor => (1.0 - alphad, 1.0 - alphas),
            _ => unreachable!(),
        };

        let r0 = alphas * rs * fs + alphad * rd * fd;
        let g0 = alphas * gs * fs + alphad * gd * fd;
        let b0 = alphas * bs * fs + alphad * bd * fd;
        let alpha0 = (alphas + alphad * (1.0 - alphas)) * 255.0;

        *dest = Rgba::from([
            NumCast::from(r0).unwrap(),
            NumCast::from(g0).unwrap(),
            NumCast::from(b0).unwrap(),
            NumCast::from(alpha0).unwrap(),
        ]);
    }

    let (bottom_w, bottom_h) = bottom.dimensions();
    let (top_w, top_h) = top.dimensions();

    if offset_x >= bottom_w as i64
        || offset_x + (top_w as i64) < 0
        || offset_y >= bottom_h as i64
        || offset_y + (top_h as i64) < 0
    {
        return;
    }

    for y in 0..bottom_h {
        for x in 0..bottom_w {
            let rel_x = (x as i64) - offset_x;
            let rel_y = (y as i64) - offset_y;
            if rel_x >= bottom_w as i64 || rel_x < 0 || rel_y >= bottom_h as i64 || rel_y < 0 {
                continue;
            }

            let mut bottom_pixel = bottom.get_pixel(x, y);

            match op {
                CompositeOperator::Blend => {
                    if rel_x >= top_w as i64 || rel_y >= top_h as i64 {
                        continue;
                    }
                    let p = top.get_pixel(rel_x as u32, rel_y as u32);
                    bottom_pixel.blend(&p);
                    bottom.put_pixel(x, y, bottom_pixel);
                }
                CompositeOperator::Clear
                | CompositeOperator::Copy
                | CompositeOperator::Dest
                | CompositeOperator::DestAtop
                | CompositeOperator::DestIn
                | CompositeOperator::DestOut
                | CompositeOperator::DestOver
                | CompositeOperator::SrcAtop
                | CompositeOperator::SrcIn
                | CompositeOperator::SrcOut
                | CompositeOperator::Xor
                | CompositeOperator::SrcOver => {
                    let rgba = if rel_x >= top_w as i64 || rel_y >= top_h as i64 {
                        Rgba::from([
                            NumCast::from(0).unwrap(),
                            NumCast::from(0).unwrap(),
                            NumCast::from(0).unwrap(),
                            NumCast::from(0).unwrap(),
                        ])
                    } else {
                        let p = top.get_pixel(rel_x as u32, rel_y as u32);
                        p.to_rgba()
                    };
                    let mut bottom_rgba = bottom_pixel.to_rgba();
                    porter_duff_composite(&mut bottom_rgba, &rgba, op);
                    let channel = &[
                        bottom_rgba[0],
                        bottom_rgba[1],
                        bottom_rgba[2],
                        bottom_rgba[3],
                    ];
                    let pixel = Pixel::from_slice(channel);
                    bottom.put_pixel(x, y, *pixel);
                }
            }
        }
    }
}

pub fn extent<I>(
    img: &I,
    width: u32,
    height: u32,
    offset_x: i64,
    offset_y: i64,
) -> ImageBuffer<
    <I as GenericImageView>::Pixel,
    Vec<<<I as GenericImageView>::Pixel as Pixel>::Subpixel>,
>
where
    I: GenericImage,
{
    let p = img.get_pixel(0, 0);
    let mut extent_img = ImageBuffer::from_pixel(width, height, p);

    composite(
        &mut extent_img,
        img,
        -offset_x,
        -offset_y,
        CompositeOperator::Copy,
    );

    extent_img
}
