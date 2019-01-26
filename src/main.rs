// Téléchargement de l'image Bing du jour et installation comme arrière-plan
use serde_json::value::Value;
use reqwest::Client;
use std::error::Error;
use std::fmt;
use image::{ImageFormat, load_from_memory_with_format};

const URL_DESC: &str = "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-US";

#[derive(Debug)]
pub enum BingError {
    InvalidImageUrl,
}

impl fmt::Display for BingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BingError::InvalidImageUrl => write!(f, 
                "La propriété «url» n'est pas dans le descriptif JSON. \nVérifier dans {}",
                URL_DESC),
        }
    }
}

impl Error for BingError {}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Téléchargement du descriptif de l'image...");
    let client = Client::builder().build()?;
    let desc: Value = client.get(URL_DESC).send()?.json()?;
    let url = desc["images"][0]["url"].as_str();
    if url.is_none() {
        return Err(From::from(BingError::InvalidImageUrl));
    }
    let url_img = "https://www.bing.com".to_owned() + url.unwrap();

    println!("Téléchargement de l'image JPEG...");
    let mut resp = client.get(&url_img).send()?;

    println!("Transformer en BMP et sauvegarder...");
    let mut buf: Vec<u8> = vec![];
    resp.copy_to(&mut buf)?;
    let img = load_from_memory_with_format(&buf, ImageFormat::JPEG)?;
    
 

    Ok(())
}
