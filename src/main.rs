extern crate getopts;

mod word_stream;
mod word_count;

use std::collections::BTreeMap;
use std::env;
use getopts::Options;

use word_count::WordCount;

fn opt_value<T>(opt: Option<String>, default: T) -> Result<T, String>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match opt {
        Some(s) => match s.parse::<T>() {
            Ok(v) => Ok(v),
            Err(msg) => Err(format!("{}", msg)),
        },
        None => Ok(default),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();
    let separators = " \t\n\r";

    opts.optopt("t", "threads", "set number of threads", "thread");
    opts.optopt("b", "bs", "set read buffer size", "bufsize");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    let nthreads = match opt_value::<u64>(matches.opt_str("t"), 10) {
        Ok(v) => v,
        Err(msg) => {
            println!("-t option error: {}", msg);
            std::process::exit(1);
        }
    };

    let bufsize = match opt_value::<usize>(matches.opt_str("b"), 1024) {
        Ok(v) => v,
        Err(msg) => {
            println!("-b option error: {}", msg);
            std::process::exit(1);
        }
    };

    let mut result: BTreeMap<String, u64> = BTreeMap::new();

    for opt in matches.free {
        let filename = opt.clone();

        let wc = WordCount::new(&filename, &String::from(separators), nthreads, bufsize);

        match wc.process() {
            Ok(res) => for (w, c) in res {
                let count = result.entry(w).or_insert(0);

                *count += c;
            },
            Err(msg) => println!("WordCount error for {}: {}", filename, msg),
        }
    }

    for (w, c) in result {
        println!("{} -> {}", w, c);
    }
}
