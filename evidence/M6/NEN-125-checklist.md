# NEN-125 — Kanıt kaydı

Tarih: 2026-09-14  
Task: `NEN-125`  
ADR: [`ADR-0047`](../../docs/adr/0047-preferred-language-auto-download.md)

## Kabul edilen sözleşme

- Otomatik OpenSubtitles indirmesi varsayılan olarak kapalıdır.
- Ayar açıkken önce birinci, sonra yalnız yerel `embedded`/`user` kaynağı
  olmayan ikinci tercih dili değerlendirilir.
- Otomatik seçim yalnız `Automatic`, çakışmasız identity assessment ve
  `ConfidenceScore >= 60` ile yapılabilir.
- Her medya için günde en fazla bir indirme denemesi vardır; hata veya kota
  durumunda seçim kapalı kalır ve aynı açılışta retry yapılmaz.
- Başarılı indirme görünür kaynak olur; kullanıcı seçimi sessizce değişmez.
  `ai` kaynağı otomatik seçime girmez.

## Karar girdisi

`NEN-035` fixture ölçümü: 10 vaka; 8 otomatik (%80), 1 aday (%10), 1 manuel
(%10); yanlış sessiz otomatik karar 0. `core/crates/nen-identity/src/confidence.rs`
mevcut otomatik güven kapısını `ConfidenceScore(60)` olarak tanımlar. Yeni bir
ölçülmemiş eşik eklenmedi.

## Dokümantasyon doğrulaması

- ADR-0047 kullanıcı onayıyla `status: accepted` oldu.
- ADR-0010 Notlar bölümü ertelenmiş `opensubtitles` basamağının açıldığını ve
  implementasyonun `NEN-038` kapsamında olduğunu kaydeder.
- `docs/adr/README.md` kabul edilen ADR listesinde 0047'yi gösterir;
  `docs/DECISIONS.md` kabul sayacı 38 ve liste 0047'yi içerir.

## Komut kanıtı

- `bash scripts/check-docs.sh` — **10/10 geçti**.
- `bash scripts/task-index.sh --check` — **geçti**.
- `git diff --check` — **geçti**.
- CI `34886476167` — format, clippy, cargo test, cargo deny, shell testleri,
  iki task index kontrolü ve docs kontrolü **tamamı başarılı**.

K23 kapsamında özel URL, credential, tam yol, provider payload'ı, altyazı
diyaloğu veya dosya metadata'sı bu kayda alınmamıştır.
