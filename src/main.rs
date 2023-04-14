use image;
use std::path;
use std::vec;
use tokio;

// use clap::{Parser, Subcommand};
const MAX_ITERATIONS: u16 = 65535;
const COL_1: [u8; 3] = [185, 237, 221];
const COL_2: [u8; 3] = [135, 203, 185];
const COL_3: [u8; 3] = [85, 157, 170];
const COL_4: [u8; 3] = [87, 125, 134];

struct PixelInfo {
    x: u32,
    y: u32,
    color: [u8; 3],
}

#[tokio::main]
async fn main() {
    //retrieve command line flags and values
    // let commandline_arg_matches = clap::clap_app!(mandelbrots_mage =>
    //     (about: "A command line tool for creating images and GIFs of the Mandelbrot set.")
    //     (version: "1.0")
    //     (@arg RESOlUTION: -r --resolution + takes_value + takes_value "The dimensions of the resulting image. (Width by Height). Default is 1920x1080")
    //     (@arg CENTER: -c --center + takes_value + takes_value "The point of the Mandelbrot set to center the image on. Default is 0+0i")
    //     (@arg MANDELBROT_SCALE: -s --scale + takes_value + takes_value "The range of real numbers from the Mandelbrot set to display. Default is -2,2. Note that the specified center must be inside this range.")
    //     (@arg SAVE_PATH: --path + takes_value "The location to save the image or GIF. Default is the current directory.")
    //     (@arg SHUFFLE_COLORS: --shuffle "Changes what colors represents which degree of instability.")
    //     (@arg GIF: -g --gif + takes_value + takes_value + takes_value "Creates a GIF zooming in on the Mandelbrot set. The first argument is the number of frames, and the last two arguments are the ending scale. Default is 100 frames, and -.01, .01")
    // )
    // .get_matches();

    let x_res: u32 = 1920;
    let y_res: u32 = 1920;
    let image_location = [-2.0, 2.0, -2.0, 2.0];

    let mut resulting_image: Vec<Vec<[u8; 3]>> =
        vec![vec![[0, 0, 0]; x_res as usize]; y_res as usize];

    let mut thread_handels = vec![];
    let start_time = std::time::Instant::now();

    for x in 0..x_res {
        for y in 0..y_res {
            let x_coord = image_location[0]
                + (x as f64 / x_res as f64) * (image_location[1] - image_location[0]) as f64;
            let y_coord = image_location[2]
                + (y as f64 / y_res as f64) * (image_location[3] - image_location[2]) as f64;

            let pixel_task = tokio::spawn(async move {
                let mut z = [0.0, 0.0];
                let c = [x_coord, y_coord];
                let mut iterations = 0;

                while z[0] * z[0] + z[1] * z[1] < 4.0 && iterations < 255 {
                    let temp = z[0] * z[0] - z[1] * z[1] + c[0];
                    z[1] = 2.0 * z[0] * z[1] + c[1];
                    z[0] = temp;
                    iterations += 1;
                }

                // return pixel_info;
                let color = match iterations {
                    1..=75 => COL_1,
                    76..=150 => COL_2,
                    151..=200 => COL_3,
                    201..=250 => COL_4,
                    _ => [iterations, iterations, iterations],
                };

                PixelInfo {
                    x: x,
                    y: y,
                    color: color,
                }
            });

            thread_handels.push(pixel_task);
        }
    }

    for thread in thread_handels {
        let pixel_info = thread.await.unwrap();
        resulting_image[pixel_info.y as usize][pixel_info.x as usize] = pixel_info.color;
    }

    // save image
    let mut image_buffer = image::ImageBuffer::new(x_res, y_res);
    for (x, y, pixel) in image_buffer.enumerate_pixels_mut() {
        *pixel = image::Rgb(resulting_image[y as usize][x as usize]);
    }

    // let mut dyn_image = image::DynamicImage::ImageRgb8(image_buffer);
    // dyn_image = dyn_image.resize(x_res, y_res, image::imageops::FilterType::Nearest);
    // let dyn_save_result = dyn_image.save("./mandelbrot_img.png");
    let save_result = image::save_buffer(
        path::Path::new("./mandelbrot_img.png"),
        &image_buffer,
        x_res,
        y_res,
        image::ColorType::Rgb8,
    );

    match save_result {
        Ok(_) => println!("Image saved successfully!"),
        Err(e) => println!("Error saving image: {}", e),
    }

    println!("Time taken: {}ms", start_time.elapsed().as_millis());
}
