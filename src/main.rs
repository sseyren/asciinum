use core::str;
use std::{self, io::BufRead, process::ExitCode};

mod asciinum;
use asciinum::*;

fn parse_radix_arg(arg: &str) -> Result<RadixSettings, String> {
    if let (Some(s), Some(n), Some(l), None) = (
        arg.chars().nth(0),
        arg.chars().nth(1),
        arg.chars().nth(2),
        arg.chars().nth(3),
    ) {
        let symbols = match s {
            'a' => RadixSymbols::All,
            'u' => RadixSymbols::UnixSafe,
            'd' => RadixSymbols::Disabled,
            _ => return Err("first character of radix arg must be one of these: {a,u,d}".into()),
        };
        let numbers = match n {
            'a' => RadixNumbers::All,
            'd' => RadixNumbers::Disabled,
            _ => return Err("second character of radix arg must be one of these: {a,d}".into()),
        };
        let letters = match l {
            'i' => RadixLetters::Insensitive,
            's' => RadixLetters::Sensitive,
            'o' => RadixLetters::SensitiveOrdered,
            _ => return Err("third character of radix arg must be one of these: {i,s,o}".into()),
        };
        Ok(RadixSettings::new(symbols, numbers, letters))
    } else {
        Err("must be 3 characters long".into())
    }
}

fn main() -> ExitCode {
    let mut argv: Vec<String> = Vec::new();
    let mut program_args = std::env::args_os();
    program_args.next(); // skip first arg
    for arg in program_args {
        match arg.into_string() {
            Ok(sarg) => argv.push(sarg),
            Err(arg) => {
                eprintln!(
                    "couldn't parse program arg `{}`: it's not a valid utf-8 string",
                    arg.to_string_lossy(),
                );
                return ExitCode::FAILURE;
            }
        }
    }
    if argv
        .iter()
        .find(|&arg| arg == "-h" || arg == "--help")
        .is_some()
    {
        println!("help string");
        return ExitCode::SUCCESS;
    }
    if argv.len() > 1 {
        eprintln!("too many arguments. use `--help` for more info.");
        return ExitCode::FAILURE;
    }

    let settings = match argv.pop() {
        Some(arg) => match parse_radix_arg(&arg) {
            Ok(settings) => settings,
            Err(err) => {
                eprintln!("couldn't parse program arg `{}`: {}", arg, err);
                return ExitCode::FAILURE;
            }
        },
        None => RadixSettings::new(
            RadixSymbols::Disabled,
            RadixNumbers::All,
            RadixLetters::SensitiveOrdered,
        ),
    };
    let converter = AsciiConverter::new(&settings);

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
                            println!("{}", converter.convert(number));
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
