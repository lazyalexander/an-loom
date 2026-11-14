//! Hash the bytes consumed by tar, rather than trusting a previous source read.
use super::*;
struct Checked<R> {
    input: R,
    hash: Sha256,
    bytes: u64,
}
impl<R: Read> Read for Checked<R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let count = self.input.read(buffer)?;
        self.hash.update(&buffer[..count]);
        self.bytes += count as u64;
        Ok(count)
    }
}
pub(super) fn append<W: Write>(
    archive: &mut tar::Builder<W>,
    name: &str,
    file: &manifest::File,
    input: impl Read,
) -> Result<()> {
    let mut checked = Checked {
        input,
        hash: Sha256::new(),
        bytes: 0,
    };
    super::append(archive, name, file.bytes, &mut checked)?;
    if checked.bytes != file.bytes || format!("{:x}", checked.hash.finalize()) != file.sha256 {
        return Err(fail("runtime source changed during archive creation"));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn changed_bytes_are_rejected_after_the_preflight_window() -> Result<()> {
        let bytes = b"original";
        let file = manifest::File {
            sha256: format!("{:x}", Sha256::digest(bytes)),
            bytes: bytes.len() as u64,
            executable: false,
        };
        let mut archive = tar::Builder::new(Vec::new());
        assert!(append(&mut archive, "payload", &file, &b"modified"[..]).is_err());
        let mut archive = tar::Builder::new(Vec::new());
        append(&mut archive, "payload", &file, &bytes[..])?;
        Ok(())
    }
}
