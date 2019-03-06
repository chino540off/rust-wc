use std::io::prelude::*;
use std::io::SeekFrom;
use std::fs::File;

pub struct WordStream {
    file: File,
    buffer: Vec<u8>,
    bufoffset: usize,
    bufsize: usize,
    read: usize,
    size: usize,
    separators: Vec<u8>,
}

impl WordStream {
    pub fn new(
        filename: String,
        bufsize: usize,
        offset: u64,
        size: usize,
        separators: &[u8],
    ) -> WordStream {
        let _offset = if offset == 0 { 0 } else { offset - 1 };

        let mut file = match File::open(filename) {
            Ok(file) => file,
            Err(msg) => {
                panic!("open error: {}", msg);
            }
        };

        //println!("start at offset {}", offset);
        match file.seek(SeekFrom::Start(_offset)) {
            Ok(_) => {}
            Err(msg) => panic!("seek error: {}", msg),
        }

        let mut s = WordStream {
            file: file,
            buffer: vec![0; bufsize],
            bufoffset: 0,
            bufsize: 0,
            read: 0,
            size: size,
            separators: separators.to_vec(),
        };

        if offset != 0 {
            {
                // offset - 1 => starting a new word ?
                let c = s.getc();
                if s.is_separator(c) {
                    //println!("Starting word");
                    return s;
                }
            }

            // skiping middled word
            s.skip_word();
        }

        s
    }

    fn skip_word(&mut self) {
        // skip next "word"
        loop {
            let c = self.getc();
            self.read += 1;

            if !self.is_word(c) {
                break;
            }
        }
    }

    fn skip_separator(&mut self) -> Option<u8> {
        // Walk throw next word
        loop {
            let c = self.getc();

            if c.is_none() {
                return None;
            }

            self.read += 1;

            if !self.is_separator(c) {
                return c;
            }
        }
    }

    fn is_separator(&mut self, c: Option<u8>) -> bool {
        match c {
            Some(c) => self.separators.iter().find(|&&x| c == x).is_some(),
            None => true,
        }
    }

    fn is_word(&mut self, c: Option<u8>) -> bool {
        !self.is_separator(c)
    }

    fn _read(&mut self) -> usize {
        match self.file.read(&mut self.buffer[..]) {
            Ok(size) => {
                //println!("read {} bytes", size);
                self.bufoffset = 0;
                self.bufsize = size;
                size
            }
            Err(msg) => panic!("read error: {}", msg),
        }
    }

    pub fn getc(&mut self) -> Option<u8> {
        let offset = if self.bufoffset == self.bufsize {
            if self._read() == 0 {
                return None;
            }

            0
        } else {
            self.bufoffset
        };

        self.bufoffset += 1;

        //println!("getc -> {}", self.buffer[offset]);
        Some(self.buffer[offset])
    }

    pub fn word(&mut self) -> Option<String> {
        //println!("skip_separator");
        let c = self.skip_separator();

        if self.read > self.size || c.is_none() {
            return None;
        }

        //println!("start new word");
        let mut word = Vec::new();

        word.push(c.unwrap());

        loop {
            let c = self.getc();
            self.read += 1;

            if !self.is_word(c) {
                break;
            }

            word.push(c.unwrap());
        }

        //println!("next word: {}", String::from_utf8(word.clone()).unwrap());

        match String::from_utf8(word) {
            Ok(s) => Some(s),
            Err(msg) => {
                println!("from_utf8 error: {}", msg);
                None
            }
        }
    }
}

impl Iterator for WordStream {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.word()
    }
}

#[cfg(test)]
mod tests {
    use super::WordStream;

    #[test]
    fn test_getc_beyond_buffer() {
        let filename = String::from("tests/test.txt");

        let mut s = WordStream::new(filename, 2, 0, 2 as usize, " ".as_bytes());

        assert_eq!(s.getc().is_some(), true);
        assert_eq!(s.getc().is_some(), true);
        assert_eq!(s.getc().is_some(), true);
    }

    #[test]
    fn test_getc_eof() {
        let filename = String::from("tests/test.txt");

        let mut s = WordStream::new(filename, 2, 8, 2 as usize, " ".as_bytes());

        assert_eq!(s.getc().is_some(), true);
        assert_eq!(s.getc().is_some(), true);
        assert_eq!(s.getc().is_some(), true);
        assert_eq!(s.getc().is_some(), true);
        assert_eq!(s.getc().is_some(), false);
    }

    //#[test]
    //fn test_getc_fuzzing() {
    //    let mut file = File::open("tests/test.txt").unwrap();

    //    let mut s = WordStream::new(&mut file, 2, 10, 2 as usize, " ".as_bytes());

    //    assert_eq!(s.getc().is_some(), true);
    //    assert_eq!(s.getc().is_some(), true);
    //    assert_eq!(s.getc().is_some(), true);
    //    assert_eq!(s.getc().is_none(), true);
    //}

    #[test]
    fn test_word() {
        {
            let filename = String::from("tests/test.txt");

            let mut s = WordStream::new(filename, 1024, 0, 2 as usize, " ".as_bytes());

            assert_eq!(s.word().is_some(), true);
            assert_eq!(s.word().is_some(), false);
        }

        {
            let filename = String::from("tests/test.txt");

            let mut s = WordStream::new(filename, 1024, 0, 6 as usize, " ".as_bytes());

            assert_eq!(s.word().is_some(), true);
            assert_eq!(s.word().is_some(), true);
            assert_eq!(s.word().is_some(), false);
        }

        {
            let filename = String::from("tests/test.txt");

            let mut s = WordStream::new(filename, 1024, 0, 10 as usize, " ".as_bytes());

            assert_eq!(s.word().is_some(), true);
            assert_eq!(s.word().is_some(), true);
            assert_eq!(s.word().is_some(), true);
            assert_eq!(s.word().is_some(), false);
        }

        {
            let filename = String::from("tests/test.txt");

            let mut s = WordStream::new(filename, 1024, 1, 2 as usize, " ".as_bytes());

            assert_eq!(s.word().is_some(), false);
        }

        {
            let filename = String::from("tests/test.txt");

            let mut s = WordStream::new(filename, 1024, 1, 10 as usize, " ".as_bytes());

            assert_eq!(s.word().is_some(), true);
            assert_eq!(s.word().is_some(), true);
            assert_eq!(s.word().is_some(), false);
        }

        {
            let filename = String::from("tests/test.txt");

            let mut s = WordStream::new(filename, 1024, 3, 10 as usize, " ".as_bytes());

            assert_eq!(s.word().is_some(), true);
            assert_eq!(s.word().is_some(), true);
            assert_eq!(s.word().is_some(), false);
        }
    }
}
