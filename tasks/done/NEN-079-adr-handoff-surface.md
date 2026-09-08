---
id: NEN-079
title: Decide the macOS handoff receiving surface
milestone: M4
size: S
state: done
closed: 2026-09-08
depends_on: [NEN-078]
blocks: [NEN-080]
adr: [43]
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

- [x] ADR dosyası `docs/adr/00##-*.md`, `status: accepted`, kullanıcı onaylı
- [x] Üç sorunun üçü de numaralı Karar olarak cevaplanmış
- [x] `NEN-078`'in ölçüm raporuna atıf var; kararlar ölçülene dayanıyor
- [x] `docs/architecture.md`'nin port/sınır tutarsızlığı kapanmış
- [x] `bash scripts/check-docs.sh` çıkış 0

## Kanıt kaydı

**ADR:** [`docs/adr/0043-macos-handoff-surface.md`](../../docs/adr/0043-macos-handoff-surface.md)
— `status: accepted`, `date: 2026-09-08`, kullanıcı onayı bu oturumda dört ayrı
soruda alındı (alıcı yüzey · erişilebilirlik/bayrak eşdeğerliği · port kararı ·
ayrıştırma sahibi).

**Üç sorunun cevabı — beş numaralı Karar'a dağıldı:**

1. **Alıcı yüzey (Karar 1):** üçü birden — `CFBundleDocumentTypes`
   (`public.movie`/`public.audiovisual-content`, `LSHandlerRank: Alternate`),
   `CFBundleURLTypes` (`nenplayer` scheme'i) ve argv (positional locator +
   tanınan bayraklar, tanınmayan bayrak sessizce yok sayılır). Gerekçe: ölçülen
   gönderici davranışı tekil değil — `NEN-078` Bulgu 4'ün argv yolu ile
   product-spec §5'in "open-with" maddesi ve statik incelemenin scheme-şekilli
   webview linki aynı anda karşılanmalı.
2. **Başlangıç pozisyonu (Karar 2):** argv'de üç eşdeğer bayrak
   (`--start=`/`--start-time=`/`--start-position=`, `NEN-078` Bulgu 6'nın
   gösterdiği "sabitlenen CLI sözleşmesi, uygulama kimliği değil" bulgusuna
   dayanarak — taklit değil), scheme'de `#t=` fragment'i, birim sınırda saniye
   → core'da milisaniye (ADR-0042'nin `deferredSeekMs` kontratıyla aynı).
   Geçersiz/eksik değer sessizce düşer, medya baştan açılır.
3. **Port mu? (Karar 3):** hayır — handoff **inbound** bir OS olayı sayılır,
   core çağırmaz, OS kabuğa iter; sonuç mevcut `PlaybackSession::load` + seek
   çağrısına düşer. `docs/architecture.md`'nin sınır kuralından "Stremio
   handoff" ve "lifecycle" çıkarıldı, yerine "Inbound OS olayları port
   değildir" alt bölümü eklendi (ADR-0043 Karar 3'e referansla).

Ayrıca Karar 4 (ayrıştırma core'da, kabuk ham olayı taşır) ve Karar 5 (handoff
metadata'sının biçimi: ayrı alan yok, locator'ın kendisi ADR-0009'un kanıt
katmanına girer) `NEN-080`/`NEN-082`'nin ihtiyacı olarak eklendi — kapsam
gereği yalnız üç soru zorunluydu, beş karar da `NEN-078`'in ölçümüne dayanıyor.

**Dürüstçe kaydedilen iki sınır (ADR'nin "Sonuçlar" bölümü):** Stremio'nun
kendi ayar listesi ve `...` menüsü kapalı küme — Nen Player bunlara giremiyor,
taklit reddedildiği için; `nenplayer://` scheme'i bu makinede Developer ID
eksikliği yüzünden (`NEN-078` Bulgu 9) kanıtlanamıyor, roadmap S11'e bağlı.

**Değişen dosyalar:** `docs/adr/0043-macos-handoff-surface.md` (yeni),
`docs/architecture.md` (sınır kuralı + yeni alt bölüm), `docs/adr/README.md`
(0043 satırı eklendi, planlanan 0014 Android'e daraltılıp M10'a taşındı),
`docs/milestones/M4-stremio-macos.md` (açık mimari sorusu paragrafı karar ile
güncellendi).

**Kod değişmedi** — bu bir karar/doküman task'ı (`docs/testing-strategy.md` →
"Kanıt formatı": doküman → link/tutarlılık kontrolü). `bash
scripts/check-docs.sh` çıkış **0**. Rust workspace ve Swift paketine
dokunulmadı, bu yüzden koşulmadı.
