# Lisans Durumu

> **Bu bir lisans metni değildir.** Nen Player için henüz lisans seçilmemiştir.

## Bugünkü durum

Nen Player şu an **kişisel kullanım** için geliştiriliyor ve **side-loading**
ile dağıtılıyor. App Store veya başka bir public dağıtım kanalı hedefi
**ertelenmiştir** (roadmap **S11**).

## Neden kökte `LICENSE` dosyası yok

GitHub ve paket araçları depo kökündeki `LICENSE*` dosyasını **hukuki metadata**
olarak okur ve lisans rozetine dönüştürür. Oraya placeholder bir dosya koymak,
henüz verilmemiş bir kararı verilmiş gibi gösterir — bu, boş bırakmaktan daha
yanıltıcıdır.

Gerçek lisans seçilene kadar kökte lisans dosyası bulunmayacak; durum bu
dosyada anlatılacaktır.

## Kararı neyin beklettiği

| Bağımlılık | Etki |
|---|---|
| **ADR-0012** — macOS motor seçimi (aday: libmpv) | libmpv ve bağımlılıkları LGPL/GPL bileşenler içerebilir. Linkleme modeli (dinamik/statik) ve hangi mpv derleme konfigürasyonunun kullanıldığı, seçilebilecek lisansları doğrudan kısıtlar |
| **S11** — public dağıtım | Kişisel kullanım ile public dağıtım farklı yükümlülükler doğurur |
| **S12** — lisans ailesi | open-source / source-available / private — S11 ve ADR-0012 netleşmeden seçilemez |

## Karar zamanı

**M3 (macOS Vertical Slice) başlamadan önce**, ADR-0012 ile birlikte.

Karar verildiğinde: depo köküne gerçek lisans dosyası eklenecek, gerekçe
ADR-0012'ye yazılacak ve bu dosya o karara işaret edecek şekilde güncellenecektir.

## İlgili

- [`docs/adr/README.md`](adr/README.md) — ADR-0012
- [`docs/roadmap.md`](roadmap.md) — S11, S12
- [`docs/DECISIONS.md`](DECISIONS.md) — ertelenmiş kararlar
