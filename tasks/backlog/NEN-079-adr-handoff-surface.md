---
id: NEN-079
title: Decide the macOS handoff receiving surface
milestone: M4
size: S
state: backlog
closed:
depends_on: [NEN-078]
blocks: [NEN-080]
adr: []
---

# NEN-079 — Decide the macOS handoff receiving surface

## Sonuç

Nen Player'ın başka bir uygulamadan gelen medyayı hangi yüzeyden aldığı,
başlangıç pozisyonunu nereden okuduğu ve handoff'un kendi portunu alıp almadığı
`accepted` bir ADR ile sabitlenmiştir.

## Bağlam

Kural 4: mimari karar → önce ADR. Bu ADR olmadan `NEN-080` kod yazamaz.

`NEN-078`'in ölçümü girdidir. Karara bağlanacak üç soru:

1. **Alıcı yüzey.** `CFBundleDocumentTypes` (open-with) · `CFBundleURLTypes`
   (özel scheme) · positional argv — hangisi, kaçı birden. Bugün
   `platforms/macos/Resources/Info.plist` bunların **hiçbirini** taşımıyor ve
   `NenPlayerApp.swift`'in `AppDelegate`'inde `application(_:open:)` yok.
2. **Başlangıç pozisyonunun taşıyıcısı ve birimi.** Kontratın kendisi zaten
   kararlı: ADR-0042 yüklenirken verilen seek'in reddedilmeyip tutulacağını ve
   `FILE_LOADED` anında uygulanacağını söylüyor (`NEN-052`). Açık olan, o
   değerin handoff'tan **nasıl** geldiği.
3. **Handoff port mu?** `docs/architecture.md`'nin sınır kuralı "Stremio
   handoff"u `port + platform adapter` sınıfında sayıyor, ama aynı dosyanın
   port tablosunda böyle bir port **yok**. ADR bu tutarsızlığı kapatmalı: ya
   port tablosuna bir satır eklenir, ya sınır kuralı düzeltilir ve handoff
   kabuğun OS olayını çözüp mevcut `PlaybackSession::load` + seek'i çağırması
   olarak tanımlanır.

ADR ayrıca güvenlik tarafını **yeni politika icat etmeden** bağlamalı: gelen
her şey `security-policy.md` §2 gereği düşman girdidir ("handoff extras" orada
adıyla geçiyor) ve §3 kullanıcı/handoff medya URL'lerinin approved-host
listesiyle sınırlandırılmayacağını zaten söylüyor. Şema kapısı bugün
`nen_app::remote_evidence::validate_url`'de.

## Kapsam

- ADR `proposed` yazılır, kullanıcı onaylar, `accepted` olur
- Üç sorunun her biri numaralı bir Karar olur
- Elenen alternatifler ve eleme gerekçeleri yazılır
- Karara göre `docs/architecture.md`'nin port tablosu **veya** sınır kuralı
  düzeltilir

## YAPILMAYACAK

- Implementasyon → `NEN-080`
- Yeni HTTP/dosya güvenlik politikası icat etmek — politika
  `security-policy.md`'de, ADR yalnız onu handoff yüzeyine bağlar
- Android Intent tarafını karara bağlamak → M10 (`ACTION_VIEW`, ayrı ADR)
- Stremio'ya pozisyon döndürme sözleşmesi → M10

## Kanıt (DoD)

- [ ] ADR dosyası `docs/adr/00##-*.md`, `status: accepted`, kullanıcı onaylı
- [ ] Üç sorunun üçü de numaralı Karar olarak cevaplanmış
- [ ] `NEN-078`'in ölçüm raporuna atıf var; kararlar ölçülene dayanıyor
- [ ] `docs/architecture.md`'nin port/sınır tutarsızlığı kapanmış
- [ ] `bash scripts/check-docs.sh` çıkış 0

## Kanıt kaydı

<!-- done olurken doldurulacak -->
