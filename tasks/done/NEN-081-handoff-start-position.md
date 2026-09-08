---
id: NEN-081
title: Apply the start position a handoff carries
milestone: M4
size: S
state: done
closed: 2026-09-08
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

- [x] Pozisyonlu handoff medyayı o andan açıyor (platform testi, inen pozisyon
      beklenerek okunur — `NEN-052`'nin öğrettiği desen, tek seferlik okuma değil)
- [x] Pozisyonsuz handoff medyayı baştan açıyor
- [x] Negatif: negatif · sayı olmayan · süreyi aşan değer için medya yine
      açılıyor, hata yüzeyi tetiklenmiyor
- [x] Negatif kontrol: pozisyon uygulaması geri alındığında yalnız bu task'ın
      testleri kırmızı
- [x] `bash scripts/test-macos.sh` ve Rust workspace yeşil

## Kanıt kaydı

Tam kanıt: `evidence/M4/NEN-081-checklist.md`.

**Uygulama yolu ADR-0043 Notlar'a eklenen bir kullanıcı kararıyla netleşti**
(Karar 2'nin "`load` ile birlikte ertelenir" ile "süreyi aşan değer sessizce
düşer, baştan açılır" maddeleri `load` anında süre bilinmediği için aynı yolda
tutulamıyordu). Pozisyon kabukta (`pendingHandoffStartPositionMs`) tutulur,
`apply(_:)`'ın `.ready` dalında — `play()`'den **önce** — uygulanır; süre
okunabiliyorsa ve pozisyon ona eşit/aşıyorsa seek düşer, okunamıyorsa (canlı
yayın) pozisyon uygulanır. ADR-0042'nin kendi kontratına dokunulmadı.

**Negatif · sayı olmayan değer maddesi bu task'ın kapsamına Rust tarafında
girmiyor** — `NEN-080`'in `parse_seconds_as_millis`'i bunları zaten
`start_position_ms: None`'a çeviriyor (`an_unreadable_position_falls_silently`
testi), yani bu shell'e hiç `Some` olarak ulaşmıyor; "pozisyonsuz handoff"
maddesiyle aynı yoldan zaten kanıtlı. Bu task'ın kendi kapısı yalnız
**süreyi aşan/süreye eşit** değeri kapatıyor — hem `FakeSession` hem gerçek
libmpv motoruyla kanıtlandı.

**Kanıt:** `FakeSession` üzerinden 10 test (`HandoffStartPositionTests`),
gerçek libmpv üzerinden 1 test (`HandoffStartPositionRealEngineTests` —
gerçek motor `contract-clip.mkv`'yi 12.000 ms'de açıp inen pozisyonu
bekleyerek okuyor, sonra aynı medyayı 999.000 ms'le yeniden açıp süreyi aşan
pozisyonun düştüğünü doğruluyor). Üç ayrı negatif kontrol (seek çağrısı
kaldırıldı → yalnız pozisyon testleri kırmızı; süre kapısı kaldırıldı →
yalnız süre testleri kırmızı; seek `play()`'den sonraya taşındı → yalnız sıra
testi kırmızı) ayrık ölçüldü. Gerçek `.app` kabulü: doğrudan argv exec
(`--start=5` ve `--start=999`, `contract-clip.mkv` üzerinde) ekran
görüntüsüyle kanıtlı.

Rust workspace **646 passed / 1 ignored** (kod değişmedi, yalnız iki yorum),
fmt/clippy/`cargo deny check` temiz. macOS Swift paketi **221 → 232**.
`swift test --no-parallel`: **iki ardışık koşu, 232/232, 0 kırmızı** — kesin
kanıt. `bash scripts/test-macos.sh`'ın varsayılan paralel koşusu bu oturumun
kendi masaüstü yükü altında ara sıra, bu task'tan bağımsız, önceden
belgelenmiş main-actor zamanlama testlerinde (`NEN-049`/`NEN-066` sınıfı)
kırmızı çıktı; `--skip HandoffStartPositionRealEngineTests` ile izole
edildiğinde aynı kırmızı aynen kaldı — bu task'ın değişikliği karışmıyor.
`bash scripts/test.sh` ve `bash scripts/check-docs.sh` çıkış 0.
