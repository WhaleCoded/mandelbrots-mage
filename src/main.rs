use image;
use std::path;
use std::sync::Arc;
use tokio;

const MAX_ITERATIONS: u16 = 255;

fn calculate_iterations(x: f32, y: f32) -> u16 {
    let mut z = [0.0, 0.0];
    let mut z_2 = [0.0, 0.0];
    let mut iterations = 0;

    let mut z_old = z;
    let mut period = 0;
    while z_2[0] + z_2[1] < 4.0 && iterations < MAX_ITERATIONS {
        z[1] = (z[0] + z[0]) * z[1] + y;
        z[0] = z_2[0] - z_2[1] + x;
        z_2[0] = z[0] * z[0];
        z_2[1] = z[1] * z[1];
        iterations += 1;

        if z == z_old {
            iterations = MAX_ITERATIONS;
            break;
        }
        period += 1;
        if period == 20 {
            z_old = z;
            period = 0;
        }
    }

    iterations
}

fn is_in_cardiod_or_bulb(x: f32, y: f32) -> bool {
    let x = x - 0.25;
    let y = y * y;
    let q = x * x + y;
    let right = q * (q + x);
    let left = 0.25 * y;
    right < left
}

fn get_coordinates_from_pixel_number(
    pixel_num: u32,
    resolution: (u32, u32),
    image_location: (f32, f32, f32, f32),
) -> (f32, f32) {
    let x = pixel_num % resolution.0;
    let y = pixel_num / resolution.0;

    let x = image_location.0
        + (x as f32 / resolution.0 as f32) * (image_location.1 - image_location.0) as f32;
    let y = image_location.2
        + (y as f32 / resolution.1 as f32) * (image_location.3 - image_location.2) as f32;

    (x, y)
}

fn mandelbrot_worker(
    plot: Arc<Vec<std::sync::atomic::AtomicU16>>,
    worker_id: u32,
    pixels_per_worker: u32,
    resolution: (u32, u32),
    image_location: (f32, f32, f32, f32),
    num_workers: u32,
) {
    for local_pixel_num in 0..pixels_per_worker {
        let pixel_num = (local_pixel_num * num_workers) + worker_id;
        if pixel_num >= resolution.0 * resolution.1 {
            break;
        }

        let (x_coord, y_coord) =
            get_coordinates_from_pixel_number(pixel_num, resolution, image_location);

        let iterations;
        if is_in_cardiod_or_bulb(x_coord, y_coord) {
            iterations = MAX_ITERATIONS;
        } else {
            iterations = calculate_iterations(x_coord, y_coord);
        }
        plot[pixel_num as usize].store(iterations, std::sync::atomic::Ordering::Relaxed);
    }
}

fn create_atomic_plot_buffer(resolution: (u32, u32)) -> Vec<std::sync::atomic::AtomicU16> {
    let plot: Vec<std::sync::atomic::AtomicU16> = (0..resolution.0 * resolution.1)
        .map(|_| std::sync::atomic::AtomicU16::new(0))
        .collect();
    plot
}

fn convert_atomic_plot_buffer_to_image_buffer(
    pixels: Vec<std::sync::atomic::AtomicU16>,
    resolution: (u32, u32),
) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let mut image_buffer = image::ImageBuffer::new(resolution.0, resolution.1);
    for (pixel_num, color) in pixels.iter().enumerate() {
        let x = pixel_num % resolution.0 as usize;
        let y = pixel_num / resolution.0 as usize;
        let pixel = image_buffer.get_pixel_mut(x as u32, y as u32);
        *pixel = image::Rgb([
            25,
            30,
            ((color.load(std::sync::atomic::Ordering::Relaxed) as f32 / MAX_ITERATIONS as f32)
                * 255.0) as u8,
        ]);
    }

    image_buffer
}

fn generate_mandelbrot_image(
    pixels: Vec<std::sync::atomic::AtomicU16>,
    resolution: (u32, u32),
    bounds: (f32, f32, f32, f32),
    num_workers: u32,
) -> Vec<std::sync::atomic::AtomicU16> {
    let pixels = Arc::new(pixels);
    let pixels_per_worker = resolution.0 * resolution.1 / num_workers;
    let mut worker_handles = Vec::new();

    for worker_id in 0..num_workers {
        let pixels = Arc::clone(&pixels);
        let handle = std::thread::spawn(move || {
            mandelbrot_worker(
                pixels,
                worker_id,
                pixels_per_worker,
                resolution,
                bounds,
                num_workers,
            )
        });
        worker_handles.push(handle);
    }

    for handle in worker_handles {
        handle.join().unwrap();
    }

    Arc::try_unwrap(pixels).unwrap()
}

#[tokio::main]
async fn main() {
    let num_tests = 75;
    let num_workers = 24;
    let resolution = (1920, 1920);
    let image_location = (-2.0, 1.0, -1.5, 1.5);

    let start_time = std::time::Instant::now();

    let mut plot = create_atomic_plot_buffer(resolution);
    for _ in 0..num_tests {
        plot = generate_mandelbrot_image(plot, resolution, image_location, num_workers);
    }
    let image_buffer = convert_atomic_plot_buffer_to_image_buffer(plot, resolution);

    let end_time = std::time::Instant::now();

    println!(
        "Total time: {}ms",
        end_time.duration_since(start_time).as_millis()
    );
    println!(
        "Time per solve: {}ms",
        end_time.duration_since(start_time).as_millis() as f32 / num_tests as f32
    );

    // save image
    let save_result = image::save_buffer(
        path::Path::new("./mandelbrot_img.png"),
        &image_buffer,
        resolution.0,
        resolution.1,
        image::ColorType::Rgb8,
    );

    match save_result {
        Ok(_) => println!("Image saved successfully!"),
        Err(e) => println!("Error saving image: {}", e),
    }
}
