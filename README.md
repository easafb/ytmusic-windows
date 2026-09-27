# YT Music (Windows)

music.youtube.com için hafif bir Windows uygulaması. Rust ile yazıldı; Windows'ta zaten kurulu olan
Microsoft Edge WebView2 motorunu kullanır, bu yüzden Electron uygulamaları gibi kendi tarayıcısını taşımaz.
Tek bir küçük `.exe` dosyasıdır.

Mac sürümü: [easafb/ytmusic-mac](https://github.com/easafb/ytmusic-mac)

## Özellikler
- Google hesabınla giriş (oturum kalıcı; Premium ile reklamsız)
- Sistem tepsisi simgesi: sol tık pencereyi gösterir/gizler, sağ tık menü (oynat/duraklat, sonraki, önceki)
- Pencereyi kapatınca müzik tepside çalmaya devam eder
- **Arka planda az bellek:** Pencere gizlenince ya da küçültülünce çizim durur ve WebView2 düşük bellek
  moduna geçer (`MemoryUsageTargetLevel.Low`); müzik çalmaya devam eder
- Klavye medya tuşları ve Windows medya paneli: WebView2'nin kendi desteğine bırakıldı, henüz doğrulanmadı
- Pencere boyutu ve konumu hatırlanır; ikinci kez açınca var olan pencere öne gelir
- Ctrl + / Ctrl − ile yakınlaştırma, F5 ile yenileme

## İndir
[Son sürümü indir](https://github.com/easafb/ytmusic-windows/releases/latest) → `YT-Music-Windows.zip` içindeki
`YTMusic.exe`'yi istediğin bir klasöre koyup çalıştır. Kurulum gerekmez.

Uygulama imzalı olmadığı için Windows ilk açılışta **"Windows kişisel bilgisayarınızı korudu"** uyarısı verebilir:
**Ek bilgi → Yine de çalıştır**.

**Gereksinim:** Windows 10 veya 11 (WebView2 Runtime; Windows 11'de hazır gelir, Windows 10'da genelde Edge ile
birlikte kuruludur. Yoksa: https://go.microsoft.com/fwlink/p/?LinkId=2124703)

Veriler (oturum, pencere konumu): `%LOCALAPPDATA%\YTMusic`

## Derleme
Windows'ta [Rust](https://rustup.rs) kuruluyken:
```
cargo build --release
```
Çıktı: `target\release\YTMusic.exe`. GitHub Actions her gönderimde derler; `v*` etiketiyle release'e zip ekler.

> Kişisel kullanım için yapılmıştır. YouTube Music, Google LLC'nin markasıdır; bu proje Google ile bağlantılı değildir.
