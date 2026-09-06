# NEN-042 — Boş durumda son açılanlar listesi

Tarih: **2026-09-06** · Apple Silicon · macOS 27.0 (26A5421a) · Xcode 26.6 ·
Swift 6.3.3 · libmpv 2.5.0 (mpv 0.41.0_8, Homebrew)

## Neyin değiştiği

`RecentMediaStore.swift` tek bookmark'lık `UserDefaultsRecentMediaStore`'u
5 kayıtlık bir listeye çevirdi (`RecentMediaEntry` — yalnız `id` ve
`displayName`, ADR-0031 Karar 2 gereği yol taşımıyor). `PlayerModel.recentMedia`
diziye döndü, `openRecentMedia(_:)` artık kayıt kimliğiyle çağrılıyor,
çözülemeyen kayıt yalnız **kendisini** düşürüyor (`recentStore.remove(id)`) —
`NEN-050`'de ertelenen `resolve()` kusuru da bu geçişte kapandı: bayat bookmark
artık security scope **açıkken** tazeleniyor ve tazeleme başarısız olsa bile
zaten çözülmüş URL kayıptan düşmüyor. `PlayerCommands`'e `Son Açılanları
Temizle` madde eklendi (liste boşken devre dışı).

## Otomatik kanıt

`RecentMediaStoreTests.swift` (yeni, 11 test — gerçek `UserDefaults` suite +
depodaki `fixtures/media/*.mkv`): sıra, round-trip, kapasite kırpma (5),
dedup-and-move-to-front, bayat bookmark + tazeleme hatası (kayıt korunuyor),
bayat bookmark + tazeleme başarılı (bookmark verisi güncelleniyor), uzak
URL'nin sessizce reddi, tek seferlik göç (eski `recentMediaBookmark` /
`recentMediaDisplayName` çifti listeye taşınıp göç bayrağıyla tekilleşiyor),
`remove` yalnız adlanan kaydı düşürüyor, `clear` listeyi ve saklanan anahtarı
siliyor.

`PlayerModelTests.swift`: medya açılınca kayıt listenin başında; **çözülemeyen
kayıt yalnız kendi id'sini düşürüyor** (`clearCount == 0`, `removedIds ==
[gone]`, kalan kayıt duruyor) — hem `resolve` `nil` dönünce hem `resolve` throw
edince, iki ayrı test; save hatasında medya yine yükleniyor; `clearRecentMedia()`
listeyi boşaltıyor. `recentMediaSaveFailedMessage` / `recentMediaUnavailableMessage`
kapalı küme testi değişmeden geçti (metinler aynı kaldı).

Negatif kontrol beş ayrı düzeltmede, her biri kendi başına geri alınıp tek
başına kırmızı üretti, sonra geri konuldu:

| Kaldırılan davranış | Kırmızıya dönen |
|---|---|
| Tazeleme hatasının yutulması | yalnız bayat+tazeleme-hatası testi |
| Dedup (`filter` by key) | yalnız dedup testi |
| Kapasite kırpması | yalnız kapasite testi |
| `isFileURL` kapısı | yalnız uzak-URL testi (Foundation bookmark API'si zaten reddediyor, ama kapı olmadan hata `recentMediaSaveFailedMessage` olarak yanlış yüzeye çıkardı) |
| `PlayerModel`'de per-entry `remove` yerine `clear` | yalnız iki "yalnız kendi id'sini düşürür" testi (ikisi birden, ayrık) |

Her seferinde suite'in geri kalanı yeşil kaldı. Swift paketi **paralel ve seri
195/195** (öncesi 184 — `NEN-050`'nin son kaydı), Rust workspace **568
passed / 1 ignored** (bu task Rust'a dokunmadı), `cargo fmt --check` ve
`cargo clippy --workspace --all-targets -- -D warnings` temiz, `.app` build'i
ve `codesign --verify --deep --strict` yeşil.

## Gerçek `.app` üzerinde manuel kabul

Ad-hoc imzalı `.build/NenPlayer.app`, yalnız depodaki `fixtures/media/*.mkv`
ile çalıştırıldı (ADR-0031 Karar 2: kanıta giren ekran görüntüsü yalnız
fixture'la üretilir).

| Senaryo | Gözlem | Sonuç |
|---|---|---|
| Altı fixture sırayla açıldı (`contract-clip.mkv` → `menu-clip.mkv` → `aspect-4x3-clip.mkv` → `aspect-cinema-clip.mkv` → `anamorphic-clip.mkv` → `glass-dark-clip.mkv`), sonra `⌘Q` ile tam kapatılıp yeniden açıldı | Boş durumda tam **5** satır, en yeniden eskiye: `glass-dark-clip.mkv, anamorphic-clip.mkv, aspect-cinema-clip.mkv, aspect-4x3-clip.mkv, menu-clip.mkv` — en eski iki kayıt (`contract-clip.mkv` ve kapasiteden önce depoda duran bir kayıt) düştü; yalnız dosya adları görünüyor, hiçbir satırda yol yok | ✅ `NEN-042-recent-list-order.png` |
| Bir satıra tıklandı | Doğru dosya açıldı, oynatma başladı | ✅ |
| Listelenen bir dosya gerçekten silindi (bookmark hedefi kayboldu), yeniden başlatılıp o satıra tıklandı | Yalnız o satır düştü, `Son açılan medya artık kullanılamıyor.` transient'i göründü, kalan dört satır yerinde kaldı, uygulama çökmedi | ✅ `NEN-042-after-eviction.png` |
| Kalan bir satıra tekrar tıklandı | Doğru şekilde açıldı — eviction diğer kayıtları bozmadı | ✅ |
| `Dosya ▸ Son Açılanları Temizle` | Liste anında boşaldı; madde liste boşken devre dışı; `⌘Q` + yeniden açılışta liste boş kaldı | ✅ |

**Bir gözlem, üründe değil OS API'sinde:** aynı klasör içinde yeniden adlandırılan
bir dosyanın (`glass-dark-clip.mkv` → `glass-dark-clip.mkv.movedaway`) bookmark'ı
hâlâ çözülüyor ve dosya sorunsuz açılıyor — macOS'un security-scoped bookmark'ı
aynı birim içindeki yeniden adlandırma/taşımaya kasıtlı olarak dayanıklı.
"Taşınan/silinen dosya listeden düşer" DoD maddesi bu yüzden dosyanın
**gerçekten** kaldırılmasıyla (repodaki fixture'a dokunulmadan, ayrı bir
kopyayla) sınandı; yeniden adlandırma testi ayrıca kayıt altına alındı çünkü
DoD'un "taşınan" ifadesinin gerçek OS davranışıyla örtüşmediği ilk elden
gözlemlendi.

## Kapılar

`bash scripts/test-macos.sh` (paralel ve `--no-parallel`, ikisi de **195/195**),
Rust workspace **568/568** (1 ignored), `cargo fmt --check`, `cargo clippy
--workspace --all-targets -- -D warnings`, `.app` build, `codesign --verify
--deep --strict`, `bash scripts/check-docs.sh` — hepsi yeşil.
