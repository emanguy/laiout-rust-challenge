use std::io::Write;

/// Forwards string data to an inner writer, performing a ROT13 cipher
/// on alphabet characters before forwarding them to the inner writer.
pub struct Rot13Writer<T>
where
    T: Write,
{
    pub inner: T,
}

impl<T> Rot13Writer<T>
where
    T: Write,
{
    /// Constructs a new Rot13Writer
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

/// Lower bound of the lowercase alphabet as a u8.
const LOWER_BEGIN_U8: u8 = 'a' as u8;
/// Upper bound of the lowercase alphabet as a u8.
const LOWER_END_U8: u8 = 'z' as u8;
/// Lower bound of the uppercase alphabet as a u8.
const UPPER_BEGIN_U8: u8 = 'A' as u8;
/// Upper bound of the uppercase alphabet as a u8.
const UPPER_END_U8: u8 = 'Z' as u8;

impl<T> Write for Rot13Writer<T>
where
    T: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut transform_buf: [u8; 8] = [0; 8];
        let mut total_bytes_written: usize = 0;

        if buf.is_empty() {
            return Ok(0);
        }

        for chunk in buf.chunks(8) {
            for (idx, character) in chunk.iter().enumerate() {
                transform_buf[idx] = match character {
                    LOWER_BEGIN_U8..=LOWER_END_U8 => rot13_character(*character, LOWER_BEGIN_U8),
                    UPPER_BEGIN_U8..=UPPER_END_U8 => rot13_character(*character, UPPER_BEGIN_U8),
                    _ => *character,
                }
            }
            total_bytes_written += self.inner.write(&transform_buf[..chunk.len()])?;
        }

        Ok(total_bytes_written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Perform the "rot13" encryption cipher on a single character represented as a u8.
/// The "lower bound" character (i.e. "A" or "a") is included so the function can wrap
/// the character around the range appropriately while preserving case.
fn rot13_character(character: u8, lower_bound_inclusive: u8) -> u8 {
    // Sanity check - character must be within the range to begin with
    if !(lower_bound_inclusive..=(lower_bound_inclusive + 26)).contains(&character) {
        return character;
    }
    // To do a "remainder" properly, we need to make the first character of the range 0
    let adjusted_bound_character = character - lower_bound_inclusive;
    // Once we do that, add 13 to the adjusted character and wrap it around the range via %
    let rotated_character = (adjusted_bound_character + 13) % 26;

    // Adjust the character back into the range
    rotated_character + lower_bound_inclusive
}

#[cfg(test)]
mod tests {
    use super::*;

    mod writer {
        use super::*;

        #[test]
        fn writer_converts_properly() {
            let original_string = "abcdefghijklmnopqrstuvwxyz ABCDEFGHIJKLMNOPQRSTUVWXYZ #!()";
            let expected_result = "nopqrstuvwxyzabcdefghijklm NOPQRSTUVWXYZABCDEFGHIJKLM #!()";

            let mut writer = Rot13Writer::new(Vec::new());
            writer.write_all(original_string.as_bytes()).unwrap();
            writer.flush().unwrap();

            let result = String::from_utf8(writer.inner).unwrap();
            assert_eq!(result, expected_result);
        }
    }

    mod rot13_character {
        use super::*;

        #[test]
        fn returns_original_character_if_outside_range() {
            let resolved_character_low = rot13_character('A' as u8, LOWER_BEGIN_U8);
            let resolved_character_high = rot13_character('Z' as u8, LOWER_BEGIN_U8);
            assert_eq!('A' as u8, resolved_character_low);
            assert_eq!('Z' as u8, resolved_character_high);
        }

        #[test]
        fn converts_lowercase_correctly() {
            let original = "abcdefghijklmnopqrstuvwxyz";
            let converted = "nopqrstuvwxyzabcdefghijklm";

            for (orig_char, conv_char) in original.chars().zip(converted.chars()) {
                let resolved_character = rot13_character(orig_char as u8, LOWER_BEGIN_U8);
                assert_eq!(conv_char as u8, resolved_character);
            }
        }

        #[test]
        fn converts_uppercase_correctly() {
            let original = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
            let converted = "NOPQRSTUVWXYZABCDEFGHIJKLM";

            for (orig_char, conv_char) in original.chars().zip(converted.chars()) {
                let resolved_character = rot13_character(orig_char as u8, UPPER_BEGIN_U8);
                assert_eq!(conv_char as u8, resolved_character);
            }
        }
    }
}
