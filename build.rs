// Uygulama simgesini ve dosya bilgilerini .exe'ye gömer.
fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/icon.ico"); // Kaynak kimliği 1 (main.rs buradan yükler)
    res.set("ProductName", "YT Music");
    res.set("FileDescription", "YT Music");
    res.set("LegalCopyright", "Kişisel kullanım için. YouTube Music, Google LLC'nin markasıdır.");
    res.compile().expect("Windows kaynakları derlenemedi");
}
