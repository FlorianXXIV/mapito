use core::panic;

use reqwest::{blocking::Client, Result};
use sha2::{Digest, Sha512};

pub trait Downloader {
    fn download_file(&self, path: &str, url: &str, hash: &str) -> Result<()>;
}
impl Downloader for Client {
    fn download_file(&self, path: &str, url: &str, hash: &str) -> Result<()> {
        let body = self.get(url).send().unwrap().bytes().unwrap();

        println!("Checking data integrity.");

        let dl_hash = Sha512::digest(&body);

        let hx_hash = base16ct::lower::encode_string(&dl_hash);
        if hx_hash == hash {
            println!("Integrity check passed.");
            let _ = std::fs::write(path, &body).unwrap();
        } else {
            panic!("Downloaded Data does not match provided Hash!")
        }
        Ok(())
    }
}
