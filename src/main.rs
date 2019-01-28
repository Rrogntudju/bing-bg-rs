// Téléchargement de l'image Bing du jour et installation comme arrière-plan dans Windows
use serde_json::value::Value;
use reqwest::Client;
use std::error::Error;
use std::{fmt, env};
use image::{ImageFormat, load_from_memory_with_format};
use core::ffi::c_void;
use winapi::um::winuser::{SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_UPDATEINIFILE, SPIF_SENDCHANGE};

const URL_DESC: &str = "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-US";

#[derive(Debug)]
pub enum BingError {
    InvalidImageUrl,
    WinError(String),
}

impl fmt::Display for BingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BingError::InvalidImageUrl => write!(f, 
                "La propriété «url» n'est pas dans le descriptif JSON. \nVérifier dans {}",
                URL_DESC),
            BingError::WinError(e) => write!(f, "{}", e),
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
    let mut buf: Vec<u8> = vec![]; // on ne peut pas utiliser image::load avec le reader de Response parce qu'il n'implémente pas BufRead + Seek
    resp.copy_to(&mut buf)?;
    let img = load_from_memory_with_format(&buf, ImageFormat::JPEG)?;
    let mut bmp_path = env::temp_dir();
    bmp_path.push("bingbg");
    bmp_path.set_extension("bmp");
    img.save(&bmp_path)?;
 
    println!("Configurer l'image comme arrière-plan...");
    //convertir la chaîne utf-8 en chaîne utf-16 terminée par null
    let mut path: Vec<u16> = bmp_path.to_str().unwrap().encode_utf16().collect();
    path.push(0); 
    let path_ptr: *mut c_void = path.as_mut_ptr() as *mut c_void;
    
    let rc = unsafe { SystemParametersInfoW(SPI_SETDESKWALLPAPER, 0, path_ptr , SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) };
    if rc == 0 {
        return 
            match std::io::Error::last_os_error().raw_os_error() {
                Some(e) => Err(From::from(BingError::WinError(format!("SystemParametersInfoW a retourné le code d'erreur {}", e)))),
                None    => Err(From::from(BingError::WinError(format!("Oups!")))),
            }
    }
   
    println!("Terminé!");
    Ok(())
}