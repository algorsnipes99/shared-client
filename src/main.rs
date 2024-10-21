use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::io::{Read, Write};
use image::{ImageBuffer, Rgb};
use minifb::{Key, Window, WindowOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let width = 1920 as usize;
    let height = 1080 as usize;

    // Create a window in the main thread
    let mut window = Window::new(
        "Image from Buffer",
        width,
        height,
        WindowOptions::default(),
    )?;

    // Create a channel for sending data from worker threads to the main thread
    let (tx, rx) = mpsc::channel();

    // Start listening on port 3002
    let listener = TcpListener::bind("0.0.0.0:3002").expect("Failed to bind to address");
    println!("Server listening on port 3002...");

    // Spawn a thread to handle incoming connections
    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let tx = tx.clone();
                    thread::spawn(move || {
                        handle_client(tx, stream);
                    });
                }
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                }
            }
        }
    });

    // Main thread: render loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Create a buffer to "clear" the window (e.g., filling with black)
        let mut clear_buffer: Vec<u32> = vec![0; width * height]; // Fill with black (RGB = 0)

        // Receive the image buffer from worker threads
        if let Ok(buffer) = rx.try_recv() {
             // Update window with the clear buffer first (clear the window)
            window.update_with_buffer(&clear_buffer, width, height)?;
            window.update_with_buffer(&buffer, width, height)?;
        }

    }

    Ok(())
}

fn handle_client(tx: mpsc::Sender<Vec<u32>>, mut stream: TcpStream) {
    let mut buffer:Vec<u8>  = Vec::new();
    let mut chunk = vec![0u8; 400000];

    // Read data from the client in chunks
    while let Ok(bytes_read) = stream.read(&mut chunk) {
        println!("bytes_read: {} ", bytes_read);

        if bytes_read == 0 {
            break; // End of stream
        }
        buffer.extend_from_slice(&chunk[..bytes_read]);
    }

    println!("Got full buffer: {} bytes", buffer.len());
    let restored_buff = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(1920, 1080, buffer.clone())
    .expect("Failed to create ImageBuffer from raw data");

    // Convert image buffer to RGB values
    let buffer: Vec<u32> = restored_buff
    .pixels()
    .map(|p| {
        let r = p[0] as u32;
        let g = p[1] as u32;
        let b = p[2] as u32;
        (r << 16) | (g << 8) | b
    })
    .collect(); // 0.8 -

    // Send the rendered buffer back to the main thread
    if tx.send(buffer).is_err() {
        eprintln!("Failed to send buffer to the main thread");
    }

    let response = "Data received successfully\n";
    if let Err(e) = stream.write_all(response.as_bytes()) {
        eprintln!("Failed to send response: {}", e);
    }
}
