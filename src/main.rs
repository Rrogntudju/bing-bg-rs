// Téléchargement de l'image Bing du jour et installation comme arrière-plan
use serde_json::value::Value;
use reqwest::Client;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum BingError {
    InvalidImageUrl
}

impl fmt::Display for BingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BingError::InvalidImageUrl => write!(f, 
                "La propriété «url» n'est pas dans le descriptif JSON. 
                Vérifier dans https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-US"),
        }
    }
}

impl Error for BingError {}

fn main() -> Result<(), Box<Error>> {
    println!("Téléchargement du descriptif de l'image...");
    let client = Client::builder().build()?;
    let desc: Value = client.get("https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-US").send()?.json()?;
    let url = desc["images"][0]["url"].as_str();
    if url.is_some() {
        let url_img = "https://www.bing.com".to_owned() + url.unwrap();
    }
    else {
        return Err(Box::new(BingError::InvalidImageUrl));
    }
    


    Ok(())
}
