# NEN-068 — libmpv display geometry ölçümü

Tarih: 2026-08-31
Makine: Apple Silicon · macOS 27.0
libmpv: 2.5.0 (mpv 0.41.0_8, Homebrew)

Ölçüm, ürün adapter'ının kullandığı seçeneklerle (`config=no`, `terminal=no`,
`idle=yes`, `pause=yes`, `keep-open=yes`, `sid=no`, `sub-auto=no`) ve
**headless** (`vo=null`, `ao=null`) kurulmuş bir `mpv_handle` üzerinde yapıldı.
Headless olması bilinçli: contract kiti `MPVEngineFactory` ile engine'i
`videoView: nil` kurar, yani kitin gördüğü engine budur.

## Soru 1 — `MPV_EVENT_VIDEO_RECONFIG` headless de üretiliyor mu?

ADR-0038 Karar 2 olayı bu tetikleyiciye bağlıyor. Eğer `vo=null` altında hiç
üretilmeseydi adapter'ın ek olarak `dwidth`/`dheight` property'lerini gözlemesi
gerekecekti.

**Üretiliyor.** Videolu her fixture'da, `MPV_EVENT_FILE_LOADED`'dan sonra
**iki kez**:

```
[contract] FILE_LOADED
[contract] VIDEO_RECONFIG #1
[contract] VIDEO_RECONFIG #2
[contract] prop dwidth = 160
[contract] prop dheight = 90
```

İki üretim, olayın coalescing olmasının pratikteki gerekçesidir: ara değer
önemli değildir, yalnız en yeni geometri önemlidir (ADR-0038 Karar 2). Kuyruk
ikisini tek slota indirir ve kabuk bir kez okur.

`dwidth` ve `dheight` ilk olarak **`<unavailable>`** olarak bildiriliyor ve
sayısal değerlerini ancak `VIDEO_RECONFIG`'lerden sonra alıyor. Bu yüzden
contract kitinin adımı olayı bekledikten sonra değeri **yoklar**, tek seferde
okumaz: ilk reconfig'in hemen ardından okunan değer henüz çözülmemiş olabilir.

## Soru 2 — Taşınan sayı gerçekten display boyutu mu?

ADR-0038 Karar 3, taşınan sayının pixel aspect ratio ve rotation **uygulanmış**
boyut olduğunu, bunu üretmenin adapter'ın yükümlülüğü olduğunu söylüyor.
`anamorphic-clip.mkv` bunun sınandığı fixture'dır: 720x576 saklanır, SAR 64:45
ile 1024x576 gösterilir.

| Fixture | Saklanan | `video-out-params/dw`x`dh` | `video-params/dw`x`dh` | reconfig |
|---|---|---|---|---|
| `contract-clip.mkv` | 160x90 | **160x90** | 160x90 | 2 |
| `aspect-4x3-clip.mkv` | 160x120 | **160x120** | 160x120 | 2 |
| `aspect-cinema-clip.mkv` | 382x160 | **382x160** | 382x160 | 2 |
| `anamorphic-clip.mkv` | 720x576 | **1024x576** | 1024x576 | 2 |
| `audio-only-clip.mka` | — | **(yok)** | (yok) | **0** |

Anamorphic satırı iddiayı kanıtlıyor: libmpv düzeltilmiş boyutu veriyor, yani
adapter'ın kendi hesabı yok — ADR-0038'in "motorun zaten hesapladığı sayı"
gerekçesi doğru.

## Soru 3 — Videosuz medyada ne oluyor?

`audio-only-clip.mka`'da iki property de okunamıyor **ve hiç
`MPV_EVENT_VIDEO_RECONFIG` üretilmiyor**. Yani `None` bir hata değil bir
durumdur (ADR-0038 Karar 1) ve "değer `None` **ve** hiç olay yok" contract
dalı uydurma değil, gerçek engine'in davranışıdır.

## Sonuç

- Adapter `MPV_EVENT_VIDEO_RECONFIG`'i doğrudan porta `VideoGeometryChanged`
  olarak çıkarır; `dwidth`/`dheight` gözlemine gerek yok.
- `video-out-params/dw`/`dh` birincil, `video-params/dw`/`dh` fallback'tir;
  bu fixture'larda ikisi de aynı yanıtı veriyor.
- Videosuz medyada iki property de okunamaz → `None`.
