use std::{collections::VecDeque, io::stdin};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use raptor::RaptorResult;

fn main() -> RaptorResult<()> {
    let fd = stdin();

    let pbm = MultiProgress::new();

    let pb_total = pbm.add(
        ProgressBar::new(0).with_style(
            ProgressStyle::with_template(
                "{prefix:50} {wide_bar:.blue} {pos:>4}/{len:>4} {percent:>4}%",
            )
            .unwrap(),
        ),
    );
    let pb_progress = pbm.add(
        ProgressBar::new(0).with_style(
            ProgressStyle::with_template(
                "{prefix:50} {wide_bar:.green} {pos:>4}/{len:>4} {percent:>4}%",
            )
            .unwrap(),
        ),
    );

    let mut sargs = VecDeque::new();

    let mut maxpkgs = 0;
    let mut offset = 0;

    for line in fd.lines() {
        let line = line?;
        let Some((key, value)) = line.split_once(": ") else {
            continue;
        };

        match key {
            "I" | "W" | "E" => {
                if value == "VALIDATING" {
                    pb_progress.set_length(maxpkgs);
                    pb_progress.inc(1);
                    pb_total.set_position(pb_progress.position());
                }
            }
            "IA" | "WA" | "EA" => {
                sargs.push_back(value.to_string());
            }
            "IF" | "WF" | "EF" => {
                let mut output = value.to_string();
                while !sargs.is_empty() {
                    let next = sargs.pop_front().unwrap();
                    output = output.replacen("%s", &next, 1);
                }
                sargs.clear();

                pb_progress.set_prefix(output);
            }
            "P" => {
                let mut parts = value.split_whitespace().collect::<Vec<&str>>();
                match parts.len() {
                    0 | 1 => continue,
                    2 => parts.push(""),
                    3 => {}
                    _ => continue,
                }
                let prog: u64 = parts[0].parse().unwrap_or(0);
                let size: u64 = parts[1].parse().unwrap_or(0);
                let comm: &str = parts[2];

                match comm {
                    "SIZEDEBS" => {
                        if prog == 0 {
                            continue;
                        }
                        maxpkgs = size;
                        pb_total.set_length(maxpkgs * 4);
                    }
                    "EXTRACTPKGS" => offset = maxpkgs,
                    "UNPACKREQ" => offset = maxpkgs * 2,
                    "CONFREQ" => offset = maxpkgs * 3,
                    "INSTCORE" => continue,
                    _ => {}
                }
                pb_total.set_position(offset + prog);

                pb_progress.set_length(size);
                pb_progress.set_position(prog);
            }
            "PF" => {
                pb_total.set_prefix(value.to_string());
            }

            _ => {
                pbm.println(format!("ERROR: {value}")).unwrap();
            }
        }
    }

    Ok(())
}
