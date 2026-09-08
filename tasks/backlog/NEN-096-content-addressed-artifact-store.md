---
id: NEN-096
title: Content-addressed artifact store with atomic commit
milestone: M5
size: M
state: backlog
closed:
depends_on: [NEN-095]
blocks: [NEN-098]
adr: [0017]
---

# NEN-096 — Content-addressed artifact store with atomic commit

## Sonuç

Bir artifact'in içeriği içeriğinden türeyen bir adresle ve **atomik** olarak
yazılıyor; yarım yazım hiçbir koşulda okunabilir olmuyor.

## Bağlam

Şartname §11: "Final artifact: yalnız validation sonrası · **atomik commit** ·
restart sonrası reuse". Atomiklik burada teorik bir incelik değil: yarım yazılmış
bir WebVTT dosyası, M5'in "yarım/progressive çıktı yayınlanmaz" kriterini
doğrudan ihlal eder.

`nen-persist` bugün üç satırlık boş bir iskelet.

## Kapsam

- İçerik adresli yazma ve okuma (ADR-0017'nin kararlaştırdığı biçimde)
- Atomik commit: geçici yazım + yerine koyma; kısmi dosya görünür olmaz
- Aynı içeriğin ikinci kez yazılmasının yeni kopya üretmemesi
- Depo kökünün dışarıdan (platformdan) enjekte edilmesi

## YAPILMAYACAK

- Metadata index ve sorgulama — `NEN-098`
- Cache identity hesabı — `NEN-097`
- Kullanıcıya "cache'i temizle" komutu — M6/UI işi, burada yalnız API sınırı
- Şifreleme — kapsam dışı; artifact kullanıcının kendi diskinde

## Kanıt (DoD)

- [ ] Unit: yazılan artifact aynı adresten birebir okunuyor
- [ ] Unit: aynı içerik iki kez yazıldığında tek kopya kalıyor
- [ ] Negatif: yazım ortasında kesilen bir commit sonrası depoda **okunabilir
      hiçbir kısmi artifact yok** (kasıtlı olarak kesilen yazımla ölçülür)
- [ ] Negatif: atomik yerine koyma düz yazımla değiştirildiğinde yukarıdaki test
      kırmızıya dönüyor — kontrol sağır değil
- [ ] Negatif: depo kökünün dışına çıkmaya çalışan bir adres reddediliyor
- [ ] Guard: depo yolu ve dosya adı hiçbir log yüzeyine düşmüyor (K23 #3)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
