# NEN-087 — Geri alınabilir Stremio MPV köprüsü kanıtı

Tarih: **2026-09-08** · Apple Silicon · macOS 27.0 · Xcode 26.6 · Swift
6.3.3 · libmpv 2.5.0

## Deterministik kök testi

`bash scripts/tests/stremio-mpv-bridge.test.sh` — **SONUÇ: tüm doğrulamalar
geçti.** Geçen senaryolar:

- temiz kurulum ve aynı komutta idempotency;
- yabancı hedefin açık `--replace` olmadan reddi;
- replace yedeğinin byte, inode metadata ve izinlerinin korunması ve
  uninstall sonrası birebir geri yükleme;
- checksum uyuşmazlığında değiştirilmiş wrapper'ın korunması;
- soğuk Stremio argv'sinin sırası/değerleri değişmeden app executable'a
  ulaşması;
- sıcak handoff'un aynı locator ve başlangıç semantiğiyle `open -a` yoluna
  ulaşması;
- tanınmayan çağrının önceki MPV executable'ına eksiksiz devri;
- locator/argv sentinel'li çağrıda wrapper stdout/stderr sızıntısının
  olmaması.

Testler geçici kök ve sahte executable'lar kullandı; gerçek sistem hedefi veya
özel medya verisi yazılmadı.

## Gerçek kurulu köprü

- `bash scripts/stremio-mpv-bridge.sh status` → **installed**.
- Kurulu hedef Nen marker v2 ve güncel `pgrep` süreç tespitiyle eşleşti;
  manifest checksum doğrulandı.
- Hedef ve `previous-mpv` yedeği: **0755, root:wheel**. Kullanıcı onayıyla
  önceki yabancı wrapper yedekte bırakıldı; Nen köprüsü kurulu kaldı.
- Önceden Nen Player süreci yoktu; koşu sonrası başlatılan süreçler
  sonlandırıldı ve son kontrol **nenplayer_process=no** verdi.

## Platform handoff kontrolü

Repo içi taze `.build/NenPlayer.app`, telif-temiz sentetik fixture ile gerçek
kurulu köprü üzerinden çalıştırıldı. Argümanlar ve fixture locator'ı çıktıya
alınmadı; wrapper ve child output'u `/dev/null`'a yönlendirildi.

```text
cold_running=yes
warm_command_rc=0
warm_process_after=yes
```

Böylece soğuk üç argümanlı Stremio çağrısının app executable'a ulaştığı ve
uygulama açıkken sıcak çağrının mevcut Nen Player sürecine dönüp başarıyla
tamamlandığı doğrulandı. Gerçek Stremio 5.1.26 uçtan uca kabulü NEN-088
kapsamında bırakıldı.

## Kapılar

- `bash scripts/test.sh` — **yeşil** (4 test dosyası).
- `bash scripts/test-macos.sh` — **239 test, 0 failure**.
- `bash scripts/build-macos-app.sh` — **çıkış 0**.
- `cargo test --manifest-path core/Cargo.toml --workspace --no-fail-fast` —
  **yeşil**.
- `cargo fmt --all --check --manifest-path core/Cargo.toml` — **yeşil**.
- `cargo clippy --manifest-path core/Cargo.toml --workspace --all-targets --
  -D warnings` — **yeşil**.
- `cargo deny check` — **advisories/bans/licenses/sources yeşil**.
- `bash scripts/check-docs.sh` — **yeşil**.
- `git diff --check` — **çıkış 0**.

Kanıt kaydı locator, token, query, ham argv veya özel tam yol içermez.
