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
        let bit_mark = || raw().flat_map(|b| b.data.iter().skip(1).map(|(on, _)| on));
        let bit_space = || raw().flat_map(|b| b.data.iter().skip(1).map(|(_, off)| off));

        let preamble_mark = avg(preamble().map(|(on, _)| on));
        let preamble_space = avg(preamble().map(|(_, off)| off));

        let bit_mark_divider = {
            let min_on = bit_mark().min().unwrap();
            let max_on = bit_mark().max().unwrap();
            (min_on + max_on) / 2
        };
        let bit_space_divider = {
            let min_off = bit_space().min().unwrap();
            let max_off = bit_space().max().unwrap();
            (min_off + max_off) / 2
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
        println!("Button data:");
        for button in buttons {
            println!("- {}", button.name);
            if let ButtonKind::Raw(raw) = button.kind {
                print!(" ");
                for (i, bit) in raw.data.iter().skip(1).enumerate() {
                    print!(
                        "{}{}",
                        if i % 8 == 0 { " " } else { "" },
                        bit_extractor(bit),
                    );
                }
                println!();
            }
        }

        Ok(())
    }
}

fn avg<'a>(iter: impl Iterator<Item = &'a u32>) -> u32 {
    let (count, sum) = iter.fold((0, 0), |(count, sum), val| (count + 1, sum + val));
    sum / count
}
