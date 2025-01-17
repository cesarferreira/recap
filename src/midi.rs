use midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Track, TrackEvent, TrackEventKind,
};
use std::time::Duration;
use std::io::Cursor;
use rodio::{OutputStream, Sink};
use std::fs::File;
use std::io::Write;
use tempfile::NamedTempFile;

const BASE_NOTE: u8 = 60; // Middle C
const VELOCITY: u8 = 100;

pub struct CommitNote {
    pub note: u8,
    pub duration: Duration,
    pub velocity: u8,
    pub channel: u8,
}

pub struct MusicConfig {
    pub base_note: u8,
    pub velocity: u8,
    pub tempo: u32,
}

impl Default for MusicConfig {
    fn default() -> Self {
        Self {
            base_note: BASE_NOTE,
            velocity: VELOCITY,
            tempo: 120,
        }
    }
}

pub fn commit_to_note(
    additions: i32,
    deletions: i32,
    file_type: &str,
    config: &MusicConfig,
) -> CommitNote {
    // Map file types to different instruments (MIDI channels)
    let channel = match file_type {
        "rs" => 0,  // Piano for Rust files
        "js" | "ts" => 1,  // Electric Piano for JS/TS
        "py" => 2,  // Strings for Python
        _ => 3,  // Default instrument
    };

    // Calculate note based on additions/deletions ratio
    let note_offset = if additions > deletions {
        (additions as f32).log2().ceil() as i8
    } else {
        -(deletions as f32).log2().ceil() as i8
    };

    let note = ((config.base_note as i16 + note_offset as i16).clamp(0, 127)) as u8;

    // Map commit size to note duration
    let total_changes = additions + deletions;
    let duration = Duration::from_millis(((total_changes as f32).log2() * 500.0) as u64)
        .max(Duration::from_millis(200))
        .min(Duration::from_millis(2000));

    CommitNote {
        note,
        duration,
        velocity: config.velocity,
        channel,
    }
}

pub fn generate_midi(notes: Vec<CommitNote>, config: &MusicConfig) -> Smf {
    let mut smf = Smf::new(Header::new(
        Format::SingleTrack,
        midly::Timing::Metrical(480.into()),
    ));

    let mut track = Track::new();
    
    // Set tempo
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo((60_000_000 / config.tempo).into())),
    });

    let mut time = 0u32;
    for note in notes {
        // Note on
        track.push(TrackEvent {
            delta: time.into(),
            kind: TrackEventKind::Midi {
                channel: note.channel.into(),
                message: MidiMessage::NoteOn {
                    key: note.note.into(),
                    vel: note.velocity.into(),
                },
            },
        });

        // Calculate duration in MIDI ticks (assuming 480 ticks per quarter note)
        let duration_ticks = (note.duration.as_secs_f32() * 480.0) as u32;

        // Note off
        track.push(TrackEvent {
            delta: duration_ticks.into(),
            kind: TrackEventKind::Midi {
                channel: note.channel.into(),
                message: MidiMessage::NoteOff {
                    key: note.note.into(),
                    vel: 0.into(),
                },
            },
        });

        time = duration_ticks;
    }

    // End of track
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    smf.tracks.push(track);
    smf
}

pub fn play_midi(midi_data: &Smf) -> Result<(), Box<dyn std::error::Error>> {
    // Create a temporary file to store MIDI data
    let mut temp_file = NamedTempFile::new()?;
    midi_data.write_std(&mut temp_file)?;
    
    // Get output stream handle
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;
    
    // Load and play the MIDI file
    let file = std::fs::File::open(temp_file.path())?;
    sink.append(rodio::Decoder::new(file)?);
    
    sink.sleep_until_end();
    Ok(())
} 