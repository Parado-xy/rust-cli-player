# Rust CLI Music Player

A simple command-line music player built in Rust, allowing basic audio playback and management.

## Features

- Play music from a specified directory
- List available tracks
- Basic playback controls:
  - Play specific tracks
  - Pause
  - Resume
  - Stop
  - Exit

## Prerequisites

- Rust (latest stable version recommended)
- Cargo package manager

## Installation

1. Clone the repository
```bash
git clone https://github.com/yourusername/rust-cli-music-player.git
cd rust-cli-music-player
```

2. Build the project
```bash
cargo build --release
```

3. Run the application
```bash
cargo run -- --dir /path/to/your/music/directory
```

## Usage

### Basic Commands

- `play <number>`: Play a track by its list number
- `pause`: Pause current playback
- `resume`: Resume paused track
- `stop`: Stop current playback
- `list`: Show available tracks
- `exit`: Close the application

### Example

```bash
# Start the player with your music directory
./musicplayer --dir ~/Music

# Show usage instructions
./musicplayer --how-to
```

## Planned Future Improvements

Anyone interested in contributing can focus on these potential enhancements:

### Functionality Improvements
1. **Playlist Support**
   - Create and save playlists
   - Implement playlist navigation
   - Add shuffle and repeat modes

2. **Advanced Playback Controls**
   - Volume adjustment
   - Seeking within tracks
   - Crossfading between songs

3. **Audio Format Support**
   - Expand beyond current format limitations
   - Add support for more audio codecs
   - Implement metadata reading (artist, album, etc.)

### Technical Improvements
1. **Error Handling**
   - More robust error management
   - Detailed error messages
   - Graceful error recovery

2. **Configuration**
   - Add config file support
   - Persistent settings between sessions
   - User-defined default directories

3. **Performance Optimizations**
   - Improve audio loading speed
   - Optimize memory usage
   - Implement efficient file scanning

### User Experience
1. **Terminal UI Enhancements**
   - Colorful output
   - Progress bars for tracks
   - Interactive track selection

2. **Search Functionality**
   - Search tracks by name
   - Filter tracks by metadata

3. **Remote Playback**
   - Network streaming
   - Integration with remote music libraries

### Advanced Features
1. **Equalizer**
   - Basic audio equalization
   - Sound profile management

2. **Keyboard Shortcuts**
   - Global media key support
   - Customizable key bindings

3. **Plugins/Extensions**
   - Create a plugin system
   - Allow community-driven feature additions

## Contributing

Contributions are welcome! Please:
- Fork the repository
- Create a feature branch
- Submit a pull request

## License

MIT

## Author

ojalla

## Acknowledgments

- [rodio](https://github.com/RustAudio/rodio) - Audio playback library
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing