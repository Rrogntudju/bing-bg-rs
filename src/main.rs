// Téléchargement de l'image Bing du jour et installation comme arrière-plan dans Cosmic
use {
    cosmic_bg_config,
    image::{load_from_memory_with_format, ImageFormat},
    reqwest::Client,
    serde_json::value::Value,
    std::{
        env, error::Error, fs::create_dir, path::Path, time::Duration
    },
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
    if let Some(home) = env::vars().find(|v| v.0 == "HOME").map(|v| v.1) {
        let bg_path = Path::new(&home).join(".Bingbg");
        if !bg_path.exists() {
            create_dir(&bg_path)?;
        }
        let img = task.await??.to_vec();
        let img = load_from_memory_with_format(&img, ImageFormat::Jpeg)?;
        let bg_path = bg_path.with_file_name("bingbg.jpg");
        img.save(&bg_path)?;

        println!("Configurer l'image comme arrière-plan...");
    } else {
        return Err(format!("La variable d'environment HOME n'est pas configurée").into());
    }

    println!("Terminé!");
    Ok(())
}
