// The ascii images come from this mod
mod image_vector;
use image_vector::get_image_vector;

// Small ascii images come from this other file
mod small_image_vector;
use small_image_vector::get_small_image_vector;

// The mga small ascii images
mod mega_small_image_vector;
use mega_small_image_vector::get_mega_small_image_vector;

mod idle_vector;
use idle_vector::get_idle_vector;

mod talking_vector;
use talking_vector::get_talking_vector;

// The library to generate random number
use rand::Rng;

// crossterm is a library to handle terminal things
use crossterm::{
    event::{self, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};

// ratatui is the main library, and it's for TUIS (Terminal User InterfaceS)
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::{CrosstermBackend, Terminal},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

// Whith this 2 packages, we handle the standar outputs and Results from the terminal
use std::io::{stdout, Result};
use std::env;

// Add these new imports for file watching
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;
use std::time::Duration;

// Define image size options
enum ImageSize {
    Normal,
    Small,
    MegaSmall,
    Idle,
    Talking,
}

fn main() -> Result<()> {
    // Parse command line arguments for image size
    let args: Vec<String> = env::args().collect();
    let image_size = if args.len() > 1 {
        match args[1].to_lowercase().as_str() {
            "small" => ImageSize::Small,
            "megasmall" | "mega-small" | "mega_small" => ImageSize::MegaSmall,
            "idle" => ImageSize::Idle,
            "talking" => ImageSize::Talking,
            _ => ImageSize::Normal,
        }
    } else {
        ImageSize::Normal
    };

    // First settings to start the terminal and output things
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    // Start using the terminal and clear all the current info apearing
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Using our image_vector file, we set the 1 picture to be the 1st element of the pictures
    // vector
    let mut current_picture = get_image_vector()[0];

    // Initialize the small vector image
    let mut small_current_picture = get_small_image_vector()[0];

    // Initialize the mega small image vector image
    let mut mega_small_current_picture = get_mega_small_image_vector()[0];

    // Initialize the idle vector image
    let mut idle_current_picture = get_idle_vector()[0];

    // Initialize the talking vector image
    let mut talking_current_picture = get_talking_vector()[0];

    // We initialize our random number generator and random_number
    let mut rng = rand::thread_rng();
    let mut random_number;

    // Define available colors and track current color
    let colors = vec![
        Color::Reset,
        Color::Red,
        Color::Green,
        Color::Yellow,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
        Color::White,
        Color::LightBlue,
        Color::LightGreen,
        Color::LightCyan,
        Color::LightRed,
        Color::LightMagenta,
        Color::LightYellow,
    ];
    let mut color_index = 0;

    // Add flags for play state and current vector type
    let mut is_playing = false;
    let mut last_play_time = std::time::Instant::now();
    let play_interval = Duration::from_millis(80); // Cycle every second
    
    // Track current vector mode (idle or talking)
    let mut current_mode = if matches!(image_size, ImageSize::Talking) {
        "talking"
    } else if matches!(image_size, ImageSize::Idle) {
        "idle"
    } else {
        "normal" // Default mode
    };

    // Setup communication channel
    let (tx, rx) = mpsc::channel();
    
    // Define path for the communication file
    let comms_path = Path::new("/tmp/raifus_control");
    
    // Create a file watcher in a separate thread
    thread::spawn(move || {
        watch_for_commands(comms_path, tx);
    });

    // Before the main loop, calculate the minimum vector length
    let min_vector_length = get_image_vector().len()
        .min(get_small_image_vector().len())
        .min(get_mega_small_image_vector().len())
        .min(get_idle_vector().len())
        .min(get_talking_vector().len());

    // TODO main loop (All the terminal things happen here)
    loop {
        // Check for commands from other processes
        match rx.try_recv() {
            Ok(command) => {
                match command.as_str() {
                    "next" => {
                        // Generate a random number within the safe range
                        random_number = rng.gen_range(0..min_vector_length);
                        update_all_images(&mut current_picture, &mut small_current_picture, 
                                         &mut mega_small_current_picture, &mut idle_current_picture,
                                         &mut talking_current_picture, random_number);
                    },
                    "color" => {
                        // Same as pressing 'p'
                        color_index = (color_index + 1) % colors.len();
                    },
                    "play" => {
                        // Start automatically cycling through images
                        is_playing = true;
                        last_play_time = std::time::Instant::now();
                    },
                    "stop" => {
                        // Stop automatically cycling through images
                        is_playing = false;
                    },
                    "switch_idle" => {
                        // Switch to idle vector
                        current_mode = "idle";
                    },
                    "switch_talking" => {
                        // Switch to talking vector
                        current_mode = "talking";
                    },
                    "switch_normal" => {
                        // Switch to normal vector
                        current_mode = "normal";
                    },
                    "quit" => break,
                    _ => {} // Unknown command
                }
            },
            Err(TryRecvError::Empty) => {}, // No command received
            Err(TryRecvError::Disconnected) => {
                // The sender has disconnected - this shouldn't normally happen
                eprintln!("Command channel disconnected");
            }
        }

        // Auto-play logic - cycle to next image if playing and interval has elapsed
        // would be better if play interval was randomized between 80 and 120 ms every time
        if is_playing && last_play_time.elapsed() >= play_interval {
            // Generate a random number within the safe range
            random_number = rng.gen_range(0..min_vector_length);
            update_all_images(&mut current_picture, &mut small_current_picture, 
                             &mut mega_small_current_picture, &mut idle_current_picture,
                             &mut talking_current_picture, random_number);
            last_play_time = std::time::Instant::now();
        }

        // Draw the main UI, and we a closure, where the main "variable" is frame
        terminal.draw(|frame| {
            // Get current color
            let current_color = colors[color_index];
            // Convert color to style
            let current_style = Style::default().fg(current_color);
            
            // Create paragraphs for each image size
            let picture = Paragraph::new(current_picture).style(current_style);
            let small_picture = Paragraph::new(small_current_picture).style(current_style);
            let mega_small_picture = Paragraph::new(mega_small_current_picture).style(current_style);
            let idle_picture = Paragraph::new(idle_current_picture).style(current_style);
            let talking_picture = Paragraph::new(talking_current_picture).style(current_style);

            // Create widgets with borders for each image size
            let banner_widget = picture
                .block(Block::default().borders(Borders::ALL).border_style(current_style))
                .alignment(Alignment::Center);

            let small_banner_widget = small_picture
                .block(Block::default().borders(Borders::ALL).border_style(current_style))
                .alignment(Alignment::Center);

            let mega_small_banner_widget = mega_small_picture
                .block(Block::default().borders(Borders::ALL).border_style(current_style))
                .alignment(Alignment::Center);

            let idle_banner_widget = idle_picture
                .block(Block::default().borders(Borders::ALL).border_style(current_style))
                .alignment(Alignment::Center);

            let talking_banner_widget = talking_picture
                .block(Block::default().borders(Borders::ALL).border_style(current_style))
                .alignment(Alignment::Center);

            // Select the appropriate widget based on the current mode first,
            // falling back to image_size if mode is normal
            let widget_to_render = match current_mode {
                "idle" => &idle_banner_widget,
                "talking" => &talking_banner_widget,
                _ => match image_size {
                    ImageSize::MegaSmall => &mega_small_banner_widget,
                    ImageSize::Small => &small_banner_widget,
                    ImageSize::Idle => &idle_banner_widget,
                    ImageSize::Talking => &talking_banner_widget,
                    ImageSize::Normal => &banner_widget,
                }
            };

            // Render the selected widget
            frame.render_widget(
                widget_to_render.clone(),
                centered_rect(frame.size(), 100, 100),
            );
        })?;

        // Handle events
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                match key.code {
                    // In case you press q, the program will finish
                    KeyCode::Char('q') => break,
                    // In case you press c, the wallpaper will change randomly
                    // using our pictures vector, and the random number generator
                    KeyCode::Char('n') => {
                        // Generate a random number within the safe range
                        random_number = rng.gen_range(0..min_vector_length);
                        update_all_images(&mut current_picture, &mut small_current_picture, 
                                         &mut mega_small_current_picture, &mut idle_current_picture,
                                         &mut talking_current_picture, random_number);
                    },
                    // Change to the next color when 'p' is pressed
                    KeyCode::Char('p') => {
                        color_index = (color_index + 1) % colors.len();
                    },
                    // Toggle play/pause with the space key
                    KeyCode::Char(' ') => {
                        is_playing = !is_playing;
                        if is_playing {
                            last_play_time = std::time::Instant::now();
                        }
                    },
                    // Switch to idle vector with 'i'
                    KeyCode::Char('i') => {
                        current_mode = "idle";
                    },
                    // Switch to talking vector with 't'
                    KeyCode::Char('t') => {
                        current_mode = "talking";
                    },
                    // Switch to normal vector with 'm'
                    KeyCode::Char('m') => {
                        current_mode = "normal";
                    },
                    _ => {}
                }
            }
        }
    }
    disable_raw_mode()?;
    Ok(())
}

// Function to center the img's
fn centered_rect(r: Rect, _percent_x: u16, _percent_y: u16) -> Rect {
    // Set both horizontal and vertical constraints to use full space
    // While still keeping the centered positioning logic
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(0),
            Constraint::Percentage(100),
            Constraint::Percentage(0),
        ])
        .split(r);

    // Horizontal layout with full width
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(0),
            Constraint::Percentage(100),
            Constraint::Percentage(0),
        ])
        .split(popup_layout[1])[1]
}

// New function to watch for commands from other processes
fn watch_for_commands(path: &Path, sender: mpsc::Sender<String>) {
    loop {
        // Try to open the control file
        if path.exists() {
            match File::open(path) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    for line in reader.lines() {
                        if let Ok(command) = line {
                            // Send the command to the main thread
                            if let Err(_) = sender.send(command.trim().to_string()) {
                                return; // If we can't send, the receiver is gone
                            }
                        }
                    }
                    
                    // Delete the file after reading to prepare for new commands
                    if let Err(e) = std::fs::remove_file(path) {
                        eprintln!("Failed to remove control file: {}", e);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to open control file: {}", e);
                }
            }
        }
        
        // Sleep to avoid busy waiting
        thread::sleep(Duration::from_millis(100));
    }
}

// Helper function to update all images
fn update_all_images(
    current_picture: &mut &'static str,
    small_current_picture: &mut &'static str,
    mega_small_current_picture: &mut &'static str,
    idle_current_picture: &mut &'static str,
    talking_current_picture: &mut &'static str,
    index: usize,
) {
    // Get lengths of all vectors
    let normal_len = get_image_vector().len();
    let small_len = get_small_image_vector().len();
    let mega_small_len = get_mega_small_image_vector().len();
    let idle_len = get_idle_vector().len();
    let talking_len = get_talking_vector().len();
    
    // Find the smallest length
    let min_length = normal_len
        .min(small_len)
        .min(mega_small_len)
        .min(idle_len)
        .min(talking_len);
    
    // Make sure index is within bounds for all vectors
    let safe_index = index % min_length;
    
    // Update all images with the safe index
    *current_picture = get_image_vector()[safe_index];
    *small_current_picture = get_small_image_vector()[safe_index];
    *mega_small_current_picture = get_mega_small_image_vector()[safe_index];
    *idle_current_picture = get_idle_vector()[safe_index];
    *talking_current_picture = get_talking_vector()[safe_index];
}
