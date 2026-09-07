# Güvenlik ve Gizlilik Politikası

> Bu dosyadaki kuralların ihlali **güvenlik hatasıdır**, stil tercihi değildir.
> Kod incelemesinde ve CI'da denetlenir.

## 1. Asla loglanmayacaklar

| # | Yasak | Neden |
|---|---|---|
| 1 | Medya URL'si | Token içerebilir; kullanıcının ne izlediğini açığa çıkarır |
| 2 | Token içeren query string | Kimlik bilgisi sızıntısı |
| 3 | Özel tam dosya yolu | Kullanıcı adı ve kütüphane yapısını açığa çıkarır |
| 4 | Subtitle diyaloğu (cue metni) | İçerik gizliliği |
| 5 | Raw provider response | Cue metni ve model çıktısını içerir |
| 6 | API key / secret | Kimlik bilgisi |
| 7 | OpenSubtitles private file ID | Sağlayıcı gizli tanımlayıcısı |
| 8 | Özel hash / filename metadata | Medya parmak izi, kimlik çıkarımına yarar |

### Loglanabilecekler

Cue **sayısı** · blok indeksi · faz adı · süre · hata **varyantı** (payload'sız) ·
dosya **uzantısı** · boyut sınıfı · opaque public source ID · redakte edilmiş
host adı (yalnız approved listede olan sabit host adları).

### `Debug` / `Display` kuralı

Gizli veri taşıyan hiçbir tipte `#[derive(Debug)]` **kullanılmaz**. `Debug`
elle yazılır ve hassas alanı `<redacted>` olarak basar:

```rust
// YANLIŞ
#[derive(Debug)]
struct MediaRef { url: String, path: PathBuf }

// DOĞRU
struct MediaRef { url: String, path: PathBuf }

impl fmt::Debug for MediaRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MediaRef")
            .field("url", &"<redacted>")
            .field("path", &"<redacted>")
            .field("extension", &self.extension())   // güvenli türev
            .finish()
    }
}
```

Bu kural NEN-006'da test edilir: hassas tiplerin `{:?}` çıktısı yasaklı desenleri
**içermemelidir**.

## 2. Untrusted girdiler

Şunların **hepsi** düşmanca kabul edilir; doğrulanmadan hiçbiri kullanılmaz:

subtitle metni · provider cevabı · OpenSubtitles cevabı · filename · media
metadata (container etiketleri dahil) · handoff extras · glossary · remote URL
path bileşenleri.

**Kural:** parse eden kod asla `unwrap`/`expect` kullanmaz; her hata typed error
döner. Untrusted metin doğrudan bir yol, komut, SQL veya format string'ine
gömülmez.

## 3. Ağ kuralları

| Kural | Detay |
|---|---|
| Yalnız HTTPS | Provider API istekleri HTTPS'tir; kullanıcı/handoff medya URL'leri S5 gereği `http/https` olabilir |
| Approved host | Sağlayıcı host'ları sabit listede; kullanıcı/handoff medya host'u bu listeyle sınırlandırılmaz |
| Bounded redirect | Yönlendirme sayısı sınırlı; her adımda host yeniden doğrulanır |
| Maksimum boyut | İndirme boyut sınırı aşılırsa akış kesilir, dosya reddedilir |
| Archive reddi | Zip/rar/gzip vb. altyazı arşivi kabul edilmez (zip bomb / path traversal) |
| Content-type doğrulaması | Beklenmeyen tipte içerik reddedilir |
| Retry sınırı | Yalnız bounded transient network/5xx retry. Doğrulama hatası retry sebebi değildir |
| Scraping yasak | Yalnız resmi API |

## 4. Dosya kuralları

Kullanıcının verdiği veya keşfedilen her altyazı dosyası için, **açmadan önce**:

1. **Regular file** olduğu doğrulanır (`stat`; dizin, FIFO, device, socket reddedilir)
2. **Symlink reddedilir** (izlenmez, çözülmez — doğrudan reddedilir)
3. **Path traversal reddedilir** — çözülmüş yol izin verilen kök altında kalmalı
4. **Boyut sınırı** uygulanır (sınırı aşan dosya okunmaz)
5. **Encoding** tespit edilir ve sanitize edilir (BOM, UTF-16, legacy code page, kontrol karakterleri)
6. **Strict parse** — geçemezse kaynak "hatalı" işaretlenir; **playback durmaz**

macOS'ta kullanıcı `Altyazı Dosyası Yükle…` panelinden bir symlink/alias'ı
açıkça seçtiğinde `NSOpenPanel` hedefi uygulamaya vermeden çözer; uygulamaya
ulaşan hedef yukarıdaki kapılardan geçer. Bu, uygulamanın kendi sidecar
keşfinde veya doğrudan dosya kapısına verilen symlink'in izlenmesi anlamına
gelmez; bu iki yüzeyde symlink reddi yürürlüktedir (ADR-0031, NEN-071).

Yazma tarafında: artifact commit **atomik** olmalıdır (geçici dosyaya yaz →
fsync → rename). Yarım dosya asla görünür olmaz.

**Sidecar keşfinin kapsamı (ADR-0041):** medyanın kendi dizini bir kez,
özyinelemesiz listelenir; adı `<basename>.` önekiyle başlayıp `.srt` ile biten
her girdi adaydır. Alt dizin yok, üst dizin yok, ikinci bir dizin yok. Her
aday yukarıdaki altı kapıdan **teker teker** geçer — dizin listelemesi aday
kümesini büyütür, kapıları gevşetmez. Aday sayısı **16** ile sınırlıdır; tam
basename eşleşmesi (`Film.srt`) sıralamada her zaman ilk gelir ve sınırdan
her zaman kurtulur.

## 5. Secret yaşam döngüsü

| Aşama | Kural |
|---|---|
| Giriş | Kullanıcı kendi anahtarını girer |
| Saklama | Platform secure storage: Keychain · Keystore · Credential Manager · Secret Service. **Plaintext config yasak** |
| Bellekte | Mümkün olduğunca kısa süre; kullanımdan sonra sıfırlanır |
| Gösterim | UI'da bir daha **tam gösterilmez** (yalnız son birkaç karakter / "kayıtlı" göstergesi) |
| Log | Asla |
| Artifact | Asla |
| Crash / telemetri | Asla |

Secret taşıyan tipler `Debug`, `Serialize` ve `Display` implementasyonlarında
değeri **açığa çıkarmaz**.

## 6. İptal ve tutarlılık

- İptal edilmiş bir işten **hiçbir callback gelmez**.
- İptal sonrası **late commit yasaktır** — iptal anından sonra artifact yazılmaz.
- Yarım/progressive subtitle **yayınlanmaz**.

## 7. Audio gizliliği (M8)

- Kullanıcı komutu olmadan **audio analizi yapılmaz**.
- Varsayılan mod `localOnly` — ses cihazdan çıkmaz.
- Remote AI'a ses gönderilecekse **açık kullanıcı izni** gerekir.
- Raw audio loglanmaz; geçici audio dosyaları işlem sonrası **silinir**.
- Mümkünse yalnız gerekli zaman pencereleri işlenir; tam medya dosyası varsayılan
  olarak yüklenmez.
- Protected/DRM audio extraction **yapılmaz**.

## 8. Kesin yasaklar

DRM bypass · scraping · torrent/debrid acquisition · protected içerik çıkarımı ·
testlerde gerçek provider kredisi/kotası kullanımı.

## 9. Test tarafı

Her güvenlik kuralının **negatif testi** olmalıdır: reddedilmesi gereken girdi
gerçekten reddediliyor mu? Örnekler:

- symlink olarak verilen `.srt` → reddedilir
- `../../etc/passwd` içeren yol → reddedilir
- boyut sınırını aşan dosya → okunmaz
- approved liste dışı host → istek atılmaz
- yönlendirme zinciri sınırı aşıyor → kesilir
- hassas tipin `{:?}` çıktısı → yasaklı desen içermez
