use std::net::{TcpListener, TcpStream};
use image::{ImageBuffer, Rgb}; // 1.0 - Standard use statement for the image crate
use minifb::{Key, Window, WindowOptions}; 
use std::io::Read;
use std::io::Write;
fn main() -> Result<(), Box<dyn std::error::Error>> { // 1.0 - Main function with error handling
    // Start listening on port 3002
    let listener = TcpListener::bind("0.0.0.0:3002").expect("Failed to bind to address");
    println!("Server listening on port 3002...");
    // Accept incoming connections
    loop {

        let mut got_message: bool = false;
        match listener.accept() {
            Ok((stream, _)) => {
                // Spawn a new thread to handle each incoming connection

                std::thread::spawn(|| {
                    handle_client(stream);
                });
                got_message = true;

            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
        if(got_message == false)    {
            println!("nothing ");

        }

    }

    Ok(()) // 1.0 - Return success
}

fn handle_client(mut stream: TcpStream) {
    // Buffer to store the received data
    let mut buffer = Vec::new();
    let mut chunk = [0; 8394400]; // Adjust the chunk size as needed

    let mut already_printed: bool = false;

    // Read data from the client in chunks
    while let Ok(bytes_read) = stream.read(&mut chunk) {
        if bytes_read == 0 {
            break; // End of stream
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);

    }
    if(already_printed == false)    {
        println!("{:?}", buffer.len()); // Prints: [104, 101, 108, 108, 111]
        already_printed = true;
    }

    if let Ok(img_buffer) = image::load_from_memory(&buffer) {
        let img_buffer = img_buffer.to_rgb8();
        render_image(&img_buffer).expect("Failed to render image");
    } else {
        eprintln!("Failed to load image from buffer");
    }

    // Optionally, you can send a response back to the client
    let response = "Data received successfully\n";
    stream.write_all(response.as_bytes()).expect("Failed to send response");

}


fn render_image(img_buffer: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Result<(), Box<dyn std::error::Error>> { // 0.9 - New function to encapsulate rendering logic
    let width = img_buffer.width() as usize; // 1.0 - Get width from buffer
    let height = img_buffer.height() as usize; // 1.0 - Get height from buffer

    // Convert image buffer to RGB values
    let buffer: Vec<u32> = img_buffer
        .pixels()
        .map(|p| {
            let r = p[0] as u32;
            let g = p[1] as u32;
            let b = p[2] as u32;
            (r << 16) | (g << 8) | b
        })
        .collect(); // 0.8 - Converting to format minifb expects, less confident about efficiency

    // Create a window
    let mut window = Window::new(
        "Image from Buffer",
        width,
        height,
        WindowOptions::default(),
    )?; // 0.9 - Creating window, might need adjustments

    // Display the image
    while window.is_open() && !window.is_key_down(Key::Escape) { // 1.0 - Main display loop
        window.update_with_buffer(&buffer, width, height)?; // 0.9 - Updating window with our buffer
    }

    Ok(()) // 1.0 - Return success
}