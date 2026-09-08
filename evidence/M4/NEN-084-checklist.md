# NEN-084 — Stremio handoff kabul kanıtı

Tarih: **2026-09-08** · Apple Silicon · macOS 27.0 · Xcode 26.6 · Swift
6.3.3 · libmpv 2.5.0 · Stremio 5.1.26

## Kabul yorumu

ADR-0043'ün kabul edilmiş sınırı uygulandı: Stremio'nun bu makinedeki kapalı
oynatıcı listesi Nen Player'ı doğrudan hedefleyemiyor ve gerçek Stremio
başlangıç konumunu `0` gönderiyor. Bu yüzden kabul, gerçek Stremio'nun canlı
gönderim yüzeyinin yeniden görülmesi ve aynı ölçülmüş CLI sözleşmesinin gerçek
Nen Player `.app` üzerinde fixture ile uçtan uca koşulması olarak kaydedildi.
Uygulama kimliği taklidi ve `nenplayer://` uçtan uca doğrulaması yapılmadı
(ADR-0043, roadmap S11).

## Adım listesi

1. Kurulu Stremio açıldı; `Info.plist` sürümü **5.1.26** olarak doğrulandı.
2. Gerçek oynatıcı ekranında harici menü açıldı; **VLC içinde oynat** ve
   **MPV içinde oynat** seçenekleri yeniden görüldü. Handoff biçiminin ayrıntılı
   argv ölçümü `evidence/M4/NEN-078-measurement.md`'deki güvenli şekil
   kaydıyla birleştirildi; bu koşuda locator veya tam argv kaydedilmedi.
3. Gerçek `.app` doğrudan çalıştırıldı: sentetik `contract-clip.mkv`, ölçülen
   `--start` bayrağı ve bilinmeyen `--no-terminal` bayrağıyla. Ekran kanıtında
   medya **00:12 / 00:30** konumunda, oynatma açık ve altyazı görünür:
   [`NEN-084-position.png`](NEN-084-position.png).
4. Aynı fixture geçici loopback HTTP sunucusundan opak `stream` adıyla ve
   `Content-Disposition` olmadan sunuldu. Handoff metadata'sı yokken medya
   **00:09 / 00:30** konumunda oynadı; hata veya kesinti görülmedi:
   [`NEN-084-no-metadata.png`](NEN-084-no-metadata.png).
5. Koşu sonrası unified log ile stdout/stderr ham kayıtları geçici dosyalarda
   tarandı ve silindi. URL/query sentinel'i, özel yol, başlangıç bayrağı ve
   bilinmeyen bayrak için eşleşme sayısı **0** oldu; ham log kanıta girmedi.

## M4 çıkış kriterleri

| Kriter | Durum | Kanıt |
|---|---|---|
| Stremio'nun ölçülmüş CLI sözleşmesiyle açılan medya doğru pozisyondan oynuyor | ✅ | `NEN-078` canlı yüzey ölçümü · `NEN-084-position.png` · 00:12 / 00:30 |
| Argüman ve medya URL'si loglanmıyor | ✅ | Unified log + stdout/stderr negatif taraması (tüm sayaçlar 0) · `NEN-083` guard suite'leri |
| Handoff metadata'sı yokken akış bozulmuyor | ✅ | `NEN-084-no-metadata.png` · opak locator, Content-Disposition yok, playback sürüyor |

## Güvenlik sınırı

Ekran kanıtları yalnız `fixtures/media/*` içeriğiyle üretildi. Kanıt ve
metinlerde gerçek medya URL'si, token, port, özel tam yol veya özel medya
metadata'sı yoktur. Stremio hesabından açılan gerçek içerik yalnız canlı yüzey
doğrulamasında kullanıldı; ekran görüntüsü veya ham çıktı olarak saklanmadı.

## Tam regresyon

- `cargo test --manifest-path core/Cargo.toml --workspace --no-fail-fast` — **yeşil**
- `cargo fmt --all --check --manifest-path core/Cargo.toml` — **yeşil**
- `cargo clippy --manifest-path core/Cargo.toml --workspace --all-targets -- -D warnings` — **yeşil**
- `cd core && cargo deny check` — **advisories/bans/licenses/sources yeşil**
- `bash scripts/test-macos.sh` — **239 test, 0 failure**
- `bash scripts/build-macos-app.sh` — **çıkış 0**
- `codesign --verify --deep --strict` — **valid on disk; designated requirement sağlandı**
- `bash scripts/test.sh` — **yeşil** (bundle, docs ve doctor kontrolleri)
- `bash scripts/check-docs.sh` — **yeşil** (tüm doküman kapıları geçti)
- `git diff --check` — **yeşil**
