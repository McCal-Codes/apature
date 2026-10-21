fn main() {
    // Windows: embed the app icon and version into apature.exe.
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("ProductName", "Apature");
        res.set(
            "FileDescription",
            "Apature: contact sheet, culling, captions and delivery for photojournalists",
        );
        let _ = res.compile();
    }
    println!("cargo:rerun-if-changed=assets/icon.ico");
}
