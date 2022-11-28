use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::{eyre, Result};

use crate::ir::ButtonKind;

use super::parse;

#[derive(Clone, Debug, Args)]
pub(crate) struct Command {
    /// Path to the .ir file to analyze.
    file: PathBuf,
}

impl Command {
    pub(super) fn run(self) -> Result<()> {
        let mut contents = String::new();
        File::open(self.file)?.read_to_string(&mut contents)?;
        let (_, buttons) =
            parse::ir_file(&contents).map_err(|e| eyre!("Invalid .ir file: {}", e))?;

        if buttons.is_empty() {
            return Err(eyre!("No buttons in .ir file"));
        }

        let raw = || {
            buttons.iter().filter_map(|b| match &b.kind {
                ButtonKind::Raw(raw) => Some(raw),
                _ => None,
            })
        };
        if raw().filter(|b| !b.data.is_empty()).count() == 0 {
            return Err(eyre!("No raw data in any of the buttons in .ir file"));
        }

        let preamble = || raw().flat_map(|b| b.data.first());
        let mark = || raw().flat_map(|b| b.data.iter().skip(1).map(|(on, _)| on));
        let space = || raw().flat_map(|b| b.data.iter().skip(1).map(|(_, off)| off));

        let preamble_mark = avg(preamble().map(|(on, _)| on));
        let preamble_space = avg(preamble().map(|(_, off)| off));

        // Identify a repeat marker by looking for a very long space.
        let repeat_marker = {
            let mut repeat = None;
            let mut metrics = Metrics::new();
            for (i, space) in space().enumerate() {
                let new_metrics = metrics.accumulate(*space);
                if new_metrics.max > 10 * new_metrics.average() {
                    repeat = Some((i, new_metrics.max / 2));
                    break;
                }
                metrics = new_metrics;
            }
            repeat
        };

        // If we find a repeat marker, only treat pulses before it as bits. Otherwise,
        // treat all pulses as bits.
        let is_bit = |i| match repeat_marker {
            Some((bit_count, _)) => i < bit_count,
            None => true,
        };
        let filter_bits = |(i, b)| is_bit(i).then(|| b);
        let bit_mark = || mark().enumerate().filter_map(filter_bits);
        let bit_space = || space().enumerate().filter_map(filter_bits);

        let bit_mark_divider = {
            let metrics = bit_mark().fold(Metrics::new(), |acc, value| acc.accumulate(*value));
            (metrics.min + metrics.max) / 2
        };
        let bit_space_divider = {
            let metrics = bit_space().fold(Metrics::new(), |acc, value| acc.accumulate(*value));
            (metrics.min + metrics.max) / 2
        };

        let bit0_mark = avg(bit_mark().filter(|v| v <= &&bit_mark_divider));
        let bit0_space = avg(bit_space().filter(|v| v <= &&bit_space_divider));
        let bit1_mark = avg(bit_mark().filter(|v| v > &&bit_mark_divider));
        let bit1_space = avg(bit_space().filter(|v| v > &&bit_space_divider));

        // Decide if it is Pulse Width Modulation or Pulse Distance Modulation.
        let is_pwm = bit1_mark.saturating_sub(bit0_mark) > bit1_space.saturating_sub(bit0_space);

        let bit_extractor = |bit: &(u32, u32)| {
            let bit = if is_pwm {
                bit.0 <= bit_mark_divider
            } else {
                bit.1 <= bit_space_divider
            };
            if bit {
                '0'
            } else {
                '1'
            }
        };

        println!(
            "Encoding type:  {}",
            if is_pwm {
                "Pulse Width Modulation"
            } else {
                "Pulse Distance Modulation"
            }
        );
        println!();
        println!("Average timings:");
        println!("- Preamble mark:  {}", preamble_mark);
        println!("- Preamble space: {}", preamble_space);
        println!("- Bit0 mark:      {}", bit0_mark);
        println!("- Bit0 space:     {}", bit0_space);
        println!("- Bit1 mark:      {}", bit1_mark);
        println!("- Bit1 space:     {}", bit1_space);
        println!();
        println!("Button data | Extra data (if present)");
        for button in buttons {
            println!("- {}", button.name);
            if let ButtonKind::Raw(raw) = button.kind {
                print!(" ");
                for (i, bit) in raw
                    .data
                    .iter()
                    .skip(1)
                    .enumerate()
                    .filter(|(i, _)| is_bit(*i))
                {
                    print!(
                        "{}{}",
                        if i % 8 == 0 { " " } else { "" },
                        bit_extractor(bit),
                    );
                }
                print!(" |");
                for value in raw
                    .data
                    .iter()
                    .skip(1)
                    .enumerate()
                    .filter_map(|(i, b)| (!is_bit(i)).then(|| b))
                    .flat_map(|(a, b)| [a, b])
                    .chain(raw.final_on.as_ref())
                {
                    print!(" {}", value);
                }
                println!();
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
struct Metrics {
    count: u32,
    sum: u32,
    min: u32,
    max: u32,
}

impl Metrics {
    fn new() -> Self {
        Metrics {
            count: 0,
            sum: 0,
            min: u32::MAX,
            max: 0,
        }
    }

    fn accumulate(&self, value: u32) -> Self {
        Metrics {
            count: self.count + 1,
            sum: self.sum + value,
            min: self.min.min(value),
            max: self.max.max(value),
        }
    }

    fn average(&self) -> u32 {
        match self.count {
            0 => 0,
            _ => self.sum / self.count,
        }
    }
}

fn avg<'a>(iter: impl Iterator<Item = &'a u32>) -> u32 {
    let (count, sum) = iter.fold((0, 0), |(count, sum), val| (count + 1, sum + val));
    sum / count
}
