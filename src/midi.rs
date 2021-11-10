use midly::num::u7;
use midly::{Format, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind};

const MIN_NOTE: u8 = 48;
const MAX_NOTE: u8 = 83;
const TRANSPOSE_OCTAVE: u8 = 12;
const DEFAULT_BEAT_DURATION_US: u32 = 500_000;

pub enum KeyPressEvent {
    PressKey(char),
    WaitUs(u64),
}

struct Event {
    time_us: u64,
    key: char,
}

type Events = Vec<Event>;

pub fn read_events(path_to_midi: &str) -> Result<Vec<KeyPressEvent>, ()> {
    let bytes = std::fs::read(path_to_midi).map_err(|_| ())?;
    let smf = Smf::parse(&bytes).map_err(|_| ())?;
    let tpb = get_ticks_per_beat(&smf);
    let tracks: Vec<Events> = smf
        .tracks
        .iter()
        .map(|track| extract_events(track, tpb, get_beat_duration(&smf)))
        .collect();

    Ok(match smf.header.format {
        Format::SingleTrack | Format::Parallel => {
            let mut merged_events = tracks.into_iter().flatten().collect::<Events>();
            merged_events.sort_by_key(|el| el.time_us);
            events_to_keypresses(merged_events)
        }
        Format::Sequential => tracks
            .into_iter()
            .map(events_to_keypresses)
            .flatten()
            .collect(),
    })
}

fn get_beat_duration(smf: &Smf) -> u32 {
    smf.tracks
        .iter()
        .flatten()
        .find_map(|el| {
            if let TrackEventKind::Meta(MetaMessage::Tempo(beat_duration_us)) = el.kind {
                Some(beat_duration_us.as_int())
            } else {
                None
            }
        })
        .unwrap_or(DEFAULT_BEAT_DURATION_US)
}

fn get_ticks_per_beat(smf: &Smf) -> u32 {
    match smf.header.timing {
        Timing::Metrical(tpb) => tpb.as_int() as u32,
        // TODO
        Timing::Timecode(_, _) => unimplemented!(),
    }
}

fn extract_events(track: &[TrackEvent], ticks_per_beat: u32, beat_duration_us: u32) -> Vec<Event> {
    let mut events = vec![];
    let mut tick_duration = beat_duration_us / ticks_per_beat;
    let mut time = 0;

    for event in track {
        if let TrackEventKind::Meta(MetaMessage::Tempo(beat_duration_us)) = event.kind {
            tick_duration = beat_duration_us.as_int() / ticks_per_beat;
        }

        if let TrackEventKind::Midi {
            channel: _,
            message: MidiMessage::NoteOn { .. } | MidiMessage::NoteOff { .. },
        } = event.kind
        {
            time += (tick_duration * event.delta.as_int()) as u64;
        }

        if let TrackEventKind::Midi {
            channel: _,
            message: MidiMessage::NoteOn { key, vel },
        } = event.kind
        {
            if vel > 0 {
                events.push(Event {
                    time_us: time,
                    key: map_note_to_char(key),
                });
            }
        }
    }

    events
}

fn events_to_keypresses(events: Events) -> Vec<KeyPressEvent> {
    let mut result = Vec::with_capacity(events.len() * 2);
    let mut previous_time = 0;
    for event in events {
        let wait_time = event.time_us - previous_time;
        if wait_time > 0 {
            result.push(KeyPressEvent::WaitUs(wait_time));
        }
        result.push(KeyPressEvent::PressKey(event.key));
        previous_time = event.time_us;
    }
    result
}

fn map_note_to_char(code: u7) -> char {
    let transposed_note = transpose_note(code.as_int());
    if let Some(note) = try_map_note_to_char(transposed_note) {
        note
    } else {
        try_map_note_to_char(transposed_note + 1).unwrap()
    }
}

fn transpose_note(mut note: u8) -> u8 {
    while note < MIN_NOTE {
        note += TRANSPOSE_OCTAVE;
    }
    while note > MAX_NOTE {
        note -= TRANSPOSE_OCTAVE;
    }

    note
}

fn try_map_note_to_char(note: u8) -> Option<char> {
    Some(match note {
        48 => 'z',
        50 => 'x',
        52 => 'c',
        53 => 'v',
        55 => 'b',
        57 => 'n',
        59 => 'm',
        60 => 'a',
        62 => 's',
        64 => 'd',
        65 => 'f',
        67 => 'g',
        69 => 'h',
        71 => 'j',
        72 => 'q',
        74 => 'w',
        76 => 'e',
        77 => 'r',
        79 => 't',
        81 => 'y',
        83 => 'u',
        _ => return None,
    })
}
