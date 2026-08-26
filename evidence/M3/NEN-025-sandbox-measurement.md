# NEN-025 Adım 0 — sidecar erişimi sandbox altında ölçüldü

**Tarih:** 2026-08-27 · **Makine:** Apple M5 / macOS 27 (Darwin 27.0.0)
**Yapı:** `bash scripts/build-macos-app.sh` — ad-hoc imza, `NenPlayer.entitlements`
(`app-sandbox` · `files.user-selected.read-only` · `files.bookmarks.app-scope`)

## Soru

Kullanıcı `NSOpenPanel`'den bir medya seçtiğinde, sandbox o medyanın **yanındaki**
aynı basename'li `.srt`'ye erişim veriyor mu? NEN-025'in sidecar keşfi buna
bağlı.

## Yöntem

`PlayerModel.openMedia(at:)` içine geçici bir prob eklendi (commit edilmedi,
ölçümden sonra `git checkout` ile geri alındı). Prob, medya URL'sinin
`startAccessingSecurityScopedResource()` çağrısından **sonra** üç şey deniyor ve
sonucu yalnız `errno` olarak logluyor — K23 gereği hiçbir yol loglanmadı:

1. kardeş `.srt`'yi `open(O_RDONLY)` ile açmak
2. medyanın dizinini `opendir` ile listelemek
3. aynı `.srt`'yi **related-item koordinasyonuyla** okumak —
   `NSFilePresenter` (`primaryPresentedItemURL` = medya,
   `presentedItemURL` = sidecar) + `NSFileCoordinator(filePresenter:)`

Fixture: `fixtures/media/contract-clip.mkv`'nin `Probe.mkv` adlı kopyası ve
yanında geçerli bir `Probe.srt`, depo dışında geçici bir dizinde. Dosya her
turda `⌘O` panelinden elle seçildi; panel gerçek powerbox iznini veriyor.

## Ölçüm

| Tur | Info.plist | Sonuç |
|---|---|---|
| 1 | değişmemiş | `scope=YES file=DENIED errno=1 dir=DENIED errno=1` |
| 2 | + `UTImportedTypeDeclarations` (SubRip) + `CFBundleDocumentTypes` → `NSIsRelatedItemType` | `file=DENIED errno=1 dir=DENIED errno=1 coord=DENIED errno=1` |
| 3 | 2'ye ek olarak medya tipleri de `CFBundleDocumentTypes`'ta (`public.movie` vd.) | `file=DENIED errno=1 dir=DENIED errno=1 coord=DENIED errno=1` |

`errno=1` = `EPERM`. Her turda uygulama tamamen sonlandırılıp yeniden
başlatıldı ve `lsregister -f` ile yeniden kaydedildi; 3. turda medya sorunsuz
oynadı, yani ölçüm bozuk bir kurulumda alınmadı.

`scope=YES` önemli: security scope medya için **açıktı**. Yani red, izni
almayı unutmaktan değil, iznin kapsamından geliyor — powerbox uzantısı seçilen
**dosyaya** çıkıyor, dizinine değil.

## Bulgu

1. Sandbox altında kardeş dosya okunamıyor ve dizin listelenemiyor.
2. **Related-item mekanizması bu kurulumda erişim vermiyor** — üç turda da
   `EPERM`. Beklenen çözüm buydu ve ölçüm onu eledi.

## Sonuç

NEN-025'in sidecar keşfi mevcut entitlement kümesiyle **çalışamaz**. Kapsam
kararı gerekiyor; seçenekler ve maliyetleri task dosyasında.

Kullanıcının elle yüklediği dosya bu ölçümden etkilenmiyor: paneli kullanan her
yol powerbox iznini zaten alıyor.
