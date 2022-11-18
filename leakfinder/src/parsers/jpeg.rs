use std::{io::{self, ErrorKind, Write, Seek, Cursor}, collections::HashMap};

use exif::{Reader, experimental::Writer, Field};
use indexmap::IndexMap;
use log::{warn, error, debug, info};

use crate::policy::{PolicyAction, POLICY, Category};

use super::ParseResponse;

fn malformed_jpeg(error: impl AsRef<str>) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, error.as_ref())
}

enum JpgMarkerOutput<'a> {
    Exif(&'a [u8]),
    Finished,
    Continue,
    Truncated,
}

/// reads entropy block, return Some(index) where index is the index of the next tag if we reach a new marker, None if we need more input
fn read_jpg_entropy(input: &[u8]) -> Option<usize> {
    let mut i = 0usize;
    loop {
        //todo: do some simd or wide int tricks here
        while i < input.len() && input[i] != 0xFF {
            i += 1;
        }
        if i + 1 >= input.len() {
            return None;
        }
        match input[i + 1] {
            // ignored byte, reset marker
            0x00 | 0xD0..=0xD7 => {
                i += 2;
                continue
            },
            _ => return Some(i),
        }
    }
}

fn read_marker_length(input: &[u8]) -> io::Result<Option<usize>> {
    if input.len() < 2 {
        return Ok(None);
    }
    let length = u16::from_be_bytes((&input[..2]).try_into().unwrap()) as usize;
    if length < 2 {
        return Err(malformed_jpeg("jpeg: length underflow"));
    }
    if input.len() < length {
        return Ok(None);
    }
    Ok(Some(length))
}

fn write_jpg_exif_data(mut output: impl Write + Seek, fields: &[&Field]) -> io::Result<()> {
    let mut writer = Writer::new();
    for field in fields {
        writer.push_field(field);
    }
    writer.write(&mut output, true)
        .map_err(|e| io::Error::new(ErrorKind::InvalidData, format!("bad jpeg EXIF data: {:?}", e)))?;
    Ok(())
}

fn read_jpg_marker_data<'a>(input: &mut &'a [u8], marker: u8) -> io::Result<JpgMarkerOutput<'a>> {
    Ok(match marker {
        0xD8 => JpgMarkerOutput::Continue,
        0xD9 => JpgMarkerOutput::Finished,
        0xE1 => {
            match read_marker_length(input)? {
                None => JpgMarkerOutput::Truncated,
                Some(length) => {
                    let output = &input[2..length];
                    *input = &(*input)[length..];
                    JpgMarkerOutput::Exif(output)
                }
            }
        },
        0xC0 | 0xC2 | 0xC4 | 0xDB | 0xFE | 0xDD | 0xE0..=0xEF => {
            match read_marker_length(input)? {
                None => JpgMarkerOutput::Truncated,
                Some(length) => {
                    *input = &(*input)[length..];
                    JpgMarkerOutput::Continue
                }
            }
        }
        0xDA => {
            let mut total_length = 0;
            match read_marker_length(input)? {
                None => return Ok(JpgMarkerOutput::Truncated),
                Some(length) => {
                    total_length += length;
                }
            }
            match read_jpg_entropy(&input[total_length..]) {
                None => JpgMarkerOutput::Truncated,
                Some(length) => {
                    total_length += length;
                    *input = &(*input)[total_length..];
                    JpgMarkerOutput::Continue
                }
            }
        }
        x => {
            return Err(malformed_jpeg(format!("jpeg: unexpected marker byte: {}", x)));
        }
    })
}

fn read_jpg_marker(input: &[u8]) -> io::Result<(u8, &[u8])> {
    if input.len() < 2 {
        return Err(malformed_jpeg("jpeg: missing expected marker, EOF"));
    }
    if input[0] != 0xFF {
        return Err(malformed_jpeg("jpeg: missing expected marker, invalid"));
    }
    Ok((input[1], &input[2..]))
}

fn process_jpg_frame(input: &[u8], mut output: impl Write, mut filter_field: impl FnMut(&str) -> bool) -> io::Result<()> {
    let original_input = input;
    let (mut marker, mut input) = read_jpg_marker(input)?;

    // check first marker is SOI
    if marker != 0xD8 {
        return Err(malformed_jpeg(format!("jpeg: start marker not SOI: {}", marker)));
    }

    let mut index = 0;

    loop {
        let mut new_input = input;
        (marker, new_input) = read_jpg_marker(new_input)?;
        // println!("marker {}", marker);
        
        let total_read_len = original_input.len() - new_input.len();
        output.write_all(&original_input[index..total_read_len])?;
        index = total_read_len;

        match read_jpg_marker_data(&mut new_input, marker)? {
            JpgMarkerOutput::Exif(data) => {
                let exif_reader = Reader::new();
                //TODO: xmp: 
                // http://ns.adobe.com/xap/1.0/
                // http://ns.adobe.com/xmp/extension/
                
                index = original_input.len() - new_input.len();

                //TODO: identify-im6.q16: Corrupt JPEG data: 2 extraneous bytes before marker 0xe1 `test_deexif.jpg' @ warning/jpeg.c/JPEGWarningHandler/389.
                match exif_reader.read_raw(data[6..].to_vec()) {
                    Ok(exif) => {
                        let mut output_fields = vec![];
                        for field in exif.fields() {
                            debug!("{} = {}", field.tag, field.display_value().with_unit(field));
                            #[cfg(test)]
                            eprintln!("{} = {}", field.tag, field.display_value().with_unit(field));
                            if filter_field(&*field.tag.to_string()) {
                                output_fields.push(field);
                            }
                        }
                        //todo: remove intermediate allocation
                        let mut exif_data = Cursor::new(vec![]);
                        write_jpg_exif_data(&mut exif_data, &output_fields[..])?;
                        let exif_data = exif_data.into_inner();
                        //todo: check for length overflow here
                        output.write_all(&(exif_data.len() as u16 + 6).to_be_bytes()[..])?;
                        //todo: check tag
                        output.write_all(b"Exif\x00\x00")?;
                        output.write_all(&exif_data[..])?;
                    },
                    Err(e) => {
                        output.write_all(&0u16.to_be_bytes()[..])?;
                        warn!("malformed exif: {:?}", e);
                        #[cfg(test)]
                        eprintln!("malformed exif: {:?}", e);
                    }
                }
                // println!("exif = {}", data.len());
                input = new_input;
            },
            JpgMarkerOutput::Finished => {
                input = new_input;
                break;
            },
            JpgMarkerOutput::Continue => {
                input = new_input;
            },
            JpgMarkerOutput::Truncated => return Err(malformed_jpeg("jpeg: truncated")),
        }
    }

    let total_read_len = original_input.len() - input.len();
    output.write_all(&original_input[index..total_read_len])?;

    Ok(())
}

pub fn parse_jpeg(body: &[u8], configuration: &IndexMap<String, PolicyAction>) -> ParseResponse {
    let policy = POLICY.load();
    let mut proper: HashMap<&str, (&String, &PolicyAction)> = HashMap::new();
    let mut do_drop_xmp = false;

    for (category_name, action) in configuration {
        let category = match policy.categories.get(category_name) {
            Some(category) => category,
            None => {
                error!("invalid category in config: {}", category_name);
                continue;
            },
        };

        if let Category::Jpeg { exif_tags, drop_xmp } = category {
            for tag in exif_tags {
                proper.insert(tag, (category_name, action));
            }
            if let Some(drop_xmp) = drop_xmp {
                do_drop_xmp = *drop_xmp;
            }
        } else {
            debug!("skipping non-jpeg category in jpeg");
            continue;
        };
    }

    let mut output = vec![];
    let mut blocked = false;
    info!("processing jpeg");
    info!("rules: {:?}", proper);
    let result = process_jpg_frame(body, &mut output, |field| {
        if let Some((category, action)) = proper.get(&field) {
            if matches!(action, PolicyAction::Ignore) {
                return true;
            }
            info!("matched EXIF tag '{}' -> {:?}", category, action);

            match action {
                PolicyAction::Ignore => unreachable!(),
                PolicyAction::Alert => true,
                PolicyAction::Mask { mask_replacement } => {
                    if !mask_replacement.is_empty() {
                        warn!("mask replacement must be empty for JPEG EXIF matches");
                    }
                    false
                },
                PolicyAction::Block => {
                    blocked = true;
                    false
                },
            }
        } else {
            debug!("unmatched EXIF tag: {}", field);
            true
        }
    });
    if let Err(e) = result {
        error!("jpeg processing failed: {:?}", e);
        return ParseResponse::Replace(vec![]);
    }

    if blocked {
        info!("jpeg image blocked (did you want to `Mask` the fields instead?)");
        return ParseResponse::Replace(vec![]);
    }

    ParseResponse::Replace(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jpeg() {
        let raw = include_bytes!("../../testing/www/test.jpg");
        let mut output = vec![];
        process_jpg_frame(&mut &raw[..], &mut output, |_| true).unwrap();
    }
}
