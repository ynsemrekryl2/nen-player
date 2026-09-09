---
id: NEN-106
title: A multi-block run rejects its own provider's progress
milestone: M5
size: S
state: backlog
closed:
depends_on: [NEN-090, NEN-093]
blocks: [NEN-099]
adr: [0004]
---

# NEN-106 — A multi-block run rejects its own provider's progress

## Sonuç

Birden fazla bloğa bölünen bir çeviri işi, kendi sağlayıcısının bildirdiği
ilerleme yüzünden `Permanent` hatasıyla düşmüyor.

## Bağlam

`NEN-096`'nın uçtan uca testi yazılırken ölçüldü. `translate_checkpointed`
(`nen-translate/src/checkpoint.rs:220`) bütün bloklar için **tek bir**
`TranslationCall` kullanıyor. `TranslationCall::progress`
(`nen-ports/src/translation/mod.rs:218`) ise aynı çağrı içinde `total`'ın
değişmesini `TranslationProviderError::Permanent` ile reddediyor — ADR-0004'ün
monoton ilerleme sözleşmesi gereği, doğru bir kural.

Ama bir sağlayıcı yalnız **kendi bloğunun** isteğini görür; belgenin toplam cue
sayısını bilemez, dolayısıyla `total` olarak blok cue sayısını bildirir. İki
bloğun çıktı pencereleri farklı boyda olduğu anda (varsayılan düzende neredeyse
her zaman: 50 cue'luk bir belge `37` + `13` verir) ikinci blok düşüyor.

Ölçüm: `MockTranslationProvider` ile 50 cue'luk bir belge
`Block(Provider(Permanent))` ile düşüyor; **aynı** kod 30 cue'da (tek blok)
geçiyor. `nen-translate`'in kendi testleri bunu görmedi çünkü fixture
sağlayıcısı (`EchoTranslationProvider`) hiç `progress` çağırmıyor.

Kusur sağlayıcıda değil orkestrasyonda: `NEN-092`'nin retry için zaten
kullandığı `TranslationCall::fork` deseni (bağımsız ilerleme toplamı, paylaşılan
iptal geçidi) blok sınırında da gerekiyor. Belge geneli yüzdesinin nasıl
toplanacağı `NEN-099`/`NEN-102`'nin işi.

## Kapsam

- Blok sınırında ilerleme dizisinin bağımsız başlaması
- İptal geçidinin paylaşılmaya devam etmesi (ADR-0004 Karar 2/5 bozulmadan)

## YAPILMAYACAK

- Belge geneli ilerleme toplaması ve kullanıcıya gösterimi — `NEN-099` · `NEN-102`
- `TranslationCall`'ın monotonluk kuralını gevşetmek — kural doğru

## Kanıt (DoD)

- [ ] Negatif: ilerleme bildiren bir sağlayıcıyla çok bloklu bir belge baştan
      sona çeviriliyor; bugünkü kodda bu test kırmızı
- [ ] Negatif: blok sınırında iptal hâlâ çalışıyor — fork paylaşılan geçidi
      koruyor (`NEN-093`'ün iptal testleri yeşil kalıyor)
- [ ] Unit: her bloğun ilerleme dizisi kendi `total`'ıyla monoton

## Kanıt kaydı

<!-- done olurken doldurulacak -->
