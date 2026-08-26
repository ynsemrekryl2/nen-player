---
id: NEN-052
title: Decide and pin what a seek during loading does
milestone: M3
size: S
state: backlog
depends_on: [NEN-051]
blocks: []
adr: [11]
---

# NEN-052 — Decide and pin what a seek during loading does

## Sonuç

Medya yüklenirken verilen bir seek'in ne yaptığı **kararlaştırılmış** ve
contract kitinde yazılıdır; fake ile gerçek adapter aynı cevabı verir.

## Kapsam

- Portun `.buffering` durumundaki seek için ne vaat ettiğinin netleştirilmesi:
  tipli ret mi, yoksa yükleme bitince uygulanan bir istek mi
- Kararın contract kitine bir senaryo olarak yazılması
- Fake ve libmpv adapter'ının aynı cevabı vermesi

## YAPILMAYACAK

- `MPV_EVENT_SEEK` kapısını gevşetmek → `NEN-051`'in kararı
- Yükleme sırasında seek'i kuyruklayıp otomatik uygulamak — karar verilmeden
  davranış eklenmez

## Neden ayrı task

`NEN-051` sırasında ölçüldü: `MPVPlaybackEngine`'in `requireMedia`'sı
`.loading` fazını **kabul ediyor**, yani port seviyesinde yüklenirken seek
etmek meşru görünüyor. Gerçek mpv ise reddediyor —
`EngineFailure(code: -12)` (`MPV_ERROR_COMMAND`). `EngineFailure` motorun
kendi sayısı demektir; bu, kabuğa "beklenmeyen motor hatası" olarak görünür,
oysa durum tamamen normal.

Contract kitinin hiçbir senaryosu yüklenirken seek etmiyor, dolayısıyla fake
ile gerçek adapter'ın bu noktada anlaşıp anlaşmadığı **bilinmiyor**. Kural 5
gereği `NEN-051`'e eklenmedi.

## Kanıt (DoD)

- [ ] Kararın hangi cevabı şart koştuğu yazılı
- [ ] Contract kitinde yüklenirken seek senaryosu var
- [ ] Fake ve libmpv adapter'ı aynı cevabı veriyor
- [ ] Negatif: adapter'ın yanlış cevap vermesi kiti kırmızıya döndürüyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
