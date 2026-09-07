---
id: NEN-081
title: Apply the start position a handoff carries
milestone: M4
size: S
state: backlog
closed:
depends_on: [NEN-080]
blocks: [NEN-084]
adr: [42]
---

# NEN-081 — Apply the start position a handoff carries

## Sonuç

Handoff bir başlangıç pozisyonu taşıyorsa medya o andan açılıyor; taşımıyorsa
veya değer geçersizse baştan açılıyor ve kullanıcıya hata gösterilmiyor.

## Bağlam

Kontrat zaten kararlı ve bu task yeni bir kontrat kurmuyor: **ADR-0042**
(`NEN-052`) yüklenirken verilen bir seek'in reddedilmeyip tutulacağını ve
`FILE_LOADED` anında uygulanacağını söylüyor; macOS'ta karşılığı
`MPVPlaybackEngine.deferredSeekMs`. `NEN-052`'nin ölçümü yükleme penceresini
**2,5–12 ms** bulmuştu — handoff'la gelen bir pozisyon tam olarak o pencereye
düşer, yani bu yol ADR-0042 olmadan güvenilir olmazdı.

Pozisyonun taşıyıcısı ve birimi `NEN-079`'un ADR'sinde kararlaştırılır.

## Kapsam

- Handoff'tan gelen pozisyonun okunması ve `load` ile birlikte uygulanması
- Geçersiz değerlerin sessizce düşürülmesi: negatif, sayı olmayan, medyanın
  süresini aşan, taşmalı değer — hepsinde medya **yine açılır**, baştan
- ADR-0042'nin Karar 4'ü korunur: yükleme başarısız olursa veya `stop` gelirse
  ertelenen seek düşer

## YAPILMAYACAK

- Alıcı yüzey → `NEN-080`
- Kaldığı yerden devam (resume) özelliği — kullanıcının kendi izleme geçmişi
  bu task değil; buradaki pozisyon **yalnız** handoff'tan gelir
- Stremio'ya son pozisyonu geri döndürmek → M10
- ADR-0042'nin seek kontratını değiştirmek

## Kanıt (DoD)

- [ ] Pozisyonlu handoff medyayı o andan açıyor (platform testi, inen pozisyon
      beklenerek okunur — `NEN-052`'nin öğrettiği desen, tek seferlik okuma değil)
- [ ] Pozisyonsuz handoff medyayı baştan açıyor
- [ ] Negatif: negatif · sayı olmayan · süreyi aşan değer için medya yine
      açılıyor, hata yüzeyi tetiklenmiyor
- [ ] Negatif kontrol: pozisyon uygulaması geri alındığında yalnız bu task'ın
      testleri kırmızı
- [ ] `bash scripts/test-macos.sh` ve Rust workspace yeşil

## Kanıt kaydı

<!-- done olurken doldurulacak -->
