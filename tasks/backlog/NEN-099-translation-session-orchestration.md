---
id: NEN-099
title: Translation session orchestration without retarget
milestone: M5
size: M
state: backlog
closed:
depends_on: [NEN-098, NEN-019]
blocks: [NEN-100]
adr: []
---

# NEN-099 — Translation session orchestration without retarget

## Sonuç

Kaynak seçmek çeviri başlatmıyor; açık bir komut bir çeviri işi başlatıyor ve iş
sırasında kullanıcının başka bir kaynağa geçmesi işi **retarget etmiyor**.

## Bağlam

Şartname §9 bu davranışı tek tek sayıyor: kaynak seçmek AI çeviri **başlatmaz** ·
iş başlangıç source fingerprint'ine bağlı kalır · yeni source'a retarget edilmez ·
kullanıcı iptal edebilir · sonuç hedef dil grubuna eklenir · kullanıcı başka
source izliyorsa **zorla** AI çıktısına geçilmez.

Sonuç `SubtitleSourceKind::Ai` olarak kataloğa girer — bu varyant
`nen-domain/src/source.rs` içinde **zaten var** (NEN-019), yeni bir tür açılmaz.

M5'in birinci ve altıncı çıkış kriterleri buraya bakıyor.

## Kapsam

- `nen-app` içinde çeviri oturumu use-case'i: başlatma, ilerleme, iptal, sonuç
- İşin başlangıç `SourceFingerprint`'ine bağlanması ve retarget'ın reddi
- Tamamlanan artifact'in kataloğa `Ai` kaynağı olarak eklenmesi
- Kaynak zaten hedef dildeyse işin hiç başlatılmaması
- Cache isabetinde provider'a hiç gidilmemesi

## YAPILMAYACAK

- FFI yüzeyi — `NEN-100`
- macOS komutu, ayarı ve ilerleme yüzeyi — `NEN-101` · `NEN-102`
- Aynı anda birden fazla çeviri işi — kapsam dışı, gerekirse yeni backlog task'ı

## Kanıt (DoD)

- [ ] Integration: kaynak seçimi → açık komut → doğrulanmış artifact akışı uçtan uca geçiyor (mock provider ile)
- [ ] Negatif: yalnız kaynak seçmek hiçbir provider çağrısı üretmiyor (çağrı sayacı 0)
- [ ] Negatif: iş sırasında başka kaynak seçilince iş **başlangıç fingerprint'inde** kalıyor ve retarget edilmiyor
- [ ] Negatif: kullanıcı başka bir source izlerken tamamlanan AI çıktısına **zorla geçilmiyor**
- [ ] Unit: kaynak zaten hedef dildeyse iş başlatılmıyor
- [ ] Unit: cache isabetinde provider hiç çağrılmıyor, artifact doğrudan dönüyor
- [ ] Negatif: iptal edilen iş kataloğa hiçbir şey eklemiyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
