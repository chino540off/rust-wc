use std::collections::BTreeMap;
use std::fs::metadata;
use std::thread;
use std::thread::JoinHandle;

struct ChunkGenerator {
    len: u64,
    offset: u64,
    size: u64,
}

impl ChunkGenerator {
    fn new(len: u64, n: u64) -> ChunkGenerator {
        let c = len / n;
        let size = if c * n < len { c + 1 } else { c };

        ChunkGenerator {
            len: len,
            offset: 0,
            size: size,
        }
    }
}

impl Iterator for ChunkGenerator {
    type Item = (u64, u64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            return None;
        } else if self.len > self.size {
            let ret = (self.offset, self.size);

            self.offset += self.size;
            self.len -= self.size;

            return Some(ret);
        } else {
            let ret = (self.offset, self.len);

            self.offset += self.len;
            self.len -= self.len;

            return Some(ret);
        };
    }
}

use word_stream::WordStream;

pub struct WordCount {
    filename: String,
    separators: String,
    nthreads: u64,
    bufsize: usize,
}

impl WordCount {
    pub fn new(filename: &String, separators: &String, nthreads: u64, bufsize: usize) -> WordCount {
        WordCount {
            filename: filename.clone(),
            separators: separators.clone(),
            nthreads: nthreads,
            bufsize: bufsize,
        }
    }

    pub fn process(&self) -> Result<BTreeMap<String, u64>, String> {
        let metadata = match metadata(&self.filename) {
            Ok(metadata) => metadata,
            Err(msg) => {
                return Err(format!("metadata error: {}", msg));
            }
        };

        let mut threads: Vec<JoinHandle<BTreeMap<String, u64>>> = Vec::new();

        println!("Starting to read {}", self.filename);

        for (offset, len) in ChunkGenerator::new(metadata.len(), self.nthreads) {
            let s = WordStream::new(
                self.filename.clone(),
                self.bufsize,
                offset,
                len as usize,
                self.separators.as_bytes().clone(),
            );

            threads.push(thread::spawn(move || {
                let mut m = BTreeMap::new();

                for w in s {
                    let count = m.entry(w).or_insert(0);

                    *count += 1;
                }
                m
            }));
        }

        let mut result: BTreeMap<String, u64> = BTreeMap::new();

        for t in threads {
            match t.join() {
                Ok(m) => for (w, c) in m {
                    let count = result.entry(w).or_insert(0);

                    *count += c;
                },
                Err(_) => panic!("join error"),
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::WordCount;

    use std::collections::BTreeMap;

    #[test]
    fn test_wc_test01() {
        let filename = String::from("tests/test.txt");
        let sep = String::from(" \t\r\n");

        let wc = WordCount::new(&filename, &sep, 1, 1024);
        let m = wc.process().unwrap_or(BTreeMap::new());

        for i in vec![2, 3, 4, 5, 6, 7, 8, 9, 10] {
            let wc_test = WordCount::new(&filename, &sep, i, 1024);
            let m_test = wc_test.process().unwrap_or(BTreeMap::new());

            println!("word count for {} threads", i);
            assert_eq!(m, m_test);
        }
    }
}
