use core::str;
use std::{self, io::BufRead, process::ExitCode};

mod asciinum;
use asciinum::*;

const CLI_HELP_TEXT: &str = r##"
Reads numbers from stdin & expresses them with ASCII characters.

Usage: asciinum {-h,--help}
       asciinum [RADIXOPT]

RADIXOPT: You can change what characters will be used for representing numbers
 with ASCII characters. This can be done with this argument. This option always
 needs to be 3 characters long and order of letters are significant.

 First character determines symbols; can be one of these: {a,u,d}
 * a -> all: every non-alphanumeric character in ASCII table (no control codes)
          corpus => !"#$%&'()*+,-./:;<=>?@[\]^_`{|}~
 * u -> unix safe: same as `a` but `/` character omitted
          corpus => !"#$%&'()*+,-.:;<=>?@[\]^_`{|}~
 * d -> disabled: there will be no symbols in output

 Second character determines numbers; can be one of these: {a,d}
 * a -> all: every numeric character in ASCII table
          corpus => 0123456789
 * d -> disabled: there will be no numbers in output

 Third character determines letters; can be one of these: {i,s,o}
 * i -> insensitive: every lowercase alphabetical character in ASCII table
          corpus => abcdefghijklmnopqrstuvwxyz
 * s -> sensitive: same as `i` but includes uppercase characters as well.
        starts with uppercase letters.
          corpus => ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz
 * o -> ordered: same as `s` but different ordering
          corpus => AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz

 These three choices of you made will be merged in the same order:
   [symbols][characters][letters]
 and generate radix (digits) for output numbers.

 Default value for RADIXOPT is: `dao`.
"##;

/// Parses radix settings program argument and returns
/// [`asciinum::RadixSettings`]. If encounters with an error, it returns error
/// message as String.
fn parse_radix_arg(arg: &str) -> Result<RadixSettings, String> {
    let mut chars = arg.chars();
    if let (Some(s), Some(n), Some(l), None) =
        (chars.next(), chars.next(), chars.next(), chars.next())
    {
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
    if argv.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!("{}", CLI_HELP_TEXT.trim());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_radix_arg() {
        assert!(parse_radix_arg("a").is_err());
        assert!(parse_radix_arg("aa").is_err());
        assert!(parse_radix_arg("aaix").is_err());

        assert!(parse_radix_arg("asd").is_err());
        assert!(parse_radix_arg("efg").is_err());
        assert!(parse_radix_arg("haha\r\nhehe").is_err());

        assert!(parse_radix_arg("iua").is_err());
        assert!(parse_radix_arg("suu").is_err());
        assert!(parse_radix_arg("oud").is_err());

        assert_eq!(
            parse_radix_arg("aai").unwrap(),
            RadixSettings::new(
                RadixSymbols::All,
                RadixNumbers::All,
                RadixLetters::Insensitive
            )
        );
        assert_eq!(
            parse_radix_arg("udi").unwrap(),
            RadixSettings::new(
                RadixSymbols::UnixSafe,
                RadixNumbers::Disabled,
                RadixLetters::Insensitive
            )
        );
        assert_eq!(
            parse_radix_arg("das").unwrap(),
            RadixSettings::new(
                RadixSymbols::Disabled,
                RadixNumbers::All,
                RadixLetters::Sensitive
            )
        );
        assert_eq!(
            parse_radix_arg("ado").unwrap(),
            RadixSettings::new(
                RadixSymbols::All,
                RadixNumbers::Disabled,
                RadixLetters::SensitiveOrdered
            )
        );
    }
}
