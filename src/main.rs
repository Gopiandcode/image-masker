extern crate argparse;
extern crate image;
use std::ops::Index;
use std::path::Path;
use std::fmt::{Display, Formatter};
use std::process::exit;

use argparse::{ArgumentParser, Store, StoreOption};

/// represents a region of an image - fields of the format (x,y,width,height)
#[derive(Debug,PartialEq,PartialOrd,Ord,Eq)]
pub struct Rect(u32, u32, u32, u32);

impl Rect {
    /// identifies whether a given point lies within a space
    pub fn contains(&self, (x,y): (u32, u32)) -> bool {
        x >= self.0 && x <= self.0 + self.2 &&
        y >= self.1 && y <= self.1 + self.3
    }

    /// Should only be called if the given point contains the rectangle
    pub fn skip(&self, (_,y): (u32, u32)) -> (u32, u32) {
        let n_x = self.0 + self.2 + 1;
        let n_y = y;

        (n_x, n_y)
    }
}

impl Display for Rect {
    fn fmt(&self, fmt:&mut Formatter) -> ::std::fmt::Result {
        write!(fmt, "{:?}", (self.0, self.1, self.2, self.3))
    }
}

/// represents a thresholded binary image where the threshhold is a non zero alpha value
pub struct ImageMask((u32,u32), Vec<bool>);

impl ImageMask {
    pub fn new(image: &image::GrayAlphaImage) -> Self {
        ImageMask(image.dimensions(), image.pixels().map(|p| p[1] > 0).collect())
    }

    pub fn dimensions(&self) -> (u32,u32) {
        self.0.clone()
    }
}

impl Index<(u32,u32)> for ImageMask {
    type Output = bool;

    fn index(&self, (x,y): (u32,u32)) -> &bool {
        &self.1[(x + y * (self.0).0) as usize]
    }
}

fn main() {

    let mut input_file_name : String = String::new();
    let mut output_file_name : Option<String> = None;

    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Uses marching squares algorithm to segment a binary image into multiple distinct rectangles. Uses coordinates starting (0,0) at the top left corner. Returns the result as a list of tuples");
        ap.refer(&mut input_file_name).add_argument("IMAGE", Store, "The image in any standard format to be processed").required();
        ap.refer(&mut output_file_name).metavar("OUTPUT").add_option(&["-o", "--output"], StoreOption, "An optional output image file to render the results to.");
        ap.parse_args_or_exit();
    }

    let input_path = Path::new(&input_file_name);
    if !input_path.exists() {
        eprintln!("Err: Could not find file - {:?}", input_path);
        exit(-1);
    }

    let image = load_image(&input_path);
    let (width,height) = image.dimensions();
    let rects = find_non_transparent_regions(&image);
    for rect in rects.iter() {
        println!("{}", rect);
    }

    if let Some(output_file_name) = output_file_name {
        write_rectangles_to_file(&output_file_name, (width, height), &rects);
    }
}



fn load_image<P : AsRef<Path>>(path: &P) -> ImageMask {
    let image = image::open(path);
    match image {
        Err(e) => {
            eprintln!("Err: Could not parse image format - {:?}", e);
            exit(-1);
        }
        Ok(image) => {
            let image = image.to_luma_alpha();
            ImageMask::new(&image)
        }
    }
}

fn write_rectangles_to_file<P:AsRef<Path>>(path: &P, (width, height): (u32,u32), rects: &Vec<Rect>) {
    let mut image = image::DynamicImage::new_rgba8(width,height);
    {
        let image = image.as_mut_rgba8().unwrap();
        for (x,y,pixel) in image.enumerate_pixels_mut() {
            for rect in rects.iter() {
                if rect.contains((x,y)) {
                    // pixel[0] = 0;
                    pixel[3] = 205;
                    // println!("Writing pixel");
                }
            }
        }
    }

    match image.save(path) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Err: Could not save output file - {:?}", e);
            exit(-1);
        }
    }
}


/// Uses marching squares algorithm to segment a binary image into multiple distinct rectangles
fn find_non_transparent_regions(image: &ImageMask) -> Vec<Rect> {
    let mut x : u32 = 1;
    let mut y : u32 = 1;
    let (width,height) = {
        let dimensions = image.dimensions();
        (dimensions.0 as u32, dimensions.1 as u32)
    };

    let mut regions : Vec<Rect> = Vec::new();

    while y < height {
        while x < width {
            let pixel = image[(x,y)];
            if pixel {
                // non transparent pixel
                regions.push(marching_squares((x,y), (width,height), image));
            }
            x += 1;

            // skip over seen regions
            for region in regions.iter() {
                if region.contains((x,y)) {
                    let (n_x,n_y) = region.skip((x,y));
                    x = n_x;
                    y = n_y;
                }
            }
        }
        x = 1;
        y += 1;
        // skip over seen regions
        for region in regions.iter() {
            if region.contains((x,y)) {
                let (n_x,n_y) = region.skip((x,y));
                x = n_x;
                y = n_y;
            }
        }

    }

    regions

}

fn marching_squares((mut x,mut y): (u32, u32), (width,height): (u32,u32), image: &ImageMask) -> Rect {
    const LOOKUP_DX : [i32;16] = [
         1, 0, 1, 1,
        -1, 0,-1, 1,
         0, 0, 0, 0,
        -1, 0,-1, 2 // (2 used as a sentinel value)
    ];

    const LOOKUP_DY : [i32;16] = [
        0,-1, 0, 0,
        0,-1, 0, 0,
        1,-1, 1, 1,
        0,-1, 0, 2 // (2 used as a sentinel value)
    ];

    let start_x = x;
    let start_y = y;

    let mut dx : i32 = 0;
    let mut dy : i32 = 0;

    let mut pdx : Option<i32> = None;
    let mut pdy : Option<i32> = None;
    let mut r_x = x;
    let mut r_y = y;

    let mut r_w = 0;
    let mut r_h = 0;

    loop {
        // calculate the index
        let mut i = 0;

        i |= image[(x,y)] as u32;

        i <<= 1;
        if x > 0 {
            i |= image[(x-1,y)] as u32;
        }

        i <<= 1;
        if y > 0 {
            i |= image[(x,y-1)] as u32;
        }


        i <<= 1;
        if x > 0 && y > 0 {
            i |= image[(x-1,y-1)] as u32;
        }

        // special case for horizontal down line
        if i == 6 {
            dx = if let Some(ind) = pdy { if ind == -1 { -1 } else { 1 }} else { 1 };
            dy = 0;
        }
        // special case for vertical line
        else if i == 9 {
            dx = 0;
            // if we are contining a vertical line, we need to make sure that the direction is maintained
            dy = if let Some(ind) = pdx { if ind == 1 { -1 } else { 1 }} else { 1 };
        } else {
            let n_dx = LOOKUP_DX[i as usize];
            let n_dy = LOOKUP_DY[i as usize];
            if n_dx == 2 || n_dy == 2 {
                return Rect(0,0, width,height);
            }

            dx = n_dx;
            dy = n_dy;
        }


        if (pdx.is_none() || pdx.unwrap() != dx) && (pdy.is_none() || pdy.unwrap() != dy) {
            let (n_x, n_y) = (x as u32 ,y as u32);
            // modify rectangle coords to match

            if n_x <= r_x {
                let corner_x = r_x + r_w;
                r_x = n_x;
                r_w = corner_x - r_x;
            }
            if n_x >= r_x + r_w {
                r_w = n_x - r_x;
            }

            if n_y <= r_y {
                let corner_y = r_y + r_h;
                r_y = n_y;
                r_h = corner_y - r_y;
            }
            if n_y >= r_y + r_h {
                r_h = n_y - r_y;
            }



            // update previous dx,dy
            pdx = Some(dx);
            pdy = Some(dy);
        }

        x = (x as i32 + dx) as u32;
        y = (y as i32 + dy) as u32;

        if x >= width || y >= height { break; }

        if (x as i32 - start_x as i32).abs() < 1 && (y as i32 - start_y as i32).abs() < 1 { break; }
    }
    Rect(r_x,r_y, r_w, r_h)
}
