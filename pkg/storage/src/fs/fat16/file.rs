//! File
//!
//! reference: <https://wiki.osdev.org/FAT#Directories_on_FAT12.2F16.2F32>

use super::*;

#[derive(Debug, Clone)]
pub struct File {
    /// The current offset in the file
    offset: usize,
    /// The current cluster of this file
    current_cluster: Cluster,
    /// DirEntry of this file
    entry: DirEntry,
    /// The file system handle that contains this file
    handle: Fat16Handle,
}

impl File {
    pub fn new(handle: Fat16Handle, entry: DirEntry) -> Self {
        Self {
            offset: 0,
            current_cluster: entry.cluster,
            entry,
            handle,
        }
    }

    pub fn length(&self) -> usize {
        self.entry.size as usize
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // FIXME: read file content from disk
        //      CAUTION: file length / buffer size / offset
        //
        //      - `self.offset` is the current offset in the file in bytes
        //      - use `self.handle` to read the blocks
        //      - use `self.entry` to get the file's cluster
        //      - use `self.handle.cluster_to_sector` to convert cluster to sector
        //      - update `self.offset` after reading
        //      - update `self.cluster` with FAT if necessary
        if buf.len() < self.length() as usize {
            warn!("buffer too small");
            return Err(FsError::InvalidOperation);
        }

        let mut length = self.length() as usize;
        let mut block = Block512::default();

        for i in 0..=self.length() as usize / Block512::size() {
            let sector = self.handle.cluster_to_sector(&self.entry.cluster);
            self.handle.inner.read_block(sector + i, &mut block).unwrap();
            if length > Block512::size() {
                buf[i * Block512::size()..(i + 1) * Block512::size()].copy_from_slice(block.as_u8_slice());
                length -= Block512::size();
            } else {
                buf[i * Block512::size()..i * Block512::size() + length].copy_from_slice(&block[..length]);
                break;
            }
        }
        Ok(self.length() as usize)
    }
}

// NOTE: `Seek` trait is not required for this lab
impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<usize> {
        unimplemented!()
    }
}

// NOTE: `Write` trait{} is not required for this lab
impl Write for File {
    fn write(&mut self, _buf: &[u8]) -> Result<usize> {
        unimplemented!()
    }

    fn flush(&mut self) -> Result<()> {
        unimplemented!()
    }
}
