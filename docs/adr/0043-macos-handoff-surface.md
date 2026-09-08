---
adr: 0043
title: macOS handoff alıcı yüzeyi — argv + open-with + custom scheme
status: accepted
milestone: M4
tasks: [NEN-079, NEN-080, NEN-081, NEN-082, NEN-083]
date: 2026-09-08
---

# ADR-0043 — macOS handoff alıcı yüzeyi

## Durum

`accepted`

## Bağlam

`docs/product-spec.md` §5: *"macOS: external-player launcher veya open-with
handoff · positional file/http/https · optional start position · argüman ve
medya URL'sini loglamama."* Bugün bu yüzeyin **hiçbiri** yok:
`platforms/macos/Resources/Info.plist` ne `CFBundleDocumentTypes` ne
`CFBundleURLTypes` taşıyor, `NenPlayerApp.swift`'in `AppDelegate`'i
`application(_:open:)` uygulamıyor, `CommandLine` hiç okunmuyor.

`docs/architecture.md`'nin sınır kuralı "Stremio handoff"u `port + platform
adapter` sınıfına koyuyor ama port tablosunda (`docs/architecture.md` →
"Portlar") böyle bir port yok — bu ADR bu tutarsızlığı da kapatmalıdır
(`NEN-079` Kapsam maddesi 4).

`NEN-078` (`evidence/M4/NEN-078-measurement.md`) gerçek Stremio 5.1.26 ile
mekanizmayı ölçtü ve girdi olarak şu bulguları verdi:

- **Bulgu 4:** Stremio custom URL scheme kullanmıyor; hedef oynatıcıyı
  doğrudan, kendi CLI biçiminde argv ile başlatıyor (`--start=`/`--no-terminal`
  mpv için, `--start-time=`/`--no-video-title-show` VLC için).
- **Bulgu 6:** sabitlenen şey uygulama kimliği değil, **CLI sözleşmesi** —
  "MPV" hedefinin gerçek bundle kimliği farklıydı, yalnız mpv'nin argv biçimini
  karşıladığı için orada durdu.
- **Bulgu 1 + 3:** Stremio'nun harici oynatıcı ayarı (`Etkisizleştirildi · MPV
  · IINA · Infuse · M3U Playlist`) kapalı bir liste ve oynatıcı ekranının
  `...` menüsü ayardan bağımsız, sabit iki seçenek taşıyor (VLC/MPV) — Nen
  Player bu listelerin hiçbirine giremez.
- **Bulgu 5:** iki bağımsız ölçümde de başlangıç bayrağı **0** geldi; pozisyon
  bu mekanizmadan hiç taşınmıyor.
- **Bulgu 7:** metadata için ayrı bir alan yok; yalnız URL'nin dosya adı
  segmenti zımni ipucu taşıyor.
- **Bulgu 9:** bu makinede gerçek Developer ID imzası yok
  (`security find-identity` → 0 kimlik, `NEN-043`'ün de kaydettiği durum); ad-hoc
  imzalı bir uygulama `spctl -a -vv` altında `rejected` ve hiçbir göndericiye
  scheme handler olarak görünmüyor (kontrol grubu: düzgün imzalı `claude://`
  normal çözülüyor). Argv/doküman yollarında bu kısıt gözlenmedi.
- Statik inceleme: Stremio'nun embedded webview'ı harici link tıklamasında ham
  `href`'i (`scheme://https://...` şeklini bozmadan) shell'e veriyor — bu,
  ölçülen "harici oynatıcıda oynat" akışından **ayrı** bir yol.

Ölçüm kesin bir öneri üretmedi, adayları karşılaştırdı. Karar burada verilir.

## Karar

**Karar 1 — Alıcı yüzey: argv + open-with + custom scheme, üçü birden.**

- `CFBundleDocumentTypes`: `public.movie` ve `public.audiovisual-content`
  UTI'lerini `LSHandlerRank: Alternate` (varsayılan uygulamayı çalmaz), rol
  `Viewer` ile bildirir. Bu, ölçülemeyen ama product-spec §5'in adıyla istediği
  "open-with handoff" yolunu açar.
- `CFBundleURLTypes`: `nenplayer` scheme'i bildirir. Biçim, statik incelemede
  görülen webview yolunun koruduğu önek şeklini takip eder:
  `nenplayer://<medya URL'si birebir>`.
- argv: positional locator (dosya yolu veya `http`/`https` URL'si) + tanınan
  bayraklar. **Tanınmayan bayrak sessizce yok sayılır** — Bulgu 4'ün
  `--no-terminal`/`--no-video-title-show`'u tam olarak budur; bunları hata
  saymak ölçülen gönderici davranışını kırar.
- Üçü de kabukta aynı iç yapıya (`HandoffRequest`) çözülür; `NEN-080` tek bir
  çağrı yeri yazar.

**Karar 2 — Başlangıç pozisyonunun taşıyıcısı ve birimi.**

- argv: `--start=<sn>` (mpv yazımı) · `--start-time=<sn>` (VLC yazımı) ·
  `--start-position=<sn>` (Nen Player'ın kendi kanonik adı) — üçü eşdeğer
  kabul edilir; hem `--ad=değer` hem `--ad değer` biçimi okunur.
- scheme: locator'ın sonunda opsiyonel `#t=<sn>` fragment'i (Media Fragments
  URI konvansiyonu).
- open-with: pozisyon taşımaz — LaunchServices doküman-açma çağrısında bunun
  yeri yok.
- Sınırda birim **saniye** (ondalık serbest), core'a girince **milisaniye** —
  ADR-0042'nin `deferredSeekMs` kontratıyla aynı birim.
- `0` semantik olarak "pozisyon yok" ile aynıdır; ayrı bir dal yazılmaz —
  Bulgu 5'in ölçtüğü gerçek Stremio davranışı zaten budur.
- Geçersiz değer (negatif, sayı olmayan, taşmalı, medyanın süresini aşan)
  **sessizce düşer**; medya yine baştan açılır, hata yüzeyi tetiklenmez.
- Uygulama yolu yeni değildir: ADR-0042'nin "yüklenirken verilen seek
  reddedilmez, tutulur, `FILE_LOADED` anında uygulanır" kontratı (`NEN-052`,
  ölçülen yükleme penceresi 2,5–12 ms) aynen kullanılır — handoff pozisyonu
  `PlaybackSession::load` ile birlikte, aynı erteleme mekanizmasından geçer.

**Karar 3 — Handoff'un kendi portu yoktur.**

Handoff **içeri doğru (inbound)** bir OS olayıdır: core onu hiç çağırmaz, OS
kabuğa iter. Kabuk olayı ham haliyle toplar; core (`nen-app`) çözer; sonuç
mevcut `PlaybackSession::load` (`core/crates/nen-app/src/session.rs:150`) ve
ADR-0042'nin seek'i olarak uygulanır. `docs/architecture.md`'nin sınır
kuralından "Stremio handoff" — ve aynı satırdaki, yine karşılığı port
tablosunda olmayan "lifecycle" — çıkarılır; yerine inbound OS olaylarının
port sayılmadığını söyleyen açık bir cümle konur (bu ADR'nin yürürlüğe
girmesiyle `docs/architecture.md` bu ADR'ye referansla güncellenir).

**Karar 4 — Ayrıştırma core'da yapılır, kabuk yalnız ham olayı taşır.**

Kabuk (Swift) `argv` dizisini / URL string'ini olduğu gibi FFI üzerinden
`nen-app`'e geçirir. Şema kapısı, bayrak/fragment ayrıştırması, pozisyon
dönüşümü ve tipli hata üretimi core'dadır. Gerekçe: "politika core'a,
implementasyon adaya aittir" (`docs/architecture.md`); M10'da Android Intent
tarafı aynı ayrıştırma kodunu paylaşabilir; `NEN-083`'ün K23 guard denetimi
tek bir yerde (core) durur, iki platformda tekrarlanmaz.

Yeni bir güvenlik politikası icat edilmez: `http`/`https` locator'ı için
mevcut `nen_app::remote_evidence::validate_url` (`remote_evidence.rs:230`)
şema kapısından geçer; `file://` locator'ı yerel yola çevrilip `⌘O`'nun bugün
kullandığı yoldan geçer. Desteklenmeyen şema (`ftp:`, `javascript:` vb.) ve
bozuk argüman tipli hata döner; `unwrap`/`expect` kullanılmaz
(`security-policy.md` §2).

**Karar 5 — Handoff metadata'sının biçimi: ayrı bir alan yoktur.**

Bulgu 7: argv'da başlık/sezon/bölüm/canonical ID için ayrı bir bayrak veya
ortam değişkeni gözlenmedi; bilgi yalnız URL'nin kendi path segmentlerinde
zımni olarak var. M4 bu yüzden yeni bir yapılandırılmış metadata kanalı
açmaz — handoff'un taşıdığı "metadata" **locator'ın kendisidir** ve
ADR-0009'un zaten tanımladığı kanıt katmanına (uzak URL path segmentleri,
sunucunun beyan ettiği ad) bir girdi olarak eklenir. `NEN-082` bu bağlamayı
uygular; "metadata yoksa akış bozulmuyor" kriteri bu kararla zaten varsayılan
durumdur — çünkü ayrı bir metadata alanı hiç yoktur, kaybolacak bir şey de
yoktur.

## Gerekçe

Karar 1'in üç yüzeyi birden seçmesinin gerekçesi ölçülen gönderici
davranışının **tekil olmaması**: Stremio'nun `...` menüsü argv kullanıyor
(Bulgu 4), ama product-spec §5 ayrıca "open-with handoff" istiyor ve statik
inceleme farklı bir göndericinin (embedded link) scheme-şekilli bir string
ürettiğini gösteriyor. Üç yüzeyin maliyeti düşük (üçü de aynı iç yapıya
düşüyor) ve hiçbiri diğerini dışlamıyor.

Karar 2'nin mpv/VLC yazımlarını da kabul etmesi taklit değildir: Bulgu 6
sabitlenenin **CLI sözleşmesi** olduğunu, uygulama kimliği olmadığını
gösteriyor. Nen Player kendi bundle kimliğiyle, yalnız üç eşdeğer bayrak adını
tanıyarak, bu sözleşmeyi konuşan **her** göndericiye (yalnız Stremio değil)
açılıyor — VLC.app'in veya bir Stremio yardımcı uygulamasının yerine geçme
girişimi yok.

Karar 3 mevcut mimari dille tutarlı: `HttpClient` ve `Persistence` gibi
tablodaki her port core'un **çağırdığı** bir yeteneği temsil ediyor; handoff
tam tersi yönde, OS'in kabuğa ittiği bir olay. Yeni bir port açmak bu asimetriyi
gizlerdi.

Karar 4 M10'un (Android) aynı politikayı yeniden yazmasını önlüyor ve K23
guard'ının (Bulgu listesindeki loglama yasağı) tek merkezde durmasını
sağlıyor — `guard_playback_debug.rs` deseniyle aynı yerde.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Yalnız argv | product-spec §5'in "open-with handoff" maddesini karşılamaz; open-with, sürükle-bırak ve varsayılan-uygulama senaryoları çalışmaz |
| Yalnız custom scheme (`nenplayer://`) | Bulgu 4: Stremio'nun ölçülen gerçek yolu (`...` menüsü) scheme kullanmıyor — bu yüzey tek başına ölçülen göndericiyi karşılamazdı |
| Scheme'i S11'e (Developer ID) ertelemek | Bulgu 9'un kısıtı yalnız bu makineye/bu imza durumuna özgü — geçici bir donanım/hesap kısıtı yüzünden kalıcı bir mimari karar ertelenmez; `NEN-080` bu maddeyi ayrı test edilebilir olarak işaretler |
| `MediaHandoff` portu eklemek | Handoff core'un çağırdığı bir yetenek değil, OS'in kabuğa ittiği bir olay — port tablosundaki hiçbir satırın örnekleri bu yönde değil; yeni bir "iters, çağırmaz" port kategorisi mevcut modeli karmaşıklaştırırdı |
| VLC/MPV yuvasına bundle kimliği taklidiyle girmek | Kullanıcının gerçek VLC/MPV'sini bozar, Stremio güncellemesiyle kırılgan olur, göndericiye yanlış uygulama kimliği bildirir — güvenlik ve bakım maliyeti kazancından yüksek |
| Ayrıştırmayı Swift kabuğunda yapmak | Aynı düşman-girdi politikası M10'da Kotlin'de ikinci kez yazılır, K23 denetimi iki platformda ayrışma riskiyle tekrarlanır |
| Handoff için ayrı, yapılandırılmış bir metadata alanı icat etmek | Bulgu 7: hiçbir gönderici böyle bir alan göndermiyor; icat edilen alan hiçbir zaman dolmayan, test edilemeyen bir sözleşme olurdu |

## Sonuçlar

**Olumlu:** `NEN-080` üç yüzeyi tek bir iç yapıya indirger ve kod
yazabilir; `docs/architecture.md`'nin port/sınır tutarsızlığı kapanır; M10
(Android) aynı ayrıştırma politikasını core'dan miras alır.

**Olumsuz / kabul edilen maliyet:**

- Stremio'nun kendi ayar listesi (Bulgu 1: `Etkisizleştirildi · MPV · IINA ·
  Infuse · M3U Playlist`) ve oynatıcı ekranının `...` menüsü (Bulgu 3: sabit
  VLC/MPV) **kapalı kümeler** — Nen Player bunlara giremez, çünkü bu ADR
  taklidi reddediyor. `NEN-084`'ün kabul koşusu bu yüzden Stremio'nun bu iki
  yüzeyinden değil, kabul edilen sözleşmeyi konuşan bir gönderici üzerinden
  yapılacaktır; M4'ün "Stremio'dan açılan medya doğru pozisyondan oynuyor"
  çıkış kriteri bu ADR'de yeniden yazılmaz, `NEN-084`'e bir uygulama notu
  olarak bırakılır.
- `nenplayer://` scheme'i bu makinede **kanıtlanamaz**: Bulgu 9 gereği ad-hoc
  imzalı bir uygulama hiçbir göndericiye scheme handler olarak görünmüyor.
  Bu geçici bir kısıt (roadmap **S11** — Developer ID alındığında kalkar);
  `NEN-080`'in DoD'unda scheme yolu argv/open-with'ten ayrı, "bu makinede
  doğrulanamadı" notuyla işaretlenir.
- Argv'daki üç eşdeğer bayrak adı (`--start`, `--start-time`,
  `--start-position`) küçük bir ayrıştırma yüzeyi genişlemesidir; kapalı
  kümedir, yenisi yalnız yeni bir ölçümle eklenir.

**Geri dönüş maliyeti:** orta. Üç yüzeyden biri iptal edilirse (örn. scheme
hiç kullanılmadığı görülürse) `Info.plist`'ten tek girdi silinir; ayrıştırma
core'da tek yerde olduğu için kabuk tarafında değişiklik gerekmez. Port
kararının (Karar 3) geri dönüşü daha pahalı: bir port eklenmesi gerekirse
`docs/architecture.md`'nin sınır kuralı ve port tablosu birlikte değişir ve
yeni bir ADR gerekir.

## İlgili task'lar

`NEN-079`, `NEN-080`, `NEN-081`, `NEN-082`, `NEN-083`

## Notlar

Karar sonrası ortaya çıkan gözlemler buraya eklenir. Karar değişiyorsa bu ADR
`superseded` yapılır ve **yeni bir ADR** yazılır — eski ADR düzenlenmez.
