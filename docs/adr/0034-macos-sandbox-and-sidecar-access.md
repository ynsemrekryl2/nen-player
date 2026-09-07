---
adr: 0034
title: macOS dağıtımı sandbox'sızdır; sidecar erişimi buna dayanır
status: accepted
milestone: M3
tasks: [NEN-025, NEN-043]
date: 2026-08-27
---

# ADR-0034 — macOS dağıtımı sandbox'sızdır; sidecar erişimi buna dayanır

## Durum

`accepted`

## Bağlam

M3'ün kullanıcı hikâyesi (`docs/milestones/M3-macos-slice.md`) şunu içeriyor:

> "Altyazı düğmesine bastığımda gömülü track'leri **ve dosyanın yanındaki
> `.srt`**'yi gruplu bir listede görüyorum."

`NEN-024` uygulamayı App Sandbox ile imzalıyor
(`platforms/macos/NenPlayer.entitlements` → `app-sandbox`,
`files.user-selected.read-only`, `files.bookmarks.app-scope`) ve
`scripts/build-macos-app.sh` bu entitlement'larla imzalıyor. Sandbox'ta
powerbox, `NSOpenPanel`'den seçilen **dosyaya** erişim uzantısı veriyor —
bulunduğu dizine değil. Yani "medyanın yanındaki `.srt`" tam olarak sandbox'ın
vermediği şey.

`NEN-025` Adım 0 bunu ölçtü (`evidence/M3/NEN-025-sandbox-measurement.md`).
Medyanın security scope'u **açıkken** üç mekanizma denendi, üçü de `EPERM`
(`errno=1`) döndü:

| Denenen | Sonuç |
|---|---|
| Kardeş `.srt`'yi `open(O_RDONLY)` | `DENIED errno=1` |
| Medyanın dizinini `opendir` | `DENIED errno=1` |
| Related-item koordinasyonu — `NSIsRelatedItemType` + `NSFilePresenter` + `NSFileCoordinator` | `DENIED errno=1` |

Üçüncüsü Apple'ın bu iş için gösterdiği mekanizmadır ve NEN-025'in planındaki
çözümdü. Medya tipleri de `CFBundleDocumentTypes`'a bildirildikten sonra
tekrarlandı; sonuç değişmedi. Yani karar "sandbox mı, kolaylık mı" değil:
**sandbox içinde bu özelliğin bilinen bir yolu ölçülüp elendi.**

Karar verilmezse M3 çıkış kriterine giden hikâye eksik kalır ve `NEN-025`
kapanamaz.

## Karar

macOS uygulaması **App Sandbox olmadan** dağıtılacaktır.
`com.apple.security.app-sandbox` ve ona bağlı
`com.apple.security.files.user-selected.read-only` /
`com.apple.security.files.bookmarks.app-scope` entitlement'ları
`platforms/macos/NenPlayer.entitlements` dosyasından kaldırılacaktır.

Bunun karşılığında:

1. **Mac App Store hedeflenmez.** `docs/roadmap.md`'nin non-goal listesine
   eklenir. Dağıtım kanalı Developer ID + notarization'dır (`NEN-043`).
2. **Dosya erişimi hâlâ kapılıdır.** `security-policy.md` §4 kapıları
   (`nen-app::subtitle_files`) sandbox'ın yerini almaz ama tek savunma hattı
   olarak kalır ve bu yüzden **zayıflatılamaz**: symlink, path traversal,
   regular-file ve boyut kapılarının negatif testleri `NEN-025`'in kapanış
   koşuludur.
3. **Uygulama yalnız kullanıcının gösterdiği yere bakar.** Açılan medyanın
   dizini dışında hiçbir yer okunmaz; sidecar taraması aynı basename'li tek
   dosyaya bakar, dizin listelemez, özyinelemeli aramaz.

## Gerekçe

**Ölçüm alternatifi eledi, tercih değil.** Related-item mekanizması üç ayrı
Info.plist kurulumunda `EPERM` verdi. Sandbox'ı korumanın kalan iki yolu vardı
ve ikisi de ürünü bozuyordu: klasör izni istemek, `dir-layouts` fixture'ının da
gösterdiği film-başına-klasör düzenlerinde (`Movies/Arrival (2016)/video.mkv`)
film başına bir izin istemi demektir; otomatik keşfi düşürmek ise M3'ün
hikâyesini düşürmektir.

**Sandbox notarization için gerekli değildir.** Notarization Developer ID ile
imzalı, hardened runtime'lı uygulamaları kabul eder; App Sandbox yalnız Mac App
Store dağıtımı için zorunludur. `NEN-043` bu yüzden bu karardan etkilenmiyor.

**Kategorinin gerçeği bu.** IINA, VLC ve mpv aynı sebeple sandbox'sız
dağıtılıyor: yerel medya oynatıcısı, kullanıcının kütüphanesindeki komşu
dosyalara bakmak zorunda.

**Varsayım:** Mac App Store'un hiçbir zaman hedeflenmeyeceği. Bugün roadmap'te
yok ve non-goal listesine yazılıyor; hedeflenirse bu ADR `superseded` olur ve
sidecar keşfi o milestone'da yeniden çözülür.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Sandbox kalsın, related-item koordinasyonu kullanılsın | **Ölçüldü ve çalışmadı** — `NSIsRelatedItemType` + `NSFileCoordinator` üç kurulumda da `EPERM`. `evidence/M3/NEN-025-sandbox-measurement.md` |
| Sandbox kalsın, kullanıcıdan klasör izni istensin (app-scope bookmark) | Entitlement zaten bildirilmiş, yani teknik olarak mümkün. Fakat izin dizin kapsamlı: film-başına-klasör düzenlerinde her film için bir istem çıkar. Ürünün en sık yaşanan anını bir izin diyaloğunun arkasına koyar |
| Sandbox kalsın, otomatik sidecar keşfi M3'ten çıkarılsın | M3 çıkış kriterine giden kullanıcı hikâyesinin bir maddesini düşürür; sorunu çözmez, erteler |
| Sandbox kalsın, altyazı da panelden seçilsin (elle yükleme yeterli sayılsın) | Yukarıdakinin aynısı, farklı sözlerle: kullanıcı zaten yanında duran dosyayı ikinci kez göstermek zorunda kalır |

## Sonuçlar

**Olumlu:** sidecar keşfi düz `std::fs` ile çalışır ve `nen-app`'teki kapılar
her platformda aynı kodla sınanır. Kullanıcı, medyayı açtıktan sonra ikinci bir
izin adımı görmez. `NEN-043` (bundling + notarization) etkilenmez.

**Olumsuz / kabul edilen maliyet:** süreç, kullanıcının kendi yetkisiyle dosya
sistemine erişebilir. Sandbox'ın sağladığı ikinci savunma hattı gider; geriye
`security-policy.md` §4 kapıları ve K23 kalır. Bu, o kapıların testlerini
"iyi olurdu" olmaktan çıkarıp **zorunlu** yapar. Mac App Store kapanır.

**Geri dönüş maliyeti:** **orta.** Entitlement'ı geri eklemek tek satır, fakat
sandbox'a dönmek sidecar keşfini yeniden çözmeyi gerektirir ve bu ADR o
problemin bilinen çözümünün çalışmadığını ölçmüş durumda. Yani geri dönüş
teknik olarak ucuz, ürün olarak pahalı.

## İlgili task'lar

`NEN-025`, `NEN-043`

## Notlar

Ölçümün tam kaydı: `evidence/M3/NEN-025-sandbox-measurement.md`.

**Karar 3'ün sidecar cümlesi ADR-0041 ile değişti (2026-09-07).** "Sidecar taraması aynı basename'li tek dosyaya bakar, dizin listelemez" artık geçerli değil: tarama medyanın **kendi dizinini bir kez, özyinelemesiz** listeliyor ve `<basename>.` önekli `.srt` adaylarını topluyor. Karar 3'ün geri kalanı — açılan medyanın dizini dışında hiçbir yerin okunmaması, özyineleme yasağı — ve Karar 2'nin "§4 kapıları tek savunma hattıdır, zayıflatılamaz" hükmü aynen yürürlükte; ADR-0041 kapıları değiştirmiyor, yalnız aday kümesini büyütüyor. Gerekçe ve kabul edilen maliyet için bkz. ADR-0041. Bu ADR **supersede edilmedi**: gövdesi olduğu gibi geçerli ve `adr:` alanıyla buraya referans veren done task'lar (`NEN-025`) etkilenmiyor.
