// Téléchargement de l'image Bing du jour et installation comme arrière-plan dans Windows
use windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SPI_SETDESKWALLPAPER};
use {
    core::ffi::c_void,
    image::{load_from_memory_with_format, ImageFormat},
    reqwest::Client,
    serde_json::value::Value,
    std::env,
    std::error::Error,
    std::time::Duration,
};

const URL_DESC: &str = "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-US";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Téléchargement du descriptif de l'image...");
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    let response = client.get(URL_DESC).send().await?.text().await?;
    let desc: Value = serde_json::from_str(&response)?;
    let url = desc["images"][0]["url"].as_str();
    if url.is_none() {
        return Err(format!("La propriété «url» est absente du descriptif JSON. Vérifier dans {}", URL_DESC).into());
    }
    let url_img = "https://www.bing.com".to_owned() + url.unwrap();

    println!("Téléchargement de l'image JPEG...");
    let handle = tokio::spawn(async move {
        let response = client.get(&url_img).send().await?;
        response.bytes().await
    });
    let mut bmp_path = env::temp_dir();
    bmp_path.push("bingbg");
    bmp_path.set_extension("bmp");

    println!("Transformer en BMP et sauvegarder...");
    let bmp = handle.await??.to_vec();
    let img = load_from_memory_with_format(&bmp, ImageFormat::Jpeg)?;
    img.save(&bmp_path)?;

    println!("Configurer l'image comme arrière-plan...");
    //convertir la chaîne utf-8 en chaîne utf-16 terminée par null
    let mut path: Vec<u16> = bmp_path.to_str().unwrap().encode_utf16().collect();
    path.push(0);
    let path_ptr = path.as_mut_ptr() as *mut c_void;

    let rc = unsafe { SystemParametersInfoW(SPI_SETDESKWALLPAPER, 0, path_ptr, SPIF_UPDATEINIFILE | SPIF_SENDCHANGE) };
    if !rc.as_bool() {
        return Err(match std::io::Error::last_os_error().raw_os_error() {
            Some(e) => format!("SystemParametersInfoW a retourné le code d'erreur {}", e).into(),
            None => "Oups!".into(),
        });
    }

    println!("Terminé!");
    Ok(())
}
