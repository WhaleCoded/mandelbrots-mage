use image;
use std::path;
use std::vec;
use tokio;

use std::sync::{Arc, Mutex};

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

    let resolution = [1920, 1080];
    let image_location = [-2.0, 2.0, -2.0, 2.0];

    let start_time = std::time::Instant::now();

    let num_workers = 12;
    let pixels_per_worker = resolution[0] * resolution[1] / num_workers;
    let mut worker_handles = Vec::new();
    let results: Arc<Mutex<Vec<PixelInfo>>> = Arc::new(Mutex::new(Vec::new()));

    for worker_id in 0..num_workers {
        // let stack = Arc::clone(&stack);
        let results = Arc::clone(&results);
        let handle = std::thread::spawn(move || {
            let mut local_results = Vec::new();

            for local_pixel_num in 0..pixels_per_worker {
                let pixel_num = (local_pixel_num * num_workers) + worker_id;
                if pixel_num >= resolution[0] * resolution[1] {
                    break;
                }

                let x = pixel_num % resolution[0];
                let y = pixel_num / resolution[0];

                let x_coord = image_location[0]
                    + (x as f64 / resolution[0] as f64)
                        * (image_location[1] - image_location[0]) as f64;
                let y_coord = image_location[2]
                    + (y as f64 / resolution[1] as f64)
                        * (image_location[3] - image_location[2]) as f64;

                let mut z = [0.0, 0.0];
                let c = [x_coord, y_coord];
                let mut iterations = 0;

                while z[0] * z[0] + z[1] * z[1] < 4.0 && iterations < 255 {
                    let temp = z[0] * z[0] - z[1] * z[1] + c[0];
                    z[1] = 2.0 * z[0] * z[1] + c[1];
                    z[0] = temp;
                    iterations += 1;
                }

                let color = match iterations {
                    1..=75 => COL_1,
                    76..=150 => COL_2,
                    151..=225 => COL_3,
                    _ => COL_4,
                };

                let pixel_info = PixelInfo { x, y, color };

                local_results.push(pixel_info);
            }

            results.lock().unwrap().extend(local_results);
        });
        worker_handles.push(handle);
    }

    for handle in worker_handles {
        handle.join().unwrap();
    }

    // save image
    let mut image_buffer = image::ImageBuffer::new(resolution[0], resolution[1]);
    for pixel_info in results.lock().unwrap().iter() {
        image_buffer.put_pixel(pixel_info.x, pixel_info.y, image::Rgb(pixel_info.color));
    }

    println!("Time taken: {}ms", start_time.elapsed().as_millis());

    // let mut dyn_image = image::DynamicImage::ImageRgb8(image_buffer);
    // dyn_image = dyn_image.resize(x_res, y_res, image::imageops::FilterType::Nearest);
    // let dyn_save_result = dyn_image.save("./mandelbrot_img.png");
    let save_result = image::save_buffer(
        path::Path::new("./mandelbrot_img.png"),
        &image_buffer,
        resolution[0],
        resolution[1],
        image::ColorType::Rgb8,
    );

    match save_result {
        Ok(_) => println!("Image saved successfully!"),
        Err(e) => println!("Error saving image: {}", e),
    }
}
