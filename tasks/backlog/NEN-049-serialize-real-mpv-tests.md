---
id: NEN-049
title: Serialize real libmpv platform tests
milestone: M3
size: S
state: backlog
depends_on: [NEN-022, NEN-045, NEN-051]
blocks: []
adr: [12, 33]
---

# NEN-049 — Serialize real libmpv platform tests

## Sonuç

`scripts/test-macos.sh` içindeki gerçek libmpv testleri birbirinin zamanlamasını
bozmadan deterministik geçer; ürünün seek kontratı gevşetilmez.

## Kapsam

- Swift Testing'in tam paket koşusunda gerçek libmpv suite'lerini aynı anda
  çalıştırmaması için en dar test-runner veya suite izolasyonu
- `aSeekIsAnsweredThroughTheSession` testinin `SeekCompleted` içindeki gerçek
  konumu sınamaya devam etmesi; toleransı genişletmeme ve `0 ms`'yi kabul etmeme
- Seri/paralel farkının tekrarlı koşuyla kanıtlanması

## YAPILMAYACAK

- Playback adapter davranışını yalnız testi yeşile çevirmek için değiştirmek
- Seek UI uzlaştırması; transport sıçraması ayrı ürün işi
- Platform testlerini atlamak veya contract kitini daraltmak

## Neden ayrı task

NEN-046 kapanışında tam paralel macOS paketi iki ardışık koşuda aynı şekilde
kırmızı oldu: `aSeekIsAnsweredThroughTheSession`, beklenen `12_000 ms` yerine
`SeekCompleted(0 ms)` gördü. Test tek başına **5/5**, tüm paket açıkça
`--no-parallel` ile **31/31** geçti. NEN-046 yalnız pencere yaşam döngüsüdür;
gerçek motor testlerinin runner izolasyonu bu kapsama karıştırılmaz.

## Güncelleme (2026-08-26) — teşhis değişti

`NEN-051` açılırken kök sebep ölçüldü ve bu task'ın çıkış noktası olan
"paralel koşuda kırmızı" gözlemi **test izolasyonu sorunu değil**: mpv'nin
yüklemeden sonra yayınladığı `playback-restart`, `FILE_LOADED`'dan sonra
geldiği için `.ready` görülür görülmez verilen bir seek'i `0 ms` ile
cevaplıyordu. Paralellik yalnız o pencereyi genişletiyordu.

`NEN-051` kapandıktan sonra ölçüm yapıldı ve **ayırt edici çıkmadı**:
`bash scripts/test-macos.sh` düzeltmeden **önce de sonra da** art arda 5/5
yeşil. Yani bu makinede oran karşılaştırması bu task'ın gerekçesini ne
doğruluyor ne çürütüyor; DoD #1 (art arda ≥ 3 çıkış 0) hiçbir şey
değiştirilmeden zaten sağlanıyor ve DoD #3'ün istediği "değişiklik öncesi
paralel kırmızı" kaydı **üretilemedi**.

Bildirilen semptomun mekanizması `NEN-051`'de kaldırıldı ve deterministik
negatif kontrolle kanıtlandı. Bu task'ın kalan gerekçesi — gerçek libmpv
suite'lerinin runner izolasyonu — bağımsız bir kırmızı gözlemi olmadan
doğrulanamaz. **Karar kullanıcıya ait:** yeni bir kırmızı gözlenene kadar
beklemek ya da task'ı kapatmak. O gözlem gelmeden implementasyona
başlanmamalıdır.

### Gözlem — 2026-08-27, `NEN-025` sırasında

Aranan "bağımsız kırmızı" **bir kez gözlendi**, fakat beklenen testte değil.
`NEN-025`'in ilk paket koşusunda paralel modda `transientMessageExpires`
kırmızı geldi: 20 ms'lik geçici bildirim 300 ms'lik beklemeden sonra hâlâ
temizlenmemişti. O koşuda derleme de aynı anda sürüyordu.

Aynı ağaçta hemen ardından:

| Mod | Sonuç |
|---|---|
| yalnız o test | 1/1 yeşil |
| seri, tam paket | 3/3 yeşil (56 test) |
| paralel, tam paket (derleme sıcakken) | 4/4 yeşil |

Yani kırmızı **yükle ilişkili**, testle değil: ana aktörü meşgul eden bir
koşuda `Task.sleep`'e dayanan bir bekleyiş aç kalıyor.
`aSeekIsAnsweredThroughTheSession` değil, ama **sınıf aynı** — bu task'ın
gerekçesindeki "paralellik sebep değil, dar bir pencereyi genişleten koşul"
ifadesiyle uyumlu. Tek gözlem; oran ölçümü yapılmadı.

## Kanıt (DoD)

- [ ] `bash scripts/test-macos.sh` art arda en az 3 kez çıkış 0
- [ ] `aSeekIsAnsweredThroughTheSession` hâlâ `12_000 ± 100 ms` bekliyor
- [ ] Değişiklik öncesi paralel kırmızı ve sonrası tekrarlı yeşil sayıları
      bağlamıyla kaydedilmiş
- [ ] Ürün kaynak kodunda değişiklik yok

## Kanıt kaydı

<!-- done olurken doldurulacak -->
