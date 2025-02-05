//! A command-line music player implementation in Rust
//! Supports basic playback controls, volume adjustment, and file management
//! Author: ojalla

use clap::{ Arg, ArgMatches, Command};
use colored::*;
use rodio::{self, Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::{
    collections::HashMap,
    fs::{self, read_dir, DirEntry, File},
    io::{self, BufReader, ErrorKind, Write},
    process::exit,
    time::Instant,
};

/// Configures and returns the command-line interface for the music player
/// Sets up required arguments and flags for directory specification and help
fn cli_config() -> Command {
    Command::new("musicplayer")
        .version("0.1.0")
        .author("ojalla")
        .about("Command-line music player")
        .arg(
            Arg::new("music-dir")
                .short('d')
                .long("dir")
                .value_name("DIRECTORY")
                .help("Sets the music directory")
                .required_unless_present("how-to"),
        )
        .arg(
            Arg::new("how-to")
                .long("how-to")
                .help("Shows operation commands and how to use the application.")
                .action(clap::ArgAction::SetTrue),
        )
}

/// Gets user input from the command line with a custom prompt
/// Returns the trimmed input as a String
fn input() -> String {
    use std::io;

    let mut user_input = String::new();

    print!("{}", "musicplayer> ".cyan().bold());
    io::stdout().flush().expect("Failed To Flush Output");

    io::stdin()
        .read_line(&mut user_input)
        .expect("Error Getting User Input");

    user_input.trim().to_string()
}

/// Main struct representing the CLI music player
/// Maintains state and handles all player operations
struct CliPlayer {
    sink: rodio::Sink,                           // Audio sink for playback
    stream: rodio::OutputStream,                 // Audio output stream
    stream_handle: OutputStreamHandle,           // Handle to the audio stream
    is_playing: bool,                           // Current playback status
    is_paused: bool,                            // Current pause status
    main_dir: Option<String>,                   // Directory containing music files
    current_file: Option<String>,               // Currently playing file name
    last_input: Option<String>,                 // Last user input
    available_songs: Option<HashMap<i32, DirEntry>>, // Map of available songs
    start_time: Option<Instant>,                // Start time of current playback
}

/// Enum representing all possible commands the player can handle
enum InputCommands {
    Play,            // Plays a track
    Pause,           // Pauses current track
    Resume,          // Resumes paused track
    Exit,            // Exits application
    Stop,            // Stops playback
    List,            // Lists available tracks
    InvalidCommand,  // Invalid command handler
    Volume(f32),     // Sets volume (0.0-1.0)
    Status,          // Shows player status
    Help,            // Shows help information
}

impl CliPlayer {
    /// Creates a new instance of the CLI player
    /// Sets up audio streams and initializes default state
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        Ok(Self {
            sink,
            stream,
            stream_handle,
            is_playing: false,
            is_paused: false,
            main_dir: None,
            current_file: None,
            last_input: None,
            available_songs: Some(HashMap::new()),
            start_time: None,
        })
    }

    /// Main run loop for the player
    /// Handles initialization and command processing
    pub fn run(&mut self, arguments: ArgMatches) -> io::Result<()> {
        // Validate and set music directory
        let primary_dir = arguments
            .get_one::<String>("music-dir")
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Missing music directory"))?;

        if !fs::metadata(primary_dir)?.is_dir() {
            return Err(io::Error::new(ErrorKind::NotFound, "Directory not found"));
        }

        self.main_dir = Some(primary_dir.to_string());
        self.load_songs()?;
        
        // Display welcome message and initial song list
        println!("\n{}", "Welcome to Music Player!".green().bold());
        println!("Loaded directory: {}", primary_dir.blue());
        println!(
            "Found {} songs.\n",
            self.available_songs
                .as_ref()
                .unwrap()
                .len()
                .to_string()
                .yellow()
        );
        self.list();

        // Main program loop
        loop {
            self.get_commands();
        }
    }

    /// Loads songs from the specified directory into the available_songs HashMap
    fn load_songs(&mut self) -> io::Result<()> {
        let mut index = 1;
        if let Some(dir) = &self.main_dir {
            if let Some(sound_map) = &mut self.available_songs {
                for entry in read_dir(dir)? {
                    let entry = entry?;
                    if entry.path().is_file() {
                        sound_map.insert(index, entry);
                        index += 1;
                    }
                }
            }
        }
        Ok(())
    }

    /// Plays a song by its index number
    /// Handles stopping current playback and starting new playback
    pub fn play(&mut self, sound_index: i32) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_playing {
            self.sink.stop();
            self.sink = Sink::try_new(&self.stream_handle)?;
        }

        if let Some(sound_map) = &self.available_songs {
            if let Some(song) = sound_map.get(&sound_index) {
                let file = BufReader::new(File::open(song.path())?);
                let source = Decoder::new(file)?;
                self.sink.set_volume(1.0);
                self.sink.append(source.convert_samples::<f32>());
                self.is_playing = true;
                self.is_paused = false;
                self.current_file = Some(song.file_name().to_string_lossy().to_string());
                self.start_time = Some(Instant::now());
                println!(
                    "{}: Playing {}",
                    "Now playing".green().bold(),
                    self.current_file.as_ref().unwrap().blue()
                );
                Ok(())
            } else {
                Err(format!("{}: Invalid song index", "Error".red()).into())
            }
        } else {
            Err("No songs available".into())
        }
    }

    /// Processes and executes commands based on the InputCommands enum
    pub fn act_on_commands(&mut self, command: InputCommands) {
        match command {
            InputCommands::Play => {
                if let Some(index) = &self.last_input {
                    match index.parse::<i32>() {
                        Ok(sound_index) => {
                            if let Err(e) = self.play(sound_index) {
                                println!("{}: {}", "Error".red(), e);
                            }
                        }
                        Err(_) => println!("{}: Invalid song index", "Error".red()),
                    }
                } else {
                    println!("{}: Please provide a song index", "Error".red());
                }
            }
            InputCommands::Pause => {
                if self.is_playing {
                    self.sink.pause();
                    self.is_paused = true;
                    println!("{}: Playback paused", "Info".yellow());
                }
            }

            InputCommands::Resume => {
                if self.is_paused {
                    self.sink.play();
                    self.is_paused = false;
                    self.is_playing = true;
                    println!("{}: Playback resumed", "Info".green());
                }
            }

            InputCommands::Stop => {
                if self.is_playing {
                    self.sink.stop();
                    self.is_playing = false;
                    println!("{}: Playback stopped", "Info".red());
                }
            }

            InputCommands::List => {
                self.list();
            }

            InputCommands::Volume(vol) => {
                if (0.0..=1.0).contains(&vol) {
                    self.sink.set_volume(vol);
                    println!("{}: Volume set to {:.1}", "Success".green(), vol);
                } else {
                    println!("{}: Volume must be 0.0 to 1.0", "Error".red());
                }
            }

            InputCommands::Status => {
                println!("\n{}", "Player Status:".bold());
                println!("{}", "--------------".bold());
                if let Some(current) = &self.current_file {
                    println!("  {}: {}", "Song".bold(), current.blue());
                    let state = if self.is_paused {
                        "Paused".yellow()
                    } else if self.is_playing {
                        "Playing".green()
                    } else {
                        "Stopped".red()
                    };
                    println!("  {}: {}", "State".bold(), state);
                    if let Some(start) = &self.start_time {
                        let elapsed = start.elapsed().as_secs();
                        println!(
                            "  {}: {} seconds",
                            "Elapsed".bold(),
                            elapsed.to_string().cyan()
                        );
                    }
                } else {
                    println!("  {}: No song playing", "Song".bold());
                }
                println!("  {}: {:.1}", "Volume".bold(), self.sink.volume());
            }

            InputCommands::Exit => exit(0),

            InputCommands::Help => print_usage_instructions(),

            InputCommands::InvalidCommand => {
                println!(
                    "{}: Invalid command - type 'help' for instructions",
                    "Error".red().bold()
                )
            }
        }
    }

    /// Processes user input and converts it to appropriate commands
    pub fn get_commands(&mut self) {
        let input_line = input();
        let tokens: Vec<&str> = input_line.split_whitespace().collect();

        // If no tokens, do nothing.
        if tokens.is_empty() {
            return;
        }

        // Safely set last_input only if a second token exists.
        self.last_input = tokens.get(1).map(|s| s.to_lowercase());

        // Dispatch based on the first token.
        match tokens[0].to_lowercase().as_str() {
            "play" => self.act_on_commands(InputCommands::Play),
            "pause" => self.act_on_commands(InputCommands::Pause),
            "list" => self.act_on_commands(InputCommands::List),
            "resume" => self.act_on_commands(InputCommands::Resume),
            "stop" => self.act_on_commands(InputCommands::Stop),
            "volume" => {
                if let Some(vol_str) = tokens.get(1) {
                    if let Ok(vol) = vol_str.parse::<f32>() {
                        self.act_on_commands(InputCommands::Volume(vol));
                    } else {
                        println!("{}: Invalid volume value", "Error".red());
                    }
                } else {
                    println!("{}: Missing volume value", "Error".red());
                }
            }
            "status" => self.act_on_commands(InputCommands::Status),
            "help" => self.act_on_commands(InputCommands::Help),
            "exit" => self.act_on_commands(InputCommands::Exit),
            _ => self.act_on_commands(InputCommands::InvalidCommand),
        }
    }

    /// Lists all available songs with their index numbers
    /// Highlights currently playing song if any
    pub fn list(&self) {
        if let Some(sound_map) = &self.available_songs {
            println!("\n{}", "Available Songs:".green().bold());
            println!("{}", "-------------------------------".green());
            println!(
                "{:<6} {:<}",
                "Index".to_string().bold(),
                "Filename".to_string().bold()
            );
            for (index, entry) in sound_map {
                let filename = entry.file_name();
                let filename = filename.to_string_lossy();
                if let Some(current) = &self.current_file {
                    if filename == *current {
                        println!(
                            "{:<6} {:<} {}",
                            index.to_string().green(),
                            filename.green(),
                            "▶".green()
                        );
                    } else {
                        println!("{:<6} {:<}", index, filename);
                    }
                } else {
                    println!("{:<6} {:<}", index, filename);
                }
            }
            println!();
        }
    }
}

/// Main entry point for the application
/// Sets up Ctrl+C handler and initializes the player
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Set up Ctrl+C handler for graceful exit
    ctrlc::set_handler(|| {
        println!("\n{}: Exiting...", "Info".blue());
        exit(0);
    })?;

    let arguments = cli_config().get_matches();

    // Check if --how-to flag is present
    if arguments.get_flag("how-to") {
        print_usage_instructions();
        return Ok(());
    }

    let mut application = CliPlayer::new()?;
    application.run(arguments)?;
    Ok(())
}

/// Prints usage instructions and available commands
fn print_usage_instructions() {
    println!("\n{}", "Music Player Usage Instructions:".bold());
    println!("{}", "--------------------------------".bold());
    println!("{}:", "Commands".bold());
    println!(
        "  {} <number>   - Play the track with the given number",
        "play".green()
    );
    println!("  {}           - Pause the current track", "pause".yellow());
    println!("  {}          - Resume the paused track", "resume".green());
    println!("  {}            - Stop the current playback", "stop".red());
    println!("  {} <0.0-1.0> - Set playback volume", "volume".cyan());
    println!("  {}           - Show player status", "status".blue());
    println!("  {}           - Show available tracks", "list".cyan());
    println!("  {}           - Show this help message", "help".yellow());
    println!("  {}            - Exit the program", "exit".red());
    println!("\n{}:", "Example".bold());
    println!("  musicplayer --dir /path/to/music/directory\n");
}