// Téléchargement de l'image Bing du jour et installation comme arrière-plan dans Cosmic
use {
    anyhow::{Error, anyhow},
    cosmic_bg_config::{Mode, Source},
    cosmic_config::CosmicConfigEntry,
    cosmic_theme::{Theme, palette::Srgba},
    log::{LevelFilter, Log},
    reqwest::Client,
    serde_json::value::Value,
    std::{
        env, fs,
        io::{Cursor, Write, stderr},
        path::{Path, PathBuf},
        time::Duration,
    },
    systemd_journal_logger::{JournalLog, connected_to_journal},
};

const URL_DESC: &str = "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=1&mkt=fr-CA";

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let _ = writeln!(stderr(), "{}", record.args());
    }

    fn flush(&self) {
        let _ = stderr().flush();
    }
}

async fn set_bing_background() -> Result<(), Error> {
    // Téléchargement du descriptif de l'image
    let client = Client::builder().timeout(Duration::from_secs(10)).build()?;
    let response = client.get(URL_DESC).send().await?.text().await?;
    let desc: Value = serde_json::from_str(&response)?;
    let url_img = desc["images"][0]["url"]
        .as_str()
        .and_then(|url| Some("https://www.bing.com".to_owned() + &url.replacen("_1920x1080", "_UHD", 2)))
        .ok_or(anyhow!("La propriété «url» est absente du descriptif JSON. Vérifier dans {URL_DESC}"))?;

    // Téléchargement de l'image JPEG
    let download_task = tokio::spawn(async move { client.get(&url_img).send().await?.bytes().await });

    let bg_path = env::vars()
        .find(|v| v.0 == "HOME")
        .map(|v| v.1)
        .and_then(|home| Some(Path::new(&home).join("Images/bingbg")))
        .ok_or(anyhow!("La variable d'environment HOME n'est pas configurée"))?;
    if !bg_path.exists() {
        fs::create_dir(&bg_path)?;
    }

    let bg_file = desc["images"][0]["title"]
        .as_str()
        .and_then(|titre| Some(bg_path.join(titre.to_owned() + ".jpg")))
        .ok_or(anyhow!("La propriété «title» est absente du descriptif JSON. Vérifier dans {URL_DESC}"))?;
    let mut bg = fs::File::create(&bg_file)?;

    let img = download_task.await??.to_vec();
    bg.write_all(&img)?;

    // Pour chaque mode, configurer l'image comme arrière-plan
    for mode in Mode::ALL {
        let context = mode.context()?;
        let mut background = context.default_background();
        background.source = Source::Path(bg_file.clone());
        let mut config = cosmic_bg_config::Config::load(&context)?;
        config.set_entry(&context, background)?;
    }

    // Obtenir la couleur dominante de l'image
    let bg = image::ImageReader::new(Cursor::new(&img)).with_guessed_format()?.decode()?.to_rgb8();
    let input = okmain::InputImage::try_from(&bg)?;
    let output = okmain::colors(input)[0];
    let bg_dominant_color = Srgba::new(f32::from(output.r) / 255.0, f32::from(output.g) / 255.0, f32::from(output.b) / 255.0, 1.0);

    // Reconstruire le thème clair et le thème sombre avec la couleur dominante du fond d'écran comme accent
    let dark_config = Theme::dark_config()?;
    let light_config = Theme::light_config()?;

    let dark_theme = match Theme::get_entry(&dark_config) {
        Ok(entry) => entry,
        Err(err) => return Err(anyhow!(err.0[0].to_string())),
    };
    let light_theme = match Theme::get_entry(&light_config) {
        Ok(entry) => entry,
        Err(err) => return Err(anyhow!(err.0[0].to_string())),
    };

    let dark_theme = dark_theme.with_accent(bg_dominant_color);
    let light_theme = light_theme.with_accent(bg_dominant_color);

    dark_theme.write_entry(&dark_config)?;
    light_theme.write_entry(&light_config)?;

    // Ménage
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
async fn main() -> Result<(), Error> {
    // Configurer le log
    if connected_to_journal() {
        JournalLog::new()?.install()?;
    } else {
        log::set_logger(&SimpleLogger)?;
    }
    log::set_max_level(LevelFilter::Error);

    set_bing_background().await.inspect_err(|why| log::error!("{why}"))
}
