# NEN-027 — Enjeksiyon yolu ölçümü

> Kanıt tipi: **architecture spike** (`docs/testing-strategy.md` → "Kanıt
> formatı") — ADR-0013'ün Karar 1 ve 4'ünü sayıya bağlayan ölçüm.
>
> Ortam: macOS 27.0 · libmpv 2.5.0 · 2026-08-29.
> Medya: `fixtures/media/contract-clip.mkv`. Ölçüm doğrudan libmpv C API'siyle
> alındı (`vo=null`, `ao=null`); mpv CLI bu makinede dönmüyor — NEN-022 ve
> NEN-058 bunu zaten kaydetti. K23: hiçbir yerde özel tam dosya yolu yok.

## Soru

Kullanıcının altyazı belgesi motora **diske hiç yazılmadan** verilebilir mi?
Verilemiyorsa tek alternatif diyaloğu geçici dosyaya yazmak — kullanıcının
özel metnini dosya sistemine bırakan bir karar, ve bu yüzden ölçülmeden
alınmaması gereken bir karar.

## Ölçüm

Adapter'ın kendi başlangıç ayarlarıyla (`config=no`, `terminal=no`, `pause=yes`,
`keep-open=yes`, `sid=no`, `sub-auto=no`) bir motor kuruldu, fixture yüklendi ve
üç cue'luk bir WebVTT belgesi `memory://` üzerinden enjekte edildi.

```
sub-add memory://<webvtt> select   ->  0 (success)
```

**Enjeksiyondan önce ve sonra `track-list`:**

| # | tür | id | ff-index | external | codec |
|---|---|---|---|---|---|
| 0 | video | 1 | 0 | no | h264 |
| 1 | audio | 1 | 1 | no | opus |
| 2 | audio | 2 | 2 | no | opus |
| 3 | sub | 1 | 3 | no | subrip |
| 4 | sub | 2 | 4 | no | subrip |
| **5** | **sub** | **3** | **0** | **yes** | **webvtt** |

Enjeksiyondan sonra `sid = 3` — `select` bayrağı track'i kendisi seçiyor,
ayrıca bir `sid` yazmaya gerek yok.

**Çizilen metin, seek sonrası:**

| Seek | `sub-text` |
|---|---|
| 2.0 s (1. cue içinde) | `first line` |
| 12.0 s (cue'suz boşluk) | `` (boş) |
| 20.5 s (3. cue içinde) | `third line` |

```
sub-remove  ->  0     (track-list 6 → 5, enjekte track kayboluyor)
```

## Bulgular

1. **`memory://` altyazı için kabul ediliyor.** Diyalog diske yazılmıyor,
   geçici dosya gerekmiyor, temizlik sorunu yok. ADR-0013 Karar 4 bunun
   üzerine kuruluyor.
2. **Engine-native çizim doğru cue'yu veriyor** ve cue'suz anda boş kalıyor —
   ADR-0013 Karar 1'in "güvenilir" koşulu (product-spec §14) bu fixture'da
   sağlanıyor. Beklenen/gerçekleşen birebir karşılaştırması task'ın kalıcı
   platform testinde yapılıyor; buradaki üç nokta ölçümün kendisi.
3. **Enjekte edilen track `ff-index = 0` ile geliyor.** Port'un `TrackId`'si
   ff-index olduğu için bu, video track'inin indeksiyle aynı sayı ve
   kataloğun gömülü kaynak id'leriyle çakışıyor. NEN-058 aynı sayıyı
   `sub-auto`'nun açtığı dosyada görmüştü (`tracks(.subtitle) = [3, 4, 0]`).
   Bu yüzden `external = yes` olan track'in enumeration'dan **elenmesi**
   NEN-027'nin isteğe bağlı bir temizliği değil, doğruluk koşulu: elenmezse
   enjekte ettiğimiz altyazı, kataloğun hiç görmediği bir "gömülü track"
   olarak menüde belirir.
4. `sub-remove` track'i eksiksiz düşürüyor, yani yeni bir belge göstermeden
   önce eskisini kaldırmak tek komut.

## Yeniden üretim

Ölçüm, adapter ayarlarını birebir kuran küçük bir C programıyla alındı
(`mpv_create` → `loadfile` → `sub-add memory://…` → `seek` → `sub-text`).
Kalıcı biçimi task'ın platform testidir
(`platforms/macos/Tests/NenPlaybackMPVTests/`), tek seferlik program depoya
girmedi — spike kodu ürün kodu değildir (Rule 7).
