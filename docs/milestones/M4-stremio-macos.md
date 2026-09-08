# M4 — Stremio Handoff (macOS)

> Task kırılımı M3 kapanırken üretildi (2026-09-07).

## Amaç

Stremio'dan external player olarak açılan medyanın macOS'ta doğru pozisyondan oynaması. Nen Player bir Stremio add-on'u **değildir**; burada yalnız handoff alıcısı tarafı yapılır.

## Kapsam

- External-player launcher veya open-with handoff
- Positional file/http/https argümanı
- Optional başlangıç pozisyonu
- Handoff metadata'sının opsiyonel güçlü kanıt olarak kullanılması

## Kapsam dışı

- Android Intent tarafı → M10
- Stremio'ya sonuç/pozisyon döndürme → M10 (Android senaryosu)
- Canonical ID'nin **her zaman** geleceğini varsaymak — yasak

## Çıkış kriterleri

- [x] Gerçek Stremio 5.1.26'da "MPV içinde oynat" eylemi, geri alınabilir
      MPV köprüsü üzerinden Nen Player'ı açıyor ve gelen pozisyonu uyguluyor.
      Mevcut sürümün ölçülen `--start=0` davranışı baştan oynatma olarak
      kaydedilir; olmayan bir devam konumu tahmin edilmez.
- [x] Negatif: argüman ve medya URL'si **loglanmıyor** (log denetimi)
- [x] Handoff metadata yoksa akış bozulmuyor (kanıt opsiyoneldir)

## Kabul senaryosu (NEN-084)

Kabul koşusu **2026-09-08** tarihinde Apple Silicon / macOS 27.0 üzerinde
gerçek Stremio 5.1.26 ve gerçek Nen Player `.app` ile yapıldı. ADR-0043'ün
ölçülmüş sınırı gereği Stremio'nun kapalı harici oynatıcı listesi Nen Player'ı
doğrudan hedeflemiyor; bu nedenle Stremio'da canlı yüzey (VLC/MPV harici
oynatıcı menüsü ve sürüm) yeniden görüldü, ardından aynı CLI sözleşmesi
fixture tabanlı gerçek `.app` koşusuyla kabul edildi.

Bu koşu artık tarihsel CLI sözleşmesi kanıtıdır. M4'ün ilk kriteri, `NEN-087`
köprüsü kurulup `NEN-088` gerçek Stremio koşusu tamamlanana kadar açık kalır.

1. `contract-clip.mkv`, `--start` ve bilinmeyen `--no-terminal` bayrağıyla
   açıldı; Nen Player ekranında medya **00:12 / 00:30** konumunda oynuyordu.
2. Aynı fixture, geçici loopback sunucusundan opak adla ve
   `Content-Disposition` olmadan açıldı; metadata yokken medya **00:09 / 00:30**
   konumunda oynadı, hata veya kesinti olmadı.
3. Unified log ile stdout/stderr geçici olarak tarandı; locator, URL/query,
   özel yol ve argv parçaları için eşleşme bulunmadı. Ham çıktı saklanmadı.

Ekran ve ayrıntılı adım kaydı: `evidence/M4/NEN-084-checklist.md`.

## Task'lar

`NEN-078` · `NEN-079` · `NEN-080` · `NEN-081` · `NEN-082` · `NEN-083` ·
`NEN-084` · `NEN-086` · `NEN-087` · `NEN-088`

```
078 ölçüm ──▶ 079 ADR ──▶ 080 alıcı yüzey ──┬──▶ 081 başlangıç pozisyonu ──┐
                                             ├──▶ 082 metadata → kanıt ────┼──▶ 084 CLI kanıtı
                                             └──▶ 083 log denetimi ────────┘
084 düzeltme ──▶ 086 launcher ölçümü + ADR-0044 ──▶ 087 bridge ──▶ 088 gerçek kabul
```

**Sıra ölçümle başlıyor, kararla devam ediyor.** Alıcı yüzeyin ne olması
gerektiğini yalnız gönderen taraf söyler; `NEN-078` bunu gerçek Stremio ile
ölçer, `NEN-079` ADR'ye bağlar. `NEN-025`/ADR-0034 emsali: ölçüm kod yazılmadan
önce yapılır ve planlanan çözümü elemeye yetkilidir.

**Çıkış kriteri ↔ task eşleşmesi:**

| Çıkış kriteri | Kanıtlayan |
|---|---|
| Gerçek Stremio eylemi Nen Player'ı açıyor ve gelen pozisyonu uyguluyor | `NEN-086` + `NEN-087`, üründe `NEN-088` |
| Argüman ve medya URL'si loglanmıyor | `NEN-083` (negatif kontrol zorunlu) |
| Metadata yoksa akış bozulmuyor | `NEN-082` |

**Zaten hazır olan, yeniden yazılmayacak zemin:** `PlaybackSession::load`
(`core/crates/nen-app/src/session.rs`) · `remote_evidence::validate_url`'ün
şema kapısı · ADR-0042 + `deferredSeekMs`'in yüklenirken-seek kontratı
(`NEN-052`) · ADR-0009'un kanıt katmanları · `guard_playback_debug.rs`'in
kapalı-küme log denetimi deseni · `RecentMediaStore`'un bookmark yolu.

**Mimari soru `ADR-0043` ile kapandı:** `docs/architecture.md`'nin sınır kuralı
"Stremio handoff"u `port + platform adapter` sınıfına koyuyordu, ama aynı
dosyanın port tablosunda böyle bir port yoktu. Karar: yeni port açılmadı,
sınır kuralı düzeltildi — handoff **inbound** bir OS olayı sayılır (core
çağırmaz, OS kabuğa iter), sonuç mevcut `PlaybackSession::load` + seek
çağrısına düşer. ADR ayrıca alıcı yüzeyi (argv + open-with + custom scheme,
üçü birden), başlangıç pozisyonunun taşıyıcısını (argv bayrağı/scheme
fragment'ı, ADR-0042'nin `deferredSeekMs` kontratından geçer) ve
ayrıştırmanın core'da yapılacağını sabitledi. `NEN-080` bu kararı uygular.

## Bağımlılıklar

M3

## Düzeltme notu

`NEN-084`'ün ölçülmüş CLI sözleşmesi koşusu gerçek Stremio'nun Nen Player'ı
hedeflediği anlamına gelmiyordu. Kurulu shell'in MPV launcher'ı sabit bir
executable yolunu arıyor ve bu makinede o yolda önceden kurulmuş bir wrapper
bulunuyordu. `NEN-086` bu korelasyonu ve geri alınabilir köprü kararını
belgeliyor; `NEN-087`/`NEN-088` tamamlanmadan M4 kapanış retrosu yazılmayacak.

## Retro

**Süre ve çıktı.** M4, 2026-09-07'de başladı ve 2026-09-08'de **10 task** ile
kapandı. Gerçek Stremio 5.1.26 eylemi geri alınabilir MPV köprüsü üzerinden
Nen Player'ı hem soğuk hem sıcak handoff'ta açtı; Stremio'nun ölçülen `0`
başlangıç değeri baştan oynatma olarak doğrulandı. Negatif log sayaçları sıfır
kaldı. `NEN-088` kanıtı: `evidence/M4/NEN-088-checklist.md`.

**Yanlış çıkan varsayım.** `NEN-084`'ün ölçülmüş MPV CLI sözleşmesinin gerçek
Stremio'nun Nen Player bundle'ını doğrudan hedeflediği varsayımı yanlıştı.
`NEN-086` gerçek launcher yolunu ölçtü, `ADR-0044` geri alınabilir köprüyü
karara bağladı, `NEN-087` köprüyü güvenli kurdu ve `NEN-088` canlı akışı
kanıtladı. Stremio uygulama paketi veya bundle kimliği değiştirilmedi.

**Kararlar.** `ADR-0043` alıcı yüzeyi ve handoff ayrıştırmasını, `ADR-0044`
ise sabit MPV launcher yoluna bağlanan geri alınabilir köprüyü kabul etti.
Hiçbir ADR supersede edilmedi.

**Sonraki milestone.** Sıra **M5 — Translation Core**'da. İlk hazır işler
`NEN-044` (embedded subtitle text extraction) ve `NEN-072` (remote container
metadata); her ikisinin de bağımlılıkları tamamlandı.
