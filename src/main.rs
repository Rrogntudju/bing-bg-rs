// Téléchargement de l'image Bing du jour et installation comme arrière-plan dans Cosmic
use {
    cosmic_bg_config::{self, Source},
    reqwest::Client,
    serde_json::value::Value,
    std::{env, error::Error, fs, io::Write, path::Path, time::Duration},
    rand::distr::Alphanumeric,
    rand::Rng
};

const URL_DESC: &str = "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-US";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Téléchargement du descriptif de l'image
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    let response = client.get(URL_DESC).send().await?.text().await?;
    let desc: Value = serde_json::from_str(&response)?;
    let url = if let Some(url) = desc["images"][0]["url"].as_str() {
        url
    } else {
        return Err(format!("La propriété «url» est absente du descriptif JSON. Vérifier dans {}", URL_DESC).into());
    };
    let url_img = "https://www.bing.com".to_owned() + url;
    let download_task = tokio::spawn(async move { client.get(&url_img).send().await?.bytes().await });

    // Téléchargement de l'image JPEG
    let home = if let Some(home) = env::vars().find(|v| v.0 == "HOME").map(|v| v.1) {
        home
    } else {
        return Err(format!("La variable d'environment HOME n'est pas configurée").into());
    };
    let bg_path = Path::new(&home).join(".bingbg");
    if !bg_path.exists() {
        fs::create_dir(&bg_path)?;
    }
    let img = download_task.await??.to_vec();
    let random_name: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let img_name = [&random_name, ".jpg"].concat();
    let bg_path = bg_path.join(img_name);
    let mut bg = fs::File::create(&bg_path)?;
    bg.write_all(&img)?;

    // Configurer l'image comme arrière-plan
    let context = cosmic_bg_config::context()?;
    let mut background = context.default_background();
    background.source = Source::Path(bg_path);
    let mut config = cosmic_bg_config::Config::load(&context)?;
    config.set_entry(&context, background)?;

    // Supprimer  
    Ok(())
}
