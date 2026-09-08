---
id: NEN-092
title: Targeted repair and full-block retry budget
milestone: M5
size: M
state: backlog
closed:
depends_on: [NEN-091]
blocks: [NEN-093]
adr: []
---

# NEN-092 — Targeted repair and full-block retry budget

## Sonuç

Doğrulamadan geçemeyen bir blok en fazla **iki targeted repair** ve **bir
full-block retry** ile onarılmaya çalışılıyor; bütçe tükenirse blok başarısız
olur ve yarım çıktı üretmez.

## Bağlam

Şartname §10 bütçeyi sayıyla veriyor: "en fazla **iki targeted repair**, en
fazla **bir full-block retry**". ADR-0016 hangi ihlalin targeted repair'e uygun
olduğunu kararlaştırır (`NEN-091`).

Bütçenin asıl işlevi maliyet değil **sonlanma garantisi**: onarım döngüsünün
provider'ı sınırsız çağırmaması, M5'in "yarım çıktı yayınlanmaz" kriterinin ön
koşulu.

## Kapsam

- Targeted repair isteğinin kurulması (yalnız ihlal eden cue'lar)
- Bütçe sayacı ve tükenme davranışı
- Full-block retry'ın targeted repair'den sonra tek sefer denenmesi
- Blok başarısızlığının tipli hata olarak yukarı taşınması

## YAPILMAYACAK

- Ağ kaynaklı transient retry — bu provider portunun işi (`NEN-090`), doğrulama
  bütçesiyle karıştırılmaz
- Bütçe sayılarının kullanıcı ayarı yapılması — şartname sabitliyor
- Checkpoint ve iptal — `NEN-093`

## Kanıt (DoD)

- [ ] Unit: ilk denemede geçen blok hiç repair çağırmıyor (sayaç 0)
- [ ] Unit: tek ihlal eden cue için targeted repair yalnız o cue'yu istiyor
- [ ] Unit: iki başarısız targeted repair'den sonra tam bir full-block retry deneniyor
- [ ] Negatif: bütçe tükendiğinde blok tipli hata ile başarısız oluyor ve **kısmi sonuç teslim etmiyor**
- [ ] Negatif: bütçe sayacı kaldırıldığında döngü sonsuza gitmiyor — kasıtlı olarak hep bozuk cevap veren bir provider ile üst sınır ölçülüyor
- [ ] Guard: repair isteği ve provider cevabı loglanmıyor (K23 #4, #5)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
