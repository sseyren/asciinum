use std::num::NonZeroUsize;

use constcat;

const SYMBOLS: &str = r##"!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~"##;
const SYMBOLS_UNIXSAFE: &str = r##"!"#$%&'()*+,-.:;<=>?@[\]^_`{|}~"##;

pub enum RadixSymbols {
    All,
    /// excludes '/'
    UnixSafe,
    Disabled,
}

const NUMBERS: &str = "0123456789";

pub enum RadixNumbers {
    All,
    Disabled,
}

const LETTERS_UPPERCASE: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const LETTERS_LOWERCASE: &str = "abcdefghijklmnopqrstuvwxyz";
const LETTERS_CONCAT: &str = constcat::concat!(LETTERS_UPPERCASE, LETTERS_LOWERCASE);
const LETTERS_ORDERED: &str = "AaBbCcDdEeFfGgHhIiJjKkLlMmNnOoPpQqRrSsTtUuVvWwXxYyZz";

pub enum RadixLetters {
    /// \[a-z\]
    Insensitive,
    /// \[A-Z\]\[a-z\]
    Sensitive,
    /// \[AaBb-Zz\]
    SensitiveOrdered,
}

pub struct RadixSettings {
    pub symbols: RadixSymbols,
    pub numbers: RadixNumbers,
    pub letters: RadixLetters,
}

impl RadixSettings {
    pub fn new(symbols: RadixSymbols, numbers: RadixNumbers, letters: RadixLetters) -> Self {
        Self {
            symbols,
            numbers,
            letters,
        }
    }
    fn corpus(self: &RadixSettings) -> String {
        String::new()
            + (match self.symbols {
                RadixSymbols::All => SYMBOLS,
                RadixSymbols::UnixSafe => SYMBOLS_UNIXSAFE,
                RadixSymbols::Disabled => "",
            })
            + (match self.numbers {
                RadixNumbers::All => NUMBERS,
                RadixNumbers::Disabled => "",
            })
            + (match self.letters {
                RadixLetters::Insensitive => LETTERS_LOWERCASE,
                RadixLetters::Sensitive => LETTERS_CONCAT,
                RadixLetters::SensitiveOrdered => LETTERS_ORDERED,
            })
    }
}

/// Base translation operation as an iteration.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct BaseConvertIter {
    // starts with Some, at final step this becomes None
    number: Option<u128>,
    // TODO 1 should be forbidden too
    base: NonZeroUsize,
}

impl BaseConvertIter {
    fn new(number: u128, base: NonZeroUsize) -> Self {
        Self {
            number: Some(number),
            base,
        }
    }
}

impl Iterator for BaseConvertIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(number) = self.number {
            let base = self.base.get() as u128;
            if number < base {
                self.number = None;
                // we know `number` is smaller than base(usize)
                Some(number as usize)
            } else {
                let left = number % base;
                self.number = Some((number - left) / base);
                // because of modulo, we know `left` is smaller than base(usize)
                Some(left as usize)
            }
        } else {
            None
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AsciiConverter {
    corpus: String,
}

impl AsciiConverter {
    pub fn new(settings: &RadixSettings) -> Self {
        Self {
            corpus: settings.corpus(),
        }
    }
    /// Does decimal to ascii numbers conversion.
    ///
    /// ```
    /// let converter = AsciiConverter::new(&RadixSettings::new(
    ///     RadixSymbols::Disabled,
    ///     RadixNumbers::Disabled,
    ///     RadixLetters::Insensitive,
    /// ));
    /// assert_eq!(converter.convert(123), "et");
    /// ```
    pub fn convert(&self, decimal: u128) -> String {
        let number: String = BaseConvertIter::new(
            decimal,
            NonZeroUsize::new(self.corpus.len()).expect("we know that corpus.len() is > 0"),
        )
        .map(|digit| {
            self.corpus
                .chars()
                .nth(digit)
                .expect("corpus.len() will be always bigger than digit itself")
        })
        .collect();
        // it's okay to use .rev() here becase we know that every character in this
        // string is an ASCII character
        number.chars().rev().collect()
    }
}

pub trait TrimAsciiControlCharacters {
    /// Returns a byte slice with leading and trailing ASCII control bytes
    /// removed.
    ///
    /// 'Control bytes' refers to the definition used by
    /// [`u8::is_ascii_control`].
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(
    ///     b"\r hello world\n ".trim_ascii_control(),
    ///     b" hello world\n "
    /// );
    /// assert_eq!(b"  ".trim_ascii_control(), b"  ");
    /// assert_eq!(b"".trim_ascii_control(), b"");
    /// ```
    fn trim_ascii_control(&self) -> &[u8];
}

impl TrimAsciiControlCharacters for [u8] {
    fn trim_ascii_control(&self) -> &[u8] {
        let from = match self.iter().position(|x| !x.is_ascii_control()) {
            Some(i) => i,
            None => return &self[0..0],
        };
        let to = self
            .iter()
            .rposition(|x| !x.is_ascii_control())
            .expect("we know that at least one non-control ascii character exists");
        &self[from..=to]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_convert_iter() {
        assert_eq!(
            BaseConvertIter::new(0, NonZeroUsize::new(2).expect("not zero"))
                .collect::<Vec<usize>>(),
            vec![0]
        );
        assert_eq!(
            BaseConvertIter::new(5, NonZeroUsize::new(1000).expect("not zero"))
                .collect::<Vec<usize>>(),
            vec![5]
        );
        assert_eq!(
            BaseConvertIter::new(123456, NonZeroUsize::new(62).expect("not zero"))
                .collect::<Vec<usize>>(),
            vec![14, 7, 32]
        );
        assert_eq!(
            BaseConvertIter::new(837, NonZeroUsize::new(2).expect("not zero"))
                .collect::<Vec<usize>>(),
            vec![1, 0, 1, 0, 0, 0, 1, 0, 1, 1]
        );
        assert_eq!(
            BaseConvertIter::new(u128::MAX, NonZeroUsize::new(19209).expect("not zero"))
                .collect::<Vec<usize>>(),
            vec![3, 8970, 6739, 1611, 15517, 3461, 285, 18953, 18356]
        );
    }

    #[test]
    fn test_convert_to_ascii() {
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::Disabled,
                RadixNumbers::Disabled,
                RadixLetters::Insensitive,
            ))
            .convert(0),
            "a"
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::Disabled,
                RadixNumbers::All,
                RadixLetters::Insensitive,
            ))
            .convert(0),
            "0"
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::All,
                RadixNumbers::All,
                RadixLetters::Insensitive,
            ))
            .convert(0),
            "!"
        );

        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::All,
                RadixNumbers::All,
                RadixLetters::Insensitive,
            ))
            .convert(u128::MAX),
            r##"")+'4/yf`dygv?{w*kdvnj"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::UnixSafe,
                RadixNumbers::All,
                RadixLetters::Insensitive,
            ))
            .convert(u128::MAX),
            r##""4{173{}g^z'ikw8_<,~g@"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::Disabled,
                RadixNumbers::All,
                RadixLetters::Insensitive,
            ))
            .convert(u128::MAX),
            r##"f5lxx1zz5pnorynqglhzmsp33"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::All,
                RadixNumbers::Disabled,
                RadixLetters::Insensitive,
            ))
            .convert(u128::MAX),
            r##"~d{gxl\c&<]zql%l/"t@>v"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::UnixSafe,
                RadixNumbers::Disabled,
                RadixLetters::Insensitive,
            ))
            .convert(u128::MAX),
            r##"oa-q&ov]c>q%`n:?[wd'*$"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::Disabled,
                RadixNumbers::Disabled,
                RadixLetters::Insensitive,
            ))
            .convert(u128::MAX),
            r##"cdhefomrsrxetmsvhtomcungjkbv"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::All,
                RadixNumbers::All,
                RadixLetters::Sensitive,
            ))
            .convert(u128::MAX),
            r##",#7zGM_d_e&[bar**m,."##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::UnixSafe,
                RadixNumbers::All,
                RadixLetters::Sensitive,
            ))
            .convert(u128::MAX),
            r##".GB;(nA-8hN0iu?pLo4c"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::Disabled,
                RadixNumbers::All,
                RadixLetters::Sensitive,
            ))
            .convert(u128::MAX),
            r##"7n42DGM5Tflk9n8mt7Fhc7"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::All,
                RadixNumbers::Disabled,
                RadixLetters::Sensitive,
            ))
            .convert(u128::MAX),
            r##""*EwscH.T$Oa?x^]GS@f$"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::UnixSafe,
                RadixNumbers::Disabled,
                RadixLetters::Sensitive,
            ))
            .convert(u128::MAX),
            r##""D`<#<C\nsD>`T%lg._?T"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::Disabled,
                RadixNumbers::Disabled,
                RadixLetters::Sensitive,
            ))
            .convert(u128::MAX),
            r##"GBIWTpZqojFGQPQPXtvbJAv"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::All,
                RadixNumbers::All,
                RadixLetters::SensitiveOrdered,
            ))
            .convert(u128::MAX),
            r##",#7zDG_o_P&[nNv**T,."##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::UnixSafe,
                RadixNumbers::All,
                RadixLetters::SensitiveOrdered,
            ))
            .convert(u128::MAX),
            r##".Da;(tA-8qg0RX?ufU4O"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::Disabled,
                RadixNumbers::All,
                RadixLetters::SensitiveOrdered,
            ))
            .convert(u128::MAX),
            r##"7t42bDG5jpsS9t8Tw7cqO7"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::All,
                RadixNumbers::Disabled,
                RadixLetters::SensitiveOrdered,
            ))
            .convert(u128::MAX),
            r##""*CYWOd.j$HN?y^]DJ@p$"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::UnixSafe,
                RadixNumbers::Disabled,
                RadixLetters::SensitiveOrdered,
            ))
            .convert(u128::MAX),
            r##""b`<#<B\tWb>`j%sQ._?j"##
        );
        assert_eq!(
            AsciiConverter::new(&RadixSettings::new(
                RadixSymbols::Disabled,
                RadixNumbers::Disabled,
                RadixLetters::SensitiveOrdered,
            ))
            .convert(u128::MAX),
            r##"DaELjumVUrcDIhIhlwxneAx"##
        );
    }

    #[test]
    fn test_trim_ascii_control() {
        assert_eq!(b"\t\n\rX\x00\x1f\x7F".trim_ascii_control(), b"X");
        assert_eq!(b" X ".trim_ascii_control(), b" X ");
        assert_eq!(b" \rX\n ".trim_ascii_control(), b" \rX\n ");
        assert_eq!(Vec::from(b"\t \rX\n \x00").trim_ascii_control(), b" \rX\n ");
        assert_eq!(Vec::from(b"\x80X\x9f").trim_ascii_control(), b"\x80X\x9f");
        assert_eq!(Vec::from(b"asd").trim_ascii_control(), b"asd");
        assert_eq!(Vec::from(b"").trim_ascii_control(), b"");
    }
}
