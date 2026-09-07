# NEN-043 — Bundle libmpv into the .app — kanıt

Kapsam daraltıldı: Developer ID imzası, hardened runtime, notarization ve
`spctl -a -vv` bu makinede kanıtlanamaz (bkz. task dosyasının YAPILMAYACAK
bölümü) ve **S11**'e ertelendi. Bu task'ın DoD'u ad-hoc imzalı bir bundle'ın
gerçekten kendi kendine yeterli olduğunu kanıtlar.

## 1. `otool -L` taraması — Homebrew referansı kalmadı

`scripts/bundle-macos.sh`, gömme sonunda bundle içindeki **her** Mach-O'yu
(yürütülebilir + 48 dylib) kendi kendine tarar; `/opt/homebrew` veya
`/usr/local` geçen tek satır bulursa çıkış 1 verir.

```
▶ 6/6  bundle taranıyor: kalan Homebrew referansı var mı?
  temiz — hiçbir Mach-O /opt/homebrew veya /usr/local'a işaret etmiyor

Uygulama: /Users/ye/Developer/nen-player/platforms/macos/.build/NenPlayer.app
Gömülü dylib sayısı: 48
```

Bağımsız doğrulama (script'in kendi taramasından ayrı, elle):

```
$ otool -L platforms/macos/.build/NenPlayer.app/Contents/MacOS/NenPlayer | head -3
	@rpath/libmpv.2.dylib (compatibility version 2.0.0, current version 2.0.0)
	/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1356.0.0)
	/System/Library/Frameworks/AppKit.framework/... (System framework)

$ otool -L platforms/macos/.build/NenPlayer.app/Contents/Frameworks/libmpv.2.dylib | head -3
platforms/macos/.build/NenPlayer.app/Contents/Frameworks/libmpv.2.dylib:
	@rpath/libmpv.2.dylib (compatibility version 2.0.0, current version 2.0.0)
	@rpath/libass.9.dylib (compatibility version 14.0.0, current version 14.2.0)
```

## 2. `codesign --verify --deep --strict` — ad-hoc imza geçiyor

```
$ codesign --verify --deep --strict --verbose=2 platforms/macos/.build/NenPlayer.app
.../NenPlayer.app: valid on disk
.../NenPlayer.app: satisfies its Designated Requirement
```

## 3. Lisans metinleri ve THIRD-PARTY bildirimi

`Contents/Resources/licenses/` içinde:

```
LICENSE               — depo kökündeki GPL-3.0-or-later tam metni
mpv-Copyright          — mpv formülünün Copyright dosyası
mpv-LICENSE.GPL        — mpv'nin GPL metni
mpv-LICENSE.LGPL       — mpv'nin LGPL metni
THIRD-PARTY.md          — 48 gömülü dylib, her biri Homebrew formülü +
                           sürüm + SPDX lisansıyla (script tarafından üretilir)
```

`THIRD-PARTY.md`'den örnek satırlar:

```
| Dosya | Formül | Sürüm | Lisans |
|---|---|---|---|
| `libmpv.2.dylib` | `mpv` | 0.41.0_8 | GPL-2.0-or-later AND LGPL-2.1-or-later |
| `libavcodec.63.dylib` | `ffmpeg` | 9.0.1_1 | GPL-3.0-or-later |
| `libass.9.dylib` | `libass` | 0.17.5 | ISC |
| `libbluray.4.dylib` | `libbluray` | 1.5.0 | LGPL-2.1-or-later |
```

## 4. Negatif kontrol — gerçek `.app`, gerçek makine

`/opt/homebrew/Cellar` geçici olarak `/opt/homebrew/Cellar.nen-043-off`
adına taşındı (ölçüldü: `ye:admin` sahipli, sudo gerekmiyor;
`git`/`swift`/`bash`/`python3` hepsi `/usr/bin`–`/bin`'den geliyor, kabuk
etkilenmiyor). Üç ölçüm **aynı gizli-Cellar penceresinde**, tek shell
çağrısında alındı (`trap` ile garantili geri yükleme):

**(a) Kontrol sağır değil.** Gömme-öncesi (Homebrew'a dinamik bağlı,
`scripts/build-macos-app.sh` çıktısı) bir snapshot, Cellar gizliyken
başlatılamıyor:

```
dyld[46346]: Library not loaded: /opt/homebrew/opt/mpv/lib/libmpv.2.dylib
  Referenced from: .../unbundled-snapshot.app/Contents/MacOS/NenPlayer
  Reason: tried: '/opt/homebrew/opt/mpv/lib/libmpv.2.dylib' (no such file), ...
unbundled exit=134
```

**(b) Asıl iddia.** Gömülü `.app`, **aynı gizli-Cellar penceresinde**
çalışıyor, gerçek bir fixture'ı (`GTAVI_An_Extended_Look.mp4`, uygulamanın
kendi "son açılanlar" listesinden) açıp gerçek kare çiziyor ve oynatma
konumu ilerliyor (`00:01`, duraklat düğmesi aktif → oynuyor):

- Ekran görüntüsü, boş durum (gizli Cellar, uygulama sağlıklı açıldı):
  [`NEN-043-hidden-cellar-empty-state.png`](NEN-043-hidden-cellar-empty-state.png)
- Ekran görüntüsü, gerçek oynatma (gizli Cellar, gerçek video karesi):
  [`NEN-043-hidden-cellar-playback.png`](NEN-043-hidden-cellar-playback.png)

```
$ pgrep -x NenPlayer   # Cellar hâlâ gizliyken
46644
$ osascript -e 'tell application "System Events" to tell process "NenPlayer" to get name of window 1'
GTAVI_An_Extended_Look.mp4
```

**(c) Yüklenen image listesi.** Aynı çalışan süreç üzerinde `vmmap`:

```
$ vmmap 46644 | grep -c "/opt/homebrew"
0
$ vmmap 46644 | grep "libmpv.2.dylib" | head -1
__TEXT   ...  r-x/rwx SM=COW  .../NenPlayer.app/Contents/Frameworks/libmpv.2.dylib
```

Cellar her ölçüm bloğunun sonunda `trap` ile geri yüklendi; makine normal
duruma döndü (`pkg-config --modversion mpv` → `2.5.0`, doğrulandı).

## 5. `bash scripts/test.sh`

```
▶ bundle-macos.test.sh
    ok   rewrite_macho_deps.py çıkış 0
    ok   manifest 2 satır (liba + libb)
    ok   kopyalanan dosyalar install-name basename'iyle adlandı ...
    ok   yeniden yazılmış bundle'da vendor yolu kalmamış
    ok   mainbin @rpath/liba.1.dylib'e bağlı
    ok   liba.1.dylib @rpath/libb.2.dylib'e bağlı (geçişli bağımlılık takip edildi)
    ok   liba.1.dylib'in kendi LC_ID_DYLIB'i @rpath'e çevrildi
    ok   mainbin LC_RPATH = @executable_path/../Frameworks
    ok   liba.1.dylib LC_RPATH = @loader_path
    ok   yeniden yazılmış binary, vendor hâlâ diskteyken çalışıyor (sağlık kontrolü)
    ok   yeniden yazılmış binary, vendor GİZLİYKEN de çalışıyor (asıl iddia)
    ok   ham (yeniden yazılmamış) kopya vendor gizliyken çöküyor — kontrol sağır değil
    ok   bundle-macos.sh'in tarama deseni ham kopyada vendor yolunu buluyor
  ✓ bundle-macos.test.sh geçti

SONUÇ: 3 test dosyasının hepsi geçti.
```

Bu sentetik test, sahte bir Homebrew düzeni (`Cellar/<formül>/<sürüm>/` +
`opt/<formül>` sembolik bağı, iki katmanlı sürüm sembolik bağı) üzerinde
`scripts/lib/rewrite_macho_deps.py`'nin gerçek mantığını çalıştırıp, vendor
prefix'i diskten kaldırıldıktan **sonra** yeniden yazılmış zincirin hâlâ
çalıştığını (gerçek çalıştırma, yalnız statik okuma değil) doğruluyor.

## Diğer kapılar

| Kapı | Sonuç |
|---|---|
| `cargo fmt --check` | ok |
| `cargo clippy --workspace --all-targets -- -D warnings` | ok |
| `cargo test --workspace` | tüm crate'ler `ok`, hiç regresyon yok (Rust'a dokunulmadı) |
| `cargo deny check` | `advisories ok, bans ok, licenses ok, sources ok` |
| `bash scripts/test-macos.sh` | **202/202**, 22 suite (Swift'e dokunulmadı) |
| `bash scripts/check-docs.sh` | çıkış 0 |

## Kapsam dışı bırakılan (S11)

Developer ID imzası, hardened runtime, notarization ve `spctl -a -vv` —
ölçülen engel: `security find-identity -v -p codesigning` → 0 valid
identities; `xcrun notarytool history` → "Must provide credentials". Apple
Developer Program üyeliği olmadan bu makinede kanıtlanamaz.
