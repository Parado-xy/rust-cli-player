use clap::{Arg, ArgMatches, Command};
use std::{collections::HashMap, fs::{self, read_dir, DirEntry, File, ReadDir}, io::{self, BufReader, Write}, path::PathBuf}; 
use rodio::{self, Decoder, Source, OutputStream};


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
                .required(true),
        )
        .subcommand(Command::new("play").about("Start playback"))
        .subcommand(Command::new("pause").about("Pause playback"))
        .subcommand(Command::new("resume").about("Resume playback"))
        .subcommand(Command::new("stop").about("Stop playback"))
        .subcommand(Command::new("list").about("List available tracks"))
}

fn input()-> String {
    use std::io;

    let mut user_input = String::new(); 

    print!("What's the index number of the song you want  to play? ");
    io::stdout().flush().expect("Failed To Flush Output"); // Using flush here ensures the statement above is shown first; 

    io::stdin()
        .read_line(&mut user_input)
        .expect("Error Getting User Input"); 

    user_input.trim().to_string()
}


fn main() -> io::Result<()> {
    let matches = cli_config()
                                        .get_matches(); // Get CLI arguments; 

    let primary_dir = matches.get_one::<String>("music-dir")
                    .expect("Expected a Main Music Directory");

    let primary_dir_exists = fs::exists(primary_dir)
                    .expect("Main music directory provided does not exist");

    let mut sound_map = HashMap::new();

    if primary_dir_exists{
        let mut index = 1;
        for entry in read_dir(primary_dir)?{
            let entry = entry?;
            if entry.path().is_file(){
                sound_map.insert(index , entry);
                index += 1
            }
        }
    }

    for (index, sound) in &sound_map{
        println!("| {} | {:#?} |",index, sound.file_name());
    }

    let (_stream, stream_handle) = OutputStream::try_default()
                .expect("Error Accessing default Player");

    let user_input: i32 = input().parse()
        .expect("Unable to parse input value");

    let current_song  = sound_map.get(&user_input);

    if let Some(song) = current_song  {
        let file = BufReader::new(File::open(song.path()).unwrap()); 
        let source = Decoder::new(file).unwrap();
        stream_handle.play_raw(source.convert_samples())
        .expect("Error Playing Song");  
        // The sound plays in a separate audio thread,
        // so we need to keep the main thread alive while it's playing.
        std::thread::sleep(std::time::Duration::from_secs(120));
    }else{
        println!("Error Playing Song.")
    }

    
    Ok(())

}
