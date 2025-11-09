// Téléchargement de l'image Bing du jour et installation comme arrière-plan dans Cosmic
use {
    cosmic_bg_config::{self, Source},
    log::{LevelFilter, Log, error, info},
    reqwest::Client,
    serde_json::value::Value,
    std::{
        env,
        error::Error,
        fs,
        io::Write,
        path::{Path, PathBuf},
        time::Duration,
    },
    systemd_journal_logger::{JournalLog, connected_to_journal},
};

const URL_DESC: &str = "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=en-US";

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let _ = writeln!(std::io::stderr(), "{}", record.args());
    }

    fn flush(&self) {
        let _ = std::io::stderr().flush();
    }
}

async fn set_bing_background() -> Result<(), Box<dyn Error>> {
    // Téléchargement du descriptif de l'image
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    let response = client.get(URL_DESC).send().await?.text().await?;
    let desc: Value = serde_json::from_str(&response)?;
    let url_img = if let Some(url) = desc["images"][0]["url"].as_str() {
        "https://www.bing.com".to_owned() + url
    } else {
        return Err(format!("La propriété «url» est absente du descriptif JSON. Vérifier dans {URL_DESC}").into());
    };

    // Téléchargement de l'image JPEG
    let download_task = tokio::spawn(async move { client.get(&url_img).send().await?.bytes().await });
    let bg_path = if let Some(home) = env::vars().find(|v| v.0 == "HOME").map(|v| v.1) {
        Path::new(&home).join(".local/share/bingbg")
    } else {
        return Err("La variable d'environment HOME n'est pas configurée".into());
    };
    if !bg_path.exists() {
        fs::create_dir(&bg_path)?;
    }
    let bg_file = if let Some(titre) = desc["images"][0]["title"].as_str() {
        bg_path.join(titre.to_owned() + ".jpg")
    } else {
        return Err(format!("La propriété «title» est absente du descriptif JSON. Vérifier dans {URL_DESC}").into());
    };
    let mut bg = fs::File::create(&bg_file)?;
    let img = download_task.await??.to_vec();
    bg.write_all(&img)?;

    // Configurer l'image comme arrière-plan
    let context = cosmic_bg_config::context()?;
    let mut background = context.default_background();
    background.source = Source::Path(bg_file);
    let mut config = cosmic_bg_config::Config::load(&context)?;
    config.set_entry(&context, background)?;

    //Ménage
    let mut bg_entries: Vec<(Duration, PathBuf)> = Vec::new();
    for entry in fs::read_dir(bg_path)? {
        let entry = entry?;
        bg_entries.push((entry.metadata()?.created()?.elapsed()?, entry.path()));
    }
    bg_entries.sort_by_key(|(d, _)| *d);
    for (_, path) in bg_entries.iter().skip(20) {
        fs::remove_file(path)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Configurer le log
    if connected_to_journal() {
        JournalLog::new().unwrap().install().unwrap();
    } else {
        log::set_logger(&SimpleLogger).unwrap();
    }
    log::set_max_level(LevelFilter::Info);

    set_bing_background().await.unwrap_or_else(|e| error!("{e}"));
    info!("Fait");
}
