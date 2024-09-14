use core::str;
use std::{self, io::BufRead, process::ExitCode};

mod asciinum;
use asciinum::*;

fn default_conversion(number: u128) -> String {
    convert_to_ascii(
        number,
        &RadixSettings::new(
            RadixSymbols::Disabled,
            RadixNumbers::All,
            RadixLetters::SensitiveOrdered,
        ),
    )
}

fn main() -> ExitCode {
    let mut exit_code = ExitCode::SUCCESS;
    let mut stdin = std::io::stdin().lock();
    let mut buffer = Vec::with_capacity(40);
    loop {
        match stdin.read_until(b'\n', &mut buffer) {
            Ok(0) => {
                // we have reached end of stdin stream
                break;
            }
            Ok(_) => {
                let btrim = buffer.trim_ascii_control();
                if btrim.is_empty() {
                    continue;
                }
                match str::from_utf8(btrim) {
                    Ok(line) => match line.parse::<u128>() {
                        Ok(number) => {
                            println!("{}", default_conversion(number));
                        }
                        Err(err) => {
                            eprintln!("couldn't parse as integer `{}`: {}", line, err);
                            exit_code = ExitCode::from(2);
                        }
                    },
                    Err(err) => {
                        eprintln!(
                            "couldn't parse ``{}``: {}",
                            String::from_utf8_lossy(btrim),
                            err
                        );
                        exit_code = ExitCode::from(2);
                    }
                };
            }
            Err(err) => {
                eprintln!("couldn't read stream: {}", err);
                exit_code = ExitCode::FAILURE;
                break;
            }
        }
        buffer.clear()
    }
    exit_code
}
