use clap::{builder::Str, Arg, ArgMatches, Command, Error};
use rodio::{self, Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::{
    collections::HashMap,
    fs::{self, read_dir, DirEntry, File, ReadDir},
    io::{self, BufReader, ErrorKind, Write},
    os::windows::process,
    path::PathBuf,
    process::exit,
};

// CLI configuration
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
                .required_unless_present("how-to") // This is an argument. 
        )
        .arg(
            Arg::new("how-to")
                .long("how-to")
                .help("Shows operation commands and how to use the application.")
                .action(clap::ArgAction::SetTrue), // This is a Flag.
        )
}

fn input() -> String {
    use std::io;

    let mut user_input = String::new();

    print!(">> ");
    io::stdout().flush().expect("Failed To Flush Output"); // Using flush here ensures the statement above is shown first;

    io::stdin()
        .read_line(&mut user_input)
        .expect("Error Getting User Input");

    user_input.trim().to_string()
}

struct CliPlayer {
    sink: rodio::Sink,
    stream: rodio::OutputStream, // Store the stream to keep it alive
    stream_handle: OutputStreamHandle,
    is_playing: bool,
    is_paused: bool,
    main_dir: Option<String>,
    current_file: Option<String>,
    last_input: Option<String>,
    available_songs: Option<HashMap<i32, DirEntry>>,
}

enum InputCommands {
    Play,           // Plays a track.
    Pause,          // Pauses a track if any is being played.
    Resume,         // Resumes a track if any was paused.
    Exit,           // Exits the CLI application
    Stop,           // Stops any playback if one is currently taking place.
    List,           // Lists all tracks.
    InvalidCommand, // Catches Invalid command
}

impl CliPlayer {
    // --
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
        })
    }

    pub fn run(&mut self, arguments: ArgMatches) -> io::Result<()> {
        let primary_dir = arguments
            .get_one::<String>("music-dir")
            .ok_or_else(|| io::Error::new(ErrorKind::InvalidInput, "Missing music directory"))?;

        if !fs::metadata(primary_dir)?.is_dir() {
            return Err(io::Error::new(ErrorKind::NotFound, "Directory not found"));
        }

        self.main_dir = Some(primary_dir.to_string());
        self.load_songs()?;
        self.list();

        // Main program loop
        loop {
            self.get_commands();
        }
    }

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

    pub fn play(&mut self, sound_index: i32) -> Result<(), Box<dyn std::error::Error>> {
        // Stop any currently playing music
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
                self.current_file = Some(song.file_name().to_string_lossy().to_string());
                Ok(())
            } else {
                Err("Invalid song index".into())
            }
        } else {
            Err("No songs available".into())
        }
    }

    pub fn act_on_commands(&mut self, command: InputCommands) {
        match command {
            InputCommands::Play => {
                if let Some(index) = &self.last_input {
                    match index.parse::<i32>() {
                        Ok(sound_index) => {
                            if let Err(e) = self.play(sound_index) {
                                println!("Error playing song: {}", e);
                            }
                        }
                        Err(_) => println!("Invalid song index"),
                    }
                } else {
                    println!("Please provide a song index");
                }
            }
            InputCommands::Pause => {
                if self.is_playing {
                    self.sink.pause();
                    self.is_paused = true;
                }
            }

            InputCommands::Resume => {
                if self.is_paused {
                    self.sink.play();
                    self.is_paused = false;
                    self.is_playing = true;
                }
            }

            InputCommands::Stop => {
                if self.is_playing {
                    self.sink.stop();
                    self.is_playing = false;
                }
            }

            InputCommands::List => {
                if let Some(sound_map) = &self.available_songs {
                    for (index, sound) in sound_map {
                        println!("| {} | {:#?} |", index, sound.file_name());
                    }
                }
            }

            InputCommands::Exit => exit(0),

            InputCommands::InvalidCommand => println!("Invalid Command, run --how-to for more info.."),
        }
    }

    pub fn get_commands(&mut self) {
        let user_input = input();
        let user_input: Vec<&str> = user_input.split(" ").collect();

        // Save The Last User Input (Note that an input is different from a command, as users input "commands, inputs");
        if user_input.len() > 1 {
            self.last_input = Some(user_input[1].to_lowercase().to_string());
        }

        match user_input[0] {
            "play" => self.act_on_commands(InputCommands::Play),
            "pause" => self.act_on_commands(InputCommands::Pause),
            "list" => self.act_on_commands(InputCommands::List),
            "resume" => self.act_on_commands(InputCommands::Resume),
            "stop" => self.act_on_commands(InputCommands::Stop),
            "exit" => self.act_on_commands(InputCommands::Exit),
            _ => self.act_on_commands(InputCommands::InvalidCommand),
        }
    }

    pub fn list(&self) {
        if let Some(sound_map) = &self.available_songs {
            for (index, sound) in sound_map {
                println!("| {} | {:#?} |", index, sound.file_name());

            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

fn print_usage_instructions() {
    println!("Music Player Usage Instructions:");
    println!("--------------------------------");
    println!("Commands:");
    println!("  play <number>   - Play the track with the given number");
    println!("  pause           - Pause the current track");
    println!("  resume          - Resume the paused track");
    println!("  stop            - Stop the current playback");
    println!("  list            - Show available tracks");
    println!("  exit            - Exit the program");
    println!("\nExample:");
    println!("  musicplayer --dir /path/to/music/directory");
}
