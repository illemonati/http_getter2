extern crate argparse;
extern crate reqwest;

use argparse::{ArgumentParser, StoreOption, StoreTrue};
use std::fs::File;
use std::io::{self, Read, Write, copy};
use regex::Regex;
use indicatif::{ProgressBar, ProgressStyle};

fn main() {
    let mut url = None;
    let mut download = false;
    set_up_arguments(&mut url, &mut download);
    get_url(&url, download);
}

fn set_up_arguments(url: &mut Option<String>, download: &mut bool) {
    let mut ap = ArgumentParser::new();
    ap.set_description("get things from web");
    ap.refer(url)
        .add_option(&["-u", "--url"], StoreOption, "url for thing to get");
    ap.refer(download)
        .add_option(&["-d", "--download"], StoreTrue, "use thing if download");
    ap.parse_args_or_exit();
}

fn get_url(url: &Option<String>, download: bool) {
    if url == &None {
        println!{"please consult --help for entering a url"};
        return;
    }
    println!("[*] Connecting to {}", url.as_ref().unwrap());
    let mut res = reqwest::get(url.as_ref().unwrap().as_str()).expect("[!] Error connecting to url");
    if !download {
        println!("{}", &res.text().expect("[!] Error converting response body to text, try downloading !"));
    } else {
        let name = get_file_name(url.as_ref().unwrap());
        download_file(name, &mut res);
    }
}

fn get_file_name(url: &String) -> String {
    let re = Regex::new(r"(/.+/)(?P<filename>.+)").expect("[!] regex creation error");
    let caps = re.captures(url.as_str()).expect("[!] unable to parse file name");
    caps["filename"].to_string()
}

struct DownloadProgress<R> {
    inner: R,
    progress_bar: ProgressBar,
}

impl<R: Read> Read for DownloadProgress<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf).map(|n| {
            self.progress_bar.inc(n as u64);
            n
        })
    }
}

fn download_file(name: String, res: &mut reqwest::Response) {
    println!("[*] downloading {}", &name);
    let len = res.content_length();
    if len != None {
        let mut pb = ProgressBar::new(len.unwrap());
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .progress_chars("#>-"));
        let mut file = File::create(&name).expect("file creation error");
        let mut source = DownloadProgress {
            progress_bar: pb,
            inner: res,
        };
        copy(&mut source, &mut file);
    }
}
