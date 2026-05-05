fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icon.ico");
        res.set("ProductName", "Build Stream");
        res.set("FileDescription", "Build Stream");
        res.set_manifest_file("app.manifest");
        res.compile().expect("Failed to compile Windows resources");
    }
}
