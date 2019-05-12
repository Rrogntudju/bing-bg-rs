// Téléchargement de l'image Bing du jour et installation comme arrière-plan dans Windows
use {
    serde_json::value::Value,
    reqwest::Client,
    std::error::Error,
    std::{fmt, env},
    image::{ImageFormat, load_from_memory_with_format},
    core::ffi::c_void,
    winapi::um::winuser::{SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_UPDATEINIFILE, SPIF_SENDCHANGE},
};

const URL_DESC: &str = "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-US";

#[derive(Debug)]
struct BingError(String);

impl fmt::Display for BingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
         write!(f, "{}", self.0)
    }
}

impl Error for BingError {}

fn main() -> Result<(), Box<dyn Error>> {
    println!("Téléchargement du descriptif de l'image...");
    let client = Client::builder().build()?;
    let desc: Value = client.get(URL_DESC).send()?.json()?;
    let url = desc["images"][0]["url"].as_str();
    if url.is_none() {
        return Err(BingError(format!("La propriété «url» est absente du descriptif JSON. Vérifier dans {}", URL_DESC)).into());
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
    let path_ptr = path.as_mut_ptr() as *mut c_void;
    
    let rc = unsafe { SystemParametersInfoW(SPI_SETDESKWALLPAPER, 0, path_ptr , SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) };
    if rc == 0 {
        return 
            match std::io::Error::last_os_error().raw_os_error() {
                Some(e) => Err(BingError(format!("SystemParametersInfoW a retourné le code d'erreur {}", e)).into()),
                None    => Err(BingError("Oups!".into()).into()),
            }
    }
   
    println!("Terminé!");
    Ok(())
}
