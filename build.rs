fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("Resources/assets/images/icon.ico");
        res.compile().expect("Failed to set application icon");
    }
}
