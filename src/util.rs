use std::io::{Result, Write};

pub(crate) fn join_write_bytes<'a>(
    writer: &mut dyn Write,
    sep: &[u8],
    mut parts: impl Iterator<Item = &'a [u8]>,
) -> Result<()> {
    match parts.next() {
        None => {}
        Some(first) => {
            writer.write_all(first)?;

            for part in parts {
                writer.write_all(sep)?;
                writer.write_all(part)?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_write_bytes_empty() {
        let mut buf = Vec::new();
        join_write_bytes(&mut buf, b"|", std::iter::empty()).unwrap();
        assert_eq!(buf, b"");
    }

    #[test]
    fn join_write_bytes_single() {
        let mut buf = Vec::new();
        let parts: &[&[u8]] = &[b"hello"];
        join_write_bytes(&mut buf, b"|", parts.iter().copied()).unwrap();
        assert_eq!(buf, b"hello");
    }

    #[test]
    fn join_write_bytes_multiple() {
        let mut buf = Vec::new();
        let parts: &[&[u8]] = &[b"a", b"b", b"c"];
        join_write_bytes(&mut buf, b"|", parts.iter().copied()).unwrap();
        assert_eq!(buf, b"a|b|c");
    }
}
