---
adr: 0038
title: Video display geometry playback portundan geçer
status: accepted
milestone: M3
tasks: [NEN-068]
date: 2026-08-30
---

# ADR-0038 — Video display geometry playback portundan geçer

## Durum

`accepted`

## Bağlam

Ürün, tam ekran dışında da çerçevesiz bir yüzey istiyor: pencere açılan
medyanın kendi boyutunda açılmalı ve resize boyunca o en-boy oranını
korumalı, böylece üstte ve altta hiçbir zaman siyah bar oluşmamalı
(`NEN-068`).

Bunun için tek bir sayı gerekiyor — oynatılan akışın aspect-correct display
boyutu — ve bugün o sayı kabuğa hiçbir yoldan ulaşmıyor. Kabuk yalnız
`PlaybackSession` ile konuşuyor ([ADR-0033](0033-playback-session-ffi-surface.md));
oturum ise state, pozisyon, süre ve track metadata'sı döndürüyor. Boyutu
kabukta motora doğrudan sormak [ADR-0026](0026-playback-renderer-ownership.md)'nın
core-owned session sınırını deler.

Bu tam olarak [ADR-0036](0036-video-presentation-metadata.md)'nın konusuydu ve
o ADR `rejected`. Fakat **ret gerekçesi ürün kapsamıdır**, teknik yol değil:
kullanıcı 2026-08-29'da oynatıcı kromunda kalite rozeti istemedi, bu yüzden
çözünürlük/codec/HDR sunumu üründen çıktı ve `NEN-063` iptal edildi. Boyutun
porttan geçmesi gerektiği hiç çürütülmedi; ADR-0036 aynı gerekçelerle
"kabuk motora doğrudan sorsun" alternatifini de reddetmişti. Şimdi ihtiyaç
geri geldi — ama görsel bir rozet olarak değil, pencere geometrisinin girdisi
olarak ve **yalnız boyut** kadarıyla.

Karar verilmezse: pencere oranı ya dosya adından tahmin edilir, ya da kabuk
libmpv'ye doğrudan sorarak port sınırını deler. İkisi de M3'ün mimarisini
geri alır.

## Karar

### Karar 1 — Port aspect-correct display boyutunu taşır

`PlaybackEngine` (ve karşılığı olan `ShellEngine` / FFI yüzeyi) şu sorguyu
taşıyacaktır:

```rust
fn video_geometry(&self) -> Result<Option<VideoGeometry>, PlaybackError>;
```

`VideoGeometry` yalnız `width` ve `height` taşır; ikisi de sıfırdan büyüktür.
`None` "ortada gösterilecek video yok veya henüz çözümlenmedi" demektir —
audio-only medya, canlı akışın ilk anı, decode öncesi. Bu bir hata değildir.

Yeni bir `Capability` **eklenmez**. Her adapter `None` döndürebildiği için
capability her zaman doğru bir dal üretirdi; `nen-ports`'un kendi capability
kuralı ("her zaman doğru olan şey capability değildir") bunu yasaklıyor.

### Karar 2 — Değişiklik coalescing olayla bildirilir, değer olayda taşınmaz

Geometri değiştiğinde port `PlaybackEvent::VideoGeometryChanged` üretir.
Olay **payload taşımaz**. Kabuk olayı aldığında ve `EventsLost` resync'inde
([ADR-0011](0011-playback-port-contract.md) Karar 1) değeri yeniden okur.

Ara değer önemli değildir, yalnız en yeni geometri önemlidir; bu yüzden olay
`PositionChanged` gibi coalescing edilebilir.

### Karar 3 — Port yalnız boyut taşır

Codec, fps, dynamic range ve rotation sınıfı porttan geçmez. Taşınan sayı,
pixel aspect ratio ve rotation **uygulanmış** display boyutudur; bunu üretmek
adapter'ın yükümlülüğüdür, core yorumlamaz.

### Karar 4 — Pencere geometrisi platform sunum katmanının işidir

Açılış boyutu, en-boy oranı kilidi, minimum boyut türetmesi ve ekrana clamp
portun kararı değildir. Port yalnız sayıyı taşır; ne yapılacağına platform
sunum katmanı karar verir.

### Karar 5 — Display boyutu özel veri değildir

`width`/`height` K23 listesindeki hiçbir sınıfa girmez; loglanabilir ve
`Debug` çıktısında görünebilir.

## Gerekçe

Boyut, motorun **oynattığı gerçek akışa** aittir; media identity ya da UI
tahmini değildir. libmpv `video-out-params/dw`/`dh` ile VO filtreleri ve
rotation uygulandıktan sonraki display boyutunu verir (`video-params/dw`/`dh`
fallback'tir) ve `MPV_EVENT_VIDEO_RECONFIG` bunun doğal tetikleyicisidir —
yani adapter tarafında uydurma bir hesap yok, motorun zaten hesapladığı sayı
var.

Porttan geçmesi macOS, Android ve gelecekteki Apple adapter'larının aynı
ürün tipini üretmesini ve contract kitinin bu davranışı zorlamasını sağlar.
Optional sonuç, video olmayan veya henüz çözümlenmemiş medya için uydurma bir
değer üretmez.

Kapsamın `ADR-0036`'nın önerdiğinden dar tutulmasının nedeni ölçülü olmaktır:
ürünün bugün ihtiyacı olan tek şey boyut. Dynamic range ve codec, ihtiyaç
doğduğunda kendi ADR'siyle eklenir.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Kabuk libmpv'ye doğrudan sorsun | ADR-0026/0033 session ownership sınırını deler ve engine bilgisini UI'a taşır (ADR-0036 ile aynı gerekçe) |
| Boyutu olayın payload'ında taşı | `EventsLost` sonrası okuma yolu zaten şart; iki kaynak aynı gerçeği çoğaltır ve bayat payload riski doğurur |
| Oranı dosya adından veya konteyner beyanından çıkar | Anamorphic ve döndürülmüş akışlarda yanlış; oynatılan akışın gerçeği değil |
| `Option` yerine zorunlu değer, yoksa hata | Audio-only, canlı ve henüz decode edilmemiş kaynaklarda görünür playback'i gereksiz yere başarısız kılar |
| Yeni bir `Capability` ekle | Her adapter `None` döndürebilir; capability her zaman doğru bir dal üretir |
| ADR-0036'yı `accepted`'a çevir | O ADR reddedilmiş bir **ürün** kapsamını (kalite rozeti, dynamic range) taşıyor; tarihsel kayıt olarak korunur, kapsamı geri getirilmez |

## Sonuçlar

**Olumlu:** Pencere oranı gerçek oynatma akışından gelir, dosya adı tahmininden
değil. Ham engine ayrıntısı core ve UI'a sızmaz. Gelecekteki adapter'lar aynı
contract ile sınanır. Kapsam tek bir tipe indiği için FFI yüzeyi ADR-0036'nın
önerdiğinden dar kalır.

**Olumsuz / kabul edilen maliyet:** Playback portu, iki FFI yönü ve contract
kiti yeni bir tip, bir metot ve bir olayla genişler. Audio-only medyada pencere
oranı kilitlenmez — kabuk o durumda serbest resize'a düşer.

**Geri dönüş maliyeti: orta.** FFI tip/metot/olayları platform binding'lerine
girer; kaldırmak tüm adapter ve shell'leri eşzamanlı değiştirmeyi gerektirir.

## İlgili task'lar

`NEN-068`

## Notlar

[ADR-0036](0036-video-presentation-metadata.md) bu ADR'nin öncülüdür ve
`rejected` kalır. Aradaki fark kapsamdır: ADR-0036 çözünürlük **ve** dynamic
range'i bir kalite rozeti için taşıyordu; bu ADR yalnız display boyutunu,
pencere geometrisi için taşır. `NEN-063` (kalite rozeti) `canceled` durumda
kalır ve bu ADR onu geri getirmez.

Görünen `720p`/`1080p` sınıfına dönüştürme hâlâ portun işi değildir; bu ADR
zaten böyle bir sınıf üretmiyor.
