// Téléchargement de l'image Bing du jour et installation comme arrière-plan dans Windows
use {
    image::{load_from_memory_with_format, ImageFormat},
    minreq,
    serde_json::value::Value,
    std::error::Error,
    std::{env, fmt},
    winapi::ctypes::c_void,
    winapi::um::winuser::{
        SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER,
    },
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
    let desc: Value = minreq::get(URL_DESC).with_timeout(10).send()?.json()?;
    let url = desc["images"][0]["url"].as_str();
    if url.is_none() {
        return Err(BingError(format!(
            "La propriété «url» est absente du descriptif JSON. Vérifier dans {}",
            URL_DESC
        ))
        .into());
    }
    let url_img = "https://www.bing.com".to_owned() + url.unwrap();

    println!("Téléchargement de l'image JPEG...");
    let resp = minreq::get(&url_img).with_timeout(10).send()?;

    println!("Transformer en BMP et sauvegarder...");
    let img = load_from_memory_with_format(resp.as_bytes(), ImageFormat::JPEG)?;
    let mut bmp_path = env::temp_dir();
    bmp_path.push("bingbg");
    bmp_path.set_extension("bmp");
    img.save(&bmp_path)?;

    println!("Configurer l'image comme arrière-plan...");
    //convertir la chaîne utf-8 en chaîne utf-16 terminée par null
    let mut path: Vec<u16> = bmp_path.to_str().unwrap().encode_utf16().collect();
    path.push(0);
    let path_ptr = path.as_mut_ptr() as *mut c_void;

    let rc = unsafe {
        SystemParametersInfoW(
            SPI_SETDESKWALLPAPER,
            0,
            path_ptr,
            SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
        )
    };
    if rc == 0 {
        return Err(match std::io::Error::last_os_error().raw_os_error() {
            Some(e) => BingError(format!(
                "SystemParametersInfoW a retourné le code d'erreur {}",
                e
            ))
            .into(),
            None => BingError("Oups!".into()).into(),
        });
    }

    println!("Terminé!");
    Ok(())
}
