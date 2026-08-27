---
adr: 0036
title: Video presentation metadata playback portundan geçer
status: proposed
milestone: M3
tasks: [NEN-063]
date: 2026-08-27
---

# ADR-0036 — Video presentation metadata playback portundan geçer

## Durum

`proposed`

## Bağlam

NEN-061'in çerçevesiz üst şeridi mockup'taki kısa kalite rozetine yer açıyor,
fakat kabuğun konuştuğu `PlaybackSession` bugün yalnız state, pozisyon, süre ve
track metadata'sı döndürüyor. Çözünürlük ile HDR/SDR bilgisini kabukta engine'e
doğrudan sorarak almak ADR-0026'nın core-owned session sınırını deler;
dosya adından çıkarmak ise güvenilmez bir tahmindir.

libmpv `video-params/dw`/`dh` ile aspect-correct display boyutunu ve
`video-params/gamma` ile transfer fonksiyonunu verir. Raw property adları ve
string değerleri engine ayrıntısıdır; core ve UI bunlarla dallanmamalıdır.

## Karar

Playback portu optional `VideoPresentationMetadata` sorgusu taşıyacaktır.
Metadata yalnız pozitif display boyutunu ve kapalı `DynamicRange` (`sdr`,
`hdr`, `unknown`) sınıfını içerir; raw engine string'i taşımaz. PQ ve HLG HDR,
bilinen SDR transferleri SDR, eksik veya tanınmayan transfer `unknown` olur.
Değişiklikler coalescing `VideoMetadataChanged` olayıyla bildirilir; shell
olayı aldığında ve `EventsLost` resync sırasında optional değeri yeniden okur.

## Gerekçe

Metadata motorun oynattığı gerçek video akışına aittir; media identity veya UI
tahmini değildir. Porttan geçirilmesi macOS, Android ve Apple adapter'larının
aynı ürün tipini üretmesini ve contract kitinin bu davranışı zorlamasını
sağlar. Optional sonuç, video olmayan/henüz çözümlenmeyen medya için uydurma bir
değer üretmez. Absolute değişiklik olayı position gibi coalescing olabilir:
ara değer değil, yalnız en yeni metadata önemlidir.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Shell libmpv'ye doğrudan sorsun | ADR-0026/0033 session ownership sınırını deler ve engine bilgisini UI'a taşır |
| Dosya adından `1080p`/`HDR` çıkar | Dosya adı beyanı yanlış veya eksik olabilir; oynatılan akışın gerçeği değildir |
| Raw `gamma` string'ini FFI'dan geçir | Engine'e özgü, değişebilir string üzerinde platform UI'larını dallandırır |
| Metadata zorunlu, yoksa hata | Audio-only, live veya henüz decode edilmemiş kaynaklarda görünür playback'i gereksiz yere başarısız kılar |

## Sonuçlar

**Olumlu:** Kalite rozeti gerçek oynatma metadata'sına dayanır; ham engine
ayrıntısı core ve UI'a sızmaz; gelecekteki adapter'lar aynı contract ile sınanır.

**Olumsuz / kabul edilen maliyet:** Playback portu, iki FFI yönü ve contract
kiti yeni tip/metot/olayla genişler. Bilinmeyen transfer SDR diye tahmin
edilmediği için bazı videolarda yalnız çözünürlük görünür.

**Geri dönüş maliyeti: orta.** FFI tip/metot/olayları platform binding'lerine
girer; kaldırmak tüm adapter ve shell'leri eşzamanlı değiştirmeyi gerektirir.

## İlgili task'lar

`NEN-061` · `NEN-063`

## Notlar

Görünen `720p/1080p/1440p/2160p` sınıfına dönüştürme portun değil, platform
presentation katmanının işidir; port yalnız aspect-correct boyutu taşır.
