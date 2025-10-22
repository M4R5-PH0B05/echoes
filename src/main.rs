use std::cmp::min;
use std::fs::File;
use std::io::{self, Write};
use std::time::Duration;

use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::io::MediaSourceStream;
use symphonia::default::get_probe;

fn main() {
    decode_file("audio/test.mp3");
}

// Decode an audio file and render frames into the terminal.
fn decode_file(filename: &str) {
    let src = Box::new(File::open(filename).expect("failed to open audio file"));
    let mss = MediaSourceStream::new(src, Default::default());

    let probe = get_probe()
        .format(
            &Default::default(),
            mss,
            &Default::default(),
            &Default::default(),
        )
        .expect("unsupported media format");

    let mut format = probe.format;
    let track = format.default_track().expect("no default track in file");

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &Default::default())
        .expect("failed to build decoder");

    let mut visualizer = Visualizer::new();

    while let Ok(packet) = format.next_packet() {
        let decoded = decoder
            .decode(&packet)
            .expect("decoder error while reading packet");

        match decoded {
            AudioBufferRef::F32(buf) => {
                visualizer.render(buf.chan(0));
            }
            AudioBufferRef::S16(buf) => {
                let samples: Vec<f32> = buf.chan(0).iter().map(|x| *x as f32 / 32_768.0).collect();
                visualizer.render(&samples);
            }
            _ => eprintln!("Unsupported sample format"),
        }

        std::thread::sleep(Duration::from_millis(33));
    }
}

struct Visualizer {
    peak: f32,
    prev_columns: Vec<(f32, f32)>,
}

impl Visualizer {
    fn new() -> Self {
        Self {
            peak: 0.25,
            prev_columns: Vec::new(),
        }
    }

    fn render(&mut self, samples: &[f32]) {
        const NUM_BARS: usize = 64;
        const MAX_HEIGHT: usize = 21;

        if samples.is_empty() {
            return;
        }

        print!("\x1B[2J\x1B[H");

        let chunk_size = (samples.len() + NUM_BARS - 1) / NUM_BARS;
        let mut columns: Vec<(f32, f32)> = Vec::with_capacity(NUM_BARS);
        let mut frame_peak = 0.0f32;

        for i in 0..NUM_BARS {
            let start = i * chunk_size;
            if start >= samples.len() {
                columns.push((0.0, 0.0));
                continue;
            }

            let end = min(start + chunk_size, samples.len());
            let chunk = &samples[start..end];

            if chunk.is_empty() {
                columns.push((0.0, 0.0));
                continue;
            }

            let mut pos_peak = 0.0f32;
            let mut pos_sum = 0.0f32;
            let mut pos_count = 0u32;
            let mut neg_peak = 0.0f32;
            let mut neg_sum = 0.0f32;
            let mut neg_count = 0u32;

            for &sample in chunk {
                if sample > 0.0 {
                    pos_peak = pos_peak.max(sample);
                    pos_sum += sample;
                    pos_count += 1;
                } else if sample < 0.0 {
                    let magnitude = -sample;
                    neg_peak = neg_peak.max(magnitude);
                    neg_sum += magnitude;
                    neg_count += 1;
                }
            }

            let pos_level = if pos_count > 0 {
                let avg = pos_sum / pos_count as f32;
                0.75 * pos_peak + 0.25 * avg
            } else {
                0.0
            };

            let neg_level = if neg_count > 0 {
                let avg = neg_sum / neg_count as f32;
                0.75 * neg_peak + 0.25 * avg
            } else {
                0.0
            };

            frame_peak = frame_peak.max(pos_level.max(neg_level));
            columns.push((pos_level, neg_level));
        }

        if frame_peak > self.peak {
            self.peak = frame_peak;
        } else {
            const DECAY: f32 = 0.92;
            self.peak = self.peak * DECAY + frame_peak * (1.0 - DECAY);
        }

        let peak = self.peak.max(1e-3);

        if self.prev_columns.len() != NUM_BARS {
            self.prev_columns = vec![(0.0, 0.0); NUM_BARS];
        }

        let smoothed: Vec<(f32, f32)> = columns
            .iter()
            .zip(self.prev_columns.iter())
            .map(|(&(pos, neg), &(prev_pos, prev_neg))| {
                let norm_pos = (pos / peak).clamp(0.0, 1.0);
                let norm_neg = (neg / peak).clamp(0.0, 1.0);
                let blend = 0.65;
                let new_pos = blend * norm_pos + (1.0 - blend) * prev_pos;
                let new_neg = blend * norm_neg + (1.0 - blend) * prev_neg;
                (new_pos, new_neg)
            })
            .collect();

        self.prev_columns.copy_from_slice(&smoothed);

        const TOTAL_ROWS: usize = MAX_HEIGHT;
        let mid_row = TOTAL_ROWS / 2;
        let top_rows = mid_row;
        let mut frame = String::with_capacity((TOTAL_ROWS + 1) * (NUM_BARS * 8));

        for row in 0..TOTAL_ROWS {
            for &(pos, neg) in &smoothed {
                let pos_rows = (pos * top_rows as f32).round() as usize;
                let neg_rows = (neg * top_rows as f32).round() as usize;

                if row < mid_row {
                    let threshold = top_rows.saturating_sub(pos_rows);
                    if row >= threshold {
                        frame.push_str(color_for(pos));
                        frame.push('█');
                        frame.push_str("\x1B[0m");
                    } else {
                        frame.push(' ');
                    }
                } else if row == mid_row {
                    frame.push('─');
                } else {
                    let offset = row - mid_row - 1;
                    if offset < neg_rows {
                        frame.push_str(color_for(neg));
                        frame.push('█');
                        frame.push_str("\x1B[0m");
                    } else {
                        frame.push(' ');
                    }
                }
            }
            frame.push('\n');
        }

        print!("{}", frame);
        let _ = io::stdout().flush();
    }
}

fn color_for(level: f32) -> &'static str {
    let scaled = level.powf(0.6);
    if scaled < 0.2 {
        "\x1B[38;5;39m" // teal
    } else if scaled < 0.4 {
        "\x1B[38;5;48m" // green
    } else if scaled < 0.6 {
        "\x1B[38;5;190m" // yellow
    } else if scaled < 0.8 {
        "\x1B[38;5;208m" // orange
    } else {
        "\x1B[38;5;196m" // red
    }
}
