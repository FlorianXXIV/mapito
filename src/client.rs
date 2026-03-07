use core::panic;

use reqwest::blocking::Client;
use sha2::{Digest, Sha512};

use crate::util::error::ApiError;

pub trait Downloader {
    fn download_file(&self, path: &str, url: &str, hash: &str) -> Result<(), ApiError>;
}
impl Downloader for Client {
    fn download_file(&self, path: &str, url: &str, hash: &str) -> Result<(), ApiError> {
        let body = match self.get(url).send() {
            Ok(resp) => resp,
            Err(e) => {
                panic!("Failed to send request: {e}");
            }
        }
        .bytes()
        .unwrap();

        println!("Checking data integrity.");

        let dl_hash = Sha512::digest(&body);

        let hx_hash = base16ct::lower::encode_string(&dl_hash);
        if hx_hash == hash {
            println!("Integrity check passed.");
            std::fs::write(path, &body).unwrap();
        } else {
            println!("Integrity check failed.");
            return Err(ApiError::not_found());
        }
        Ok(())
    }
}
