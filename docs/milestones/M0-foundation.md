# M0 — Foundation

## Amaç

Repo, kalıcı roadmap/task/ADR sistemi ve toolchain ön koşul kontrolü hazır olsun;
bir sonraki oturum `tasks/INDEX.md`'yi açıp doğrudan sıradaki task'tan devam
edebilsin. Bu milestone hiçbir ürün kodu içermez ve hiçbir araç kurulumu
gerektirmez.

## Kapsam

- Dizin ağacı, `.gitignore`, `.editorconfig`, `README.md`, `CLAUDE.md`
- `docs/`: product-spec, roadmap, architecture, glossary, security-policy,
  testing-strategy
- Task sistemi: şablon, README, 28 task dosyası, üretilen `INDEX.md`
- ADR sistemi: şablon, ADR-0001 (süreç), planlanan ADR listesi
- `scripts/`: `doctor.sh`, `task-index.sh`, `check-docs.sh`

## Kapsam dışı

- Herhangi bir ürün kodu, Cargo/Gradle/Xcode projesi → M1
- CI workflow dosyası (NEN-005) ve redaction yardımcıları (NEN-006) → ikisi de
  Rust crate gerektirir, M1'e taşındı
- Lisans metni seçimi → S1 + ADR-0012, M3 öncesi
- Commit → kullanıcı isteğine bağlı

## Çıkış kriterleri

- [x] `bash scripts/doctor.sh` bu makinedeki eksikleri doğru raporluyor
- [x] `bash scripts/task-index.sh` 28 task'tan `INDEX.md` üretiyor
- [x] `bash scripts/check-docs.sh` çıkış kodu 0 (tüm denetimler geçiyor)
- [x] Negatif testler: kırık bağımlılık, ikinci aktif task, kanıtsız `done`,
      bayat index ve `accepted` olmayan ADR — beşi de hata veriyor
- [x] ADR-0001 `accepted`

## Task'lar

`NEN-001` · `NEN-002` · `NEN-003` · `NEN-004`

## Bağımlılıklar

Yok — kritik yolun başlangıcı.

## Retro

**Durum: kapandı** (2026-08-24). NEN-001 · NEN-002 · NEN-003 · NEN-004 → done.

**Ne değişti:**

- **NEN-005 (CI) ve NEN-006 (redaction) M1'e taşındı.** İkisi de Rust crate'i
  gerektiriyor; `nen-ffi` iskeleti (NEN-007) olmadan çalıştırılamazlar. M0'ın
  "hiçbir araç gerektirmez" özelliği böylece korundu.
- **`LICENSE.md` boş bırakıldı.** Lisans seçimi S1 (dağıtım modeli) ve ADR-0012
  (libmpv LGPL/GPL linkleme) kararlarına bağlı. Şimdi bir lisans yazmak,
  sonradan geri alınması zor bir taahhüt olurdu.

**Doğrulanan varsayım:** dosya tabanlı task sistemi mekanik olarak
denetlenebiliyor — beş negatif testin hepsi doğru hata ve çıkış kodu 1 verdi.

**Sürpriz:** `mpv --version` bu makinede yanıt vermeyip takıldı. `doctor.sh`
bu yüzden CLI çalıştırmak yerine kütüphaneyi (`pkg-config` → dylib yolları)
arıyor ve tüm dış komutları zaman sınırıyla çalıştırıyor. Aynı dikkat M3'te
libmpv adapter'ında da gerekecek.

**Sonraki:** M1 — ancak Rust kurulumu gerekiyor (`scripts/doctor.sh`).
Sıradaki uygun task: **NEN-007**.
