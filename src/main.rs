// Téléchargement de l'image Bing du jour et installation comme arrière-plan dans Cosmic
use {
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
    let task = tokio::spawn(async move { client.get(&url_img).send().await?.bytes().await });

    println!("Téléchargement de l'image JPEG...");
    let mut bmp_path = env::temp_dir();
    bmp_path.push("bingbg");
    bmp_path.set_extension("bmp");
    let bmp = task.await??.to_vec();

    println!("Transformer en BMP et sauvegarder...");
    let img = load_from_memory_with_format(&bmp, ImageFormat::Jpeg)?;
    img.save(&bmp_path)?;

    println!("Configurer l'image comme arrière-plan...");


    println!("Terminé!");
    Ok(())
}
