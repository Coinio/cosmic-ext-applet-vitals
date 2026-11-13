//! Reads the small `/proc` and `/sys` files the monitors poll every tick.
//!
//! `fs::read_to_string` suits them poorly. The kernel reports these files as empty, or as a page, so it can't size its
//! buffer up front: it asks for the file's size, then reads in small chunks that it doubles, e.g. 9 reads for
//! `/proc/stat`. And the buffer is freed after every read, which hands whole pages back to the allocator, that libcosmic
//! then returns to the kernel after every update, so they fault back in on the next tick. Instead each thread keeps
//! one buffer that's big enough for any of the files, which takes in a whole file in a single read.

use std::cell::RefCell;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

/// Room for the largest file polled on most machines, `/proc/self/mounts` or `/proc/stat`. It grows for larger ones.
const INITIAL_CAPACITY: usize = 16 * 1024;

thread_local! {
    static BUFFER: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Read the file at `path`, and hand its contents to `parse`.
pub fn read<T>(path: impl AsRef<Path>, parse: impl FnOnce(&str) -> T) -> io::Result<T> {
    let mut file = File::open(path)?;
    BUFFER.with(|buffer| match buffer.try_borrow_mut() {
        Ok(mut buffer) => read_into(&mut file, &mut buffer, parse),
        // Only if `parse` reads a file itself, which then can't use the buffer that holds what it's parsing.
        Err(_) => read_into(&mut file, &mut Vec::new(), parse),
    })
}

/// Read the file at `path`, trimmed of the newline the kernel ends values with, and parse it as a `T`.
pub fn read_value<T: std::str::FromStr>(path: impl AsRef<Path>) -> Option<T> {
    read(path, |contents| contents.trim().parse().ok()).ok().flatten()
}

fn read_into<T>(file: &mut File, buffer: &mut Vec<u8>, parse: impl FnOnce(&str) -> T) -> io::Result<T> {
    if buffer.is_empty() {
        buffer.resize(INITIAL_CAPACITY, 0);
    }

    let mut filled = 0;
    loop {
        if filled == buffer.len() {
            buffer.resize(buffer.len() * 2, 0);
        }
        match file.read(&mut buffer[filled..]) {
            Ok(0) => break,
            Ok(count) => filled += count,
            Err(err) if err.kind() == io::ErrorKind::Interrupted => {}
            Err(err) => return Err(err),
        }
    }

    let contents =
        std::str::from_utf8(&buffer[..filled]).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    Ok(parse(contents))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn reads_a_file_larger_than_the_buffer() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("large");
        let contents: String = (0..INITIAL_CAPACITY).map(|index| format!("{index}\n")).collect();
        fs::write(&path, &contents).unwrap();

        assert_eq!(read(&path, str::to_string).unwrap(), contents);
        // The buffer is reused, and nothing is left over from the larger file.
        fs::write(&path, "small\n").unwrap();
        assert_eq!(read(&path, str::to_string).unwrap(), "small\n");
    }

    #[test]
    fn reads_a_value() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("value");
        fs::write(&path, "1953525168\n").unwrap();

        assert_eq!(read_value::<u64>(&path), Some(1953525168));
        assert_eq!(read_value::<u64>(dir.path().join("missing")), None);
    }

    #[test]
    fn parsing_can_read_another_file() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("outer"), "outer").unwrap();
        fs::write(dir.path().join("inner"), "inner").unwrap();

        let both = read(dir.path().join("outer"), |outer| {
            format!("{outer} {}", read(dir.path().join("inner"), str::to_string).unwrap())
        });
        assert_eq!(both.unwrap(), "outer inner");
    }

    #[test]
    fn reads_proc_files_whole() {
        let stat = read("/proc/stat", str::to_string).unwrap();
        assert!(stat.starts_with("cpu "));
        assert!(stat.ends_with('\n'));
    }
}
