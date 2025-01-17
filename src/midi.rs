use midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Track, TrackEvent, TrackEventKind,
};
use std::time::Duration;
use rodio::{OutputStream, Sink, source::{SineWave, Source}};
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
    
    // Set tempo (slower tempo for better clarity)
    track.push(TrackEvent {
        delta: 0.into(),
        kind: TrackEventKind::Meta(MetaMessage::Tempo((60_000_000 / 100).into())), // 100 BPM
    });

    // Set instruments for each channel using basic GM instruments
    for channel in 0..4 {
        track.push(TrackEvent {
            delta: 0.into(),
            kind: TrackEventKind::Midi {
                channel: channel.into(),
                message: MidiMessage::ProgramChange {
                    program: match channel {
                        0 => 0.into(),   // Piano
                        1 => 25.into(),  // Steel Guitar
                        2 => 40.into(),  // Violin
                        _ => 19.into(),  // Church Organ
                    },
                },
            },
        });
    }

    let mut current_time = 0u32;
    for note in notes {
        // Note on
        track.push(TrackEvent {
            delta: 60.into(), // Small pause between notes
            kind: TrackEventKind::Midi {
                channel: note.channel.into(),
                message: MidiMessage::NoteOn {
                    key: note.note.into(),
                    vel: (100u8).into(), // Good velocity for clear sound
                },
            },
        });

        // Calculate duration based on the commit size but ensure it's audible
        let duration_ticks = ((note.duration.as_secs_f32() * 480.0) as u32)
            .max(240)  // Minimum duration (half note)
            .min(960); // Maximum duration (2 whole notes)

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

        current_time += duration_ticks + 60;
    }

    // End of track
    track.push(TrackEvent {
        delta: 480.into(), // Add a full note pause at the end
        kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
    });

    smf.tracks.push(track);
    smf
}

pub fn play_midi(midi_data: &Smf) -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    // Create a temporary file with .mid extension
    let temp_file = NamedTempFile::new()?.into_temp_path();
    let temp_path = temp_file.with_extension("mid");
    let mut file = File::create(&temp_path)?;
    midi_data.write_std(&mut file)?;

    // Save the file for external playback
    println!("\n🎵 MIDI file saved temporarily. For better playback, use an external MIDI player.");
    println!("File location: {}", temp_path.display());

    // Basic audio preview
    for event in midi_data.tracks[0].iter() {
        if let TrackEventKind::Midi { message, channel } = event.kind {
            if let MidiMessage::NoteOn { key, vel } = message {
                if vel.as_int() > 0 {
                    // Convert MIDI note to frequency
                    let freq = 440.0 * 2.0f32.powf((key.as_int() as f32 - 69.0) / 12.0);
                    
                    // Create a short preview tone
                    let source = SineWave::new(freq)
                        .amplify(0.15)
                        .take_duration(Duration::from_millis(100))
                        .fade_in(Duration::from_millis(10));
                    
                    sink.append(source);
                    std::thread::sleep(Duration::from_millis(150));
                }
            }
        }
    }

    sink.sleep_until_end();
    Ok(())
} 