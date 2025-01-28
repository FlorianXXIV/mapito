use reqwest::{blocking::Client, Result};
pub trait Downloader {
    fn download_file(&self, path: &str, url: &str) -> Result<()>;
}
impl Downloader for Client {
    fn download_file(&self, path: &str, url: &str) -> Result<()> {
        let body = self.get(url).send().unwrap().bytes().unwrap();
        let _ = std::fs::write(path, &body).unwrap();
        Ok(())
    }
}
