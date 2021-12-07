fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("build/icon.ico").compile().unwrap();
}
