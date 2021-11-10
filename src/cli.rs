use clap::{App, Arg};

const MIDI_FILE_ARG_NAME: &str = "PATH";

pub struct Args {
    pub path_to_midi: String,
}

pub fn read_args() -> Args {
    let matches = App::new("GenshinMidiPlayer")
        .version("0.1.0")
        .about("Play midi files on Windsong Lyre and Floral Zither")
        .arg(
            Arg::with_name(MIDI_FILE_ARG_NAME)
                .help("A midi file to play")
                .takes_value(true)
                .required(true),
        )
        .get_matches();
    let path = matches.value_of(MIDI_FILE_ARG_NAME).unwrap();

    Args {
        path_to_midi: path.to_string(),
    }
}
