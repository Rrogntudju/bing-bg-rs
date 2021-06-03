fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("build/icon.ico").compile().unwrap();

        windows::build!(
            Windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_SETDESKWALLPAPER, SPIF_UPDATEINIFILE, SPIF_SENDCHANGE},
        );
    }
}
