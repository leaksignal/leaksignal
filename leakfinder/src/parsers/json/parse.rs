use futures::{AsyncRead, AsyncReadExt};

use crate::pipe::PipeReader;
use anyhow::{bail, Context, Result};
use std::{
    char::REPLACEMENT_CHARACTER,
    io::{ErrorKind, Result as IoResult},
};

async fn read_u8(reader: &mut (impl AsyncRead + Unpin)) -> IoResult<u8> {
    let mut out = [0u8; 1];
    reader.read_exact(&mut out[..]).await?;
    Ok(out[0])
}

async fn maybe_read_u8(reader: &mut (impl AsyncRead + Unpin)) -> IoResult<Option<u8>> {
    let mut out = [0u8; 1];
    match reader.read_exact(&mut out[..]).await {
        Ok(()) => (),
        Err(e) if e.kind() == ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }
    Ok(Some(out[0]))
}

async fn read_u8_non_whitespace(reader: &mut (impl AsyncRead + Unpin)) -> IoResult<u8> {
    loop {
        let value = read_u8(reader).await?;
        if value == b' ' || value == b'\n' || value == b'\r' || value == b'\t' {
            continue;
        }
        break Ok(value);
    }
}

async fn maybe_read_u8_non_whitespace(
    reader: &mut (impl AsyncRead + Unpin),
) -> IoResult<Option<u8>> {
    loop {
        let value = match maybe_read_u8(reader).await? {
            Some(x) => x,
            None => break Ok(None),
        };
        if value == b' ' || value == b'\n' || value == b'\r' || value == b'\t' {
            continue;
        }
        break Ok(Some(value));
    }
}

async fn reread_u8_non_whitespace(
    first_byte: u8,
    reader: &mut (impl AsyncRead + Unpin),
) -> IoResult<u8> {
    if first_byte != b' ' && first_byte != b'\n' && first_byte != b'\r' && first_byte != b'\t' {
        return Ok(first_byte);
    }
    read_u8_non_whitespace(reader).await
}

fn decode_hex(c: u8) -> Option<u8> {
    Some(match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => return None,
    })
}

fn decode_hex4(bytes: [u8; 4]) -> Option<u16> {
    let mut value: u16 = (decode_hex(bytes[0])? as u16) << 12;
    value |= (decode_hex(bytes[1])? as u16) << 8;
    value |= (decode_hex(bytes[2])? as u16) << 4;
    value |= decode_hex(bytes[3])? as u16;
    Some(value)
}

async fn read_hex4(input: &mut PipeReader) -> IoResult<Result<u16, [u8; 4]>> {
    let mut bytes = [0u8; 4];
    input.read_exact(&mut bytes[..]).await?;
    Ok(decode_hex4(bytes).ok_or(bytes))
}

async fn read_char_with(input: &mut PipeReader, start: u8) -> IoResult<char> {
    let codepoint = if start < 0x80 {
        start as u32
    } else if start >> 5 == 0b110 {
        let byte2 = read_u8(input).await?;
        ((start & 0b00011111) as u32) << 6 | (byte2 & 0b00111111) as u32
    } else if start >> 4 == 0b1110 {
        let byte2 = read_u8(input).await?;
        let byte3 = read_u8(input).await?;
        ((start & 0b00001111) as u32) << 12
            | ((byte2 & 0b00111111) as u32) << 6
            | ((byte3 & 0b00111111) as u32)
    } else if start >> 3 == 0b11110 {
        let byte2 = read_u8(input).await?;
        let byte3 = read_u8(input).await?;
        let byte4 = read_u8(input).await?;
        ((start & 0b00000111) as u32) << 18
            | ((byte2 & 0b00111111) as u32) << 12
            | ((byte3 & 0b00111111) as u32) << 6
            | (byte4 & 0b00111111) as u32
    } else {
        // malformed unicode
        return Ok(std::char::REPLACEMENT_CHARACTER);
    };

    Ok(std::char::from_u32(codepoint).unwrap_or(std::char::REPLACEMENT_CHARACTER))
}

// assumes the leading " is already consumed
async fn read_json_string(input: &mut PipeReader) -> IoResult<String> {
    let mut out = String::new();
    let mut current_unicode_sequence: Vec<u16> = vec![];
    loop {
        let char1 = read_u8(input).await?;
        if char1 != b'\\' && !current_unicode_sequence.is_empty() {
            out.extend(
                char::decode_utf16(current_unicode_sequence.drain(..))
                    .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER)),
            );
        }
        match char1 {
            b'\\' => {
                let char2 = read_u8(input).await?;
                if char2 != b'u' && !current_unicode_sequence.is_empty() {
                    out.extend(
                        char::decode_utf16(current_unicode_sequence.drain(..))
                            .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER)),
                    );
                }
                match char2 {
                    b'"' => {
                        out.push('"');
                    }
                    b'\\' => {
                        out.push('\\');
                    }
                    b'/' => {
                        out.push('/');
                    }
                    b'b' => {
                        out.push('\x08'); // backspace
                    }
                    b'f' => {
                        out.push('\x0C'); // formfeed
                    }
                    b'n' => {
                        out.push('\n');
                    }
                    b'r' => {
                        out.push('\r');
                    }
                    b't' => {
                        out.push('\t');
                    }
                    b'u' => match read_hex4(input).await? {
                        Ok(hex) => current_unicode_sequence.push(hex),
                        Err(_bytes) => {
                            let mut out = [0u16; 1];
                            REPLACEMENT_CHARACTER.encode_utf16(&mut out[..]);
                            current_unicode_sequence.push(out[0]);
                        }
                    },
                    c => out.push(read_char_with(input, c).await?),
                }
            }
            b'"' => {
                break;
            }
            c => {
                out.push(read_char_with(input, c).await?);
            }
        }
    }
    if !current_unicode_sequence.is_empty() {
        out.extend(
            char::decode_utf16(current_unicode_sequence.drain(..))
                .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER)),
        );
    }
    Ok(out)
}

fn decode_digit(c: u8) -> Option<u8> {
    Some(match c {
        b'0'..=b'9' => c - b'0',
        _ => return None,
    })
}

// returns first byte after number, if any (not eof)
async fn read_json_number(first_byte: u8, input: &mut PipeReader) -> Result<Option<u8>> {
    let first_number = match first_byte {
        b'-' => {
            decode_digit(read_u8(input).await?).context("missing digit after negative symbol")?
        }
        b'0'..=b'9' => first_byte - b'0',
        _ => unreachable!(),
    };

    let mut post_digit = if first_number == 0 {
        match maybe_read_u8(input).await? {
            None => return Ok(None),
            Some(c) => c,
        }
    } else {
        loop {
            match maybe_read_u8(input).await? {
                Some(c) if c.is_ascii_digit() => (),
                Some(c) => break c,
                None => return Ok(None),
            };
        }
    };
    if post_digit == b'.' {
        post_digit = loop {
            match maybe_read_u8(input).await? {
                Some(c) if c.is_ascii_digit() => (),
                Some(c) => break c,
                None => return Ok(None),
            };
        };
    }
    if post_digit == b'e' || post_digit == b'E' {
        match maybe_read_u8(input).await? {
            Some(c) if c.is_ascii_digit() || c == b'+' || c == b'-' => (),
            c => return Ok(c),
        }
        post_digit = loop {
            match maybe_read_u8(input).await? {
                Some(c) if c.is_ascii_digit() => (),
                Some(c) => break c,
                None => return Ok(None),
            };
        };
    }
    Ok(Some(post_digit))
}

enum JsonResult<T> {
    Eof,
    NextByte(u8),
    EarlyReturn(T),
}

/// returns trailing byte after read, if any
#[async_recursion::async_recursion(?Send)]
async fn parse_json_internal<T>(
    first_byte: u8,
    input: &mut PipeReader,
    key_out: &mut impl FnMut(String, usize, usize) -> Option<T>,
    value_out: &mut impl FnMut(String, usize, usize) -> Option<T>,
) -> Result<JsonResult<T>> {
    let start = input.total_read().saturating_sub(1);
    match first_byte {
        // object
        b'{' => {
            let mut first_key = true;
            loop {
                let start_key = input.total_read();
                match read_u8_non_whitespace(input).await? {
                    b'}' if first_key => {
                        break;
                    },
                    b'"' => {
                        let key = read_json_string(input).await?;
                        if let Some(out) = key_out(key, start_key, input.total_read()) {
                            return Ok(JsonResult::EarlyReturn(out));
                        }
                    },
                    c => bail!("malformed json, unexpected character {c:?} @ {start_key}, expected object key or end of object"),
                }
                first_key = false;
                let char2 = read_u8_non_whitespace(input).await?;
                if char2 != b':' {
                    bail!("malformed json, unexpected character {char2:?} @ {}, expected ':' for object key", start_key + 1)
                }
                let comma_byte = parse_json_internal(
                    read_u8_non_whitespace(input).await?,
                    input,
                    key_out,
                    value_out,
                )
                .await?;
                let comma_index = input.total_read().saturating_sub(1);
                match comma_byte {
                    JsonResult::Eof => return Err(std::io::Error::new(ErrorKind::UnexpectedEof, "unexpected eof").into()),
                    JsonResult::NextByte(b',') => (),
                    JsonResult::NextByte(b'}') => break,
                    JsonResult::NextByte(c) => bail!("malformed json, unexpected character {c:?} @ {comma_index}, expected comma or end of object"),
                    JsonResult::EarlyReturn(value) => return Ok(JsonResult::EarlyReturn(value)),
                }
            }
        }
        // array
        b'[' => {
            let mut first_value = true;
            loop {
                let first_char = read_u8_non_whitespace(input).await?;
                if first_value && first_char == b']' {
                    break;
                }
                first_value = false;
                let comma_byte = parse_json_internal(first_char, input, key_out, value_out).await?;
                let comma_index = input.total_read().saturating_sub(1);
                match comma_byte {
                    JsonResult::Eof => return Err(std::io::Error::new(ErrorKind::UnexpectedEof, "unexpected eof").into()),
                    JsonResult::NextByte(b',') => (),
                    JsonResult::NextByte(b']') => break,
                    JsonResult::NextByte(c) => bail!("malformed json, unexpected character {c:?} @ {comma_index}, expected comma or end of array"),
                    JsonResult::EarlyReturn(value) => return Ok(JsonResult::EarlyReturn(value)),
                }
            }
        }
        // string
        b'"' => {
            let value = read_json_string(input).await?;
            if let Some(out) = value_out(value, start, input.total_read()) {
                return Ok(JsonResult::EarlyReturn(out));
            }
        }
        // number
        b'-' | b'0' | b'1' | b'2' | b'3' | b'4' | b'5' | b'6' | b'7' | b'8' | b'9' => {
            return match read_json_number(first_byte, input).await? {
                Some(c) => Ok(JsonResult::NextByte(
                    reread_u8_non_whitespace(c, input).await?,
                )),
                None => Ok(JsonResult::Eof),
            };
        }
        b't' => {
            let mut word = [0u8; 4];
            word[0] = b't';
            input.read_exact(&mut word[1..]).await?;
            if &word != b"true" {
                bail!("malformed json, unknown keyword: {word:?}")
            }
        }
        b'f' => {
            let mut word = [0u8; 5];
            word[0] = b'f';
            input.read_exact(&mut word[1..]).await?;
            if &word != b"false" {
                bail!("malformed json, unknown keyword: {word:?}")
            }
        }
        b'n' => {
            let mut word = [0u8; 4];
            word[0] = b'n';
            input.read_exact(&mut word[1..]).await?;
            if &word != b"null" {
                bail!("malformed json, unknown keyword: {word:?}")
            }
        }
        // unexpected
        c => {
            bail!("malformed json, unexpected character {c:?} @ {start}");
        }
    }
    Ok(maybe_read_u8_non_whitespace(input)
        .await?
        .map(JsonResult::NextByte)
        .unwrap_or(JsonResult::Eof))
}

pub async fn parse_json<T>(
    input: &mut PipeReader,
    mut key_out: impl FnMut(String, usize, usize) -> Option<T>,
    mut value_out: impl FnMut(String, usize, usize) -> Option<T>,
) -> Result<Option<T>> {
    match parse_json_internal(
        read_u8_non_whitespace(input).await?,
        input,
        &mut key_out,
        &mut value_out,
    )
    .await?
    {
        JsonResult::NextByte(_) | JsonResult::Eof => Ok(None),
        JsonResult::EarlyReturn(t) => Ok(Some(t)),
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, task::Poll};

    use futures::{pin_mut, task::waker, Future};

    use crate::pipe::{pipe, DummyWaker};

    use super::*;

    #[test]
    fn test_json_parse() {
        let raw: Vec<&str> = include_str!("./parse_tests.txt")
            .split('\n')
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .collect::<Vec<_>>();
        for value in raw {
            let (mut reader, mut writer) = pipe(0);
            assert!(writer.append(value.as_bytes()));
            drop(writer);
            let mut keys = vec![];
            let mut values = vec![];
            let waker = waker(Arc::new(DummyWaker));
            let mut context = std::task::Context::from_waker(&waker);

            {
                let future = parse_json::<()>(
                    &mut reader,
                    |key, _, _| {
                        keys.push(key);
                        None
                    },
                    |value, _, _| {
                        values.push(value);
                        None
                    },
                );
                pin_mut!(future);
                match future.poll(&mut context) {
                    Poll::Ready(Ok(None)) => (), // pass
                    Poll::Ready(Ok(Some(_))) => panic!("json shouldn't early return in tests"),
                    Poll::Ready(Err(e)) => panic!("json error in test: {e:?} for: {value}"),
                    Poll::Pending => panic!("json shouldn't be pending on a closed pipe"),
                }
            }

            println!("json: {value}");
            println!("keys: {:?}", keys);
            println!("values: {:?}\n", values);
        }
    }
}
