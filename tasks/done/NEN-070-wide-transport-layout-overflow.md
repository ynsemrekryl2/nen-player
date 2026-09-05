---
id: NEN-070
title: Keep transport controls inside wide windows
milestone: M3
size: S
state: done
depends_on: [NEN-067, NEN-068]
blocks: []
adr: [31, 38]
---

# NEN-070 — Geniş pencerede transport kontrollerini içeride tut

## Sonuç

Ekran genişliğine açılan videolarda oynat/duraklat ve tam ekran dahil bütün
transport kontrolleri pencere sınırları içinde, mevcut kenar boşluklarıyla
kalır.

## Kapsam

- Seek slider'ın geniş HStack içinde sabit kontrolleri dışarı iten yerleşim
  önceliğini düzeltmek.
- 693 pt minimum ve 1470 pt ekran genişliğinde ilk/son düğme frame'lerini
  gerçek SwiftUI yerleşimiyle sınamak.
- Kısa ve bir saatten uzun zaman etiketlerinde tek satırlı transportu korumak.

## YAPILMAYACAK

- Transportu ikinci satıra bölmek veya 57 pt yüksekliğini değiştirmek.
- Playback, pencere geometrisi, FFI ya da medya metadata sözleşmesini değiştirmek.
- Kullanıcı medyasının özel yolunu, adını veya görüntüsünü kanıta kaydetmek.

## Kanıt (DoD)

- [x] 693 pt ve 1470 pt genişliklerde ilk/son düğmeler pencere sınırları içinde.
- [x] 95 dakikalık süre ve bir saat eşiği taşma üretmiyor; seek en az 76 pt.
- [x] Bildirilen yerel medyada play/pause ile tam ekran düğmesi görünür ve tıklanabilir.
- [x] `bash scripts/test-macos.sh`, uygulama build'i, strict codesign,
      `bash scripts/test.sh` ve doküman kapıları yeşil.

## Kanıt kaydı

Seek slider'ın `.layoutPriority(1)` önceliği kaldırıldı; geniş satırda kalan
alanı doğal olarak alırken sabit uç kontrollerini artık dışarı itmiyor.

Gerçek yerleşim matrisi ve negatif kontrol `TransportControlsLayoutTests`
suite'inde yeşil. Tam macOS paketi **151 test / 16 suite / 0 failure**;
uygulama build'i, strict codesign ve iki shell test dosyası geçti. Bildirilen
özel yerel medyada 1470 pt pencerede play `22…56`, tam ekran `1418…1448`,
seek 880 pt ölçüldü. Ayrıntı: `evidence/M3/NEN-070-checklist.md`.
