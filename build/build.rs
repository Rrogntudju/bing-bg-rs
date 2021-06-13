fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("build/icon.ico").compile().unwrap();

    windows::build!(
        Windows::Win32::UI::WindowsAndMessaging::{
            SystemParametersInfoW, SYSTEM_PARAMETERS_INFO_ACTION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS
        },
    );
}
