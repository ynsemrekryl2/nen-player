---
id: NEN-025
title: User subtitle loading and sidecar discovery
milestone: M3
size: M
state: done
depends_on: [NEN-013, NEN-015, NEN-024]
blocks: [NEN-026]
adr: [8, 31, 34]
---

# NEN-025 — User subtitle loading and sidecar discovery

## Sonuç

Kullanıcı altyazı dosyası yükleyebilir ve medyanın yanındaki aynı basename'li
SRT otomatik bulunur; güvenlik kurallarını ihlal eden dosyalar reddedilir.

## Kapsam

- "Altyazı Dosyası Yükle" akışı
- Sidecar tarama: medya ile aynı basename'e sahip `.srt`
- Güvenlik kapıları (`docs/security-policy.md` §4): regular-file doğrulaması,
  symlink reddi, path traversal reddi, boyut sınırı
- Encoding kontrolü + strict SRT parse
- **Yol digest'i:** `SubtitleSourceId::user(digest)`'in beklediği `[u8; 32]`'yi
  üreten yer burasıdır (ADR-0010 Karar 2). `nen-domain` bağımlılıksız olduğu
  için hash'i o crate hesaplayamaz ve ham yolu **hiçbir koşulda** almaz
  (K23 #3). Aynı dosya iki kez yüklendiğinde katalogda tek giriş kalması bu
  digest'e bağlıdır — kanıtı da burada üretilir.
- Hatalı dosya → kaynak "hatalı" işaretlenir, **playback durmaz**
- **Red ile hata ayrımı (ADR-0031 Karar 5):** güvenlik kapısından dönen dosya
  kataloğa **hiç girmez** — kaynak olmadı. Kullanıcı dosyayı açıkça
  yüklediyse geçici bildirim alır (ADR-0031 Karar 1); sidecar taramasında
  dönen dosya **sessizce** elenir. Encoding/parse hatası ise kataloğa girer
  ve "hatalı" işaretlenir (menüdeki görünümü NEN-026'da)

## YAPILMAYACAK

- Menüde gösterim → NEN-026
- OpenSubtitles indirme → M6
- Klasör tarama / özyinelemeli arama — yalnız aynı basename

## Kanıt (DoD)

- [x] Geçerli sidecar otomatik bulunuyor ve kullanıcı grubuna ekleniyor
- [x] Negatif: symlink olarak verilen `.srt` **reddediliyor**
- [x] Negatif: `../` içeren yol **reddediliyor**
- [x] Negatif: dizin/FIFO **reddediliyor**
- [x] Negatif: boyut sınırını aşan dosya okunmuyor
- [x] Aynı dosya iki kez yüklendiğinde katalogda tek giriş kalıyor (digest eşit)
- [x] Bozuk SRT'de playback **devam ediyor**, kaynak hatalı işaretleniyor
- [x] Negatif: güvenlikten dönen dosya katalogda **hiç yok** (hatalı kaynak
      olarak da görünmüyor)
- [x] Kullanıcının elle yüklediği dosya reddedilince bildirim üretiliyor;
      tarama sırasında reddedilen dosya bildirim üretmiyor

## Kanıt kaydı

Tam kayıt: `evidence/M3/NEN-025-checklist.md` ·
sandbox ölçümü: `evidence/M3/NEN-025-sandbox-measurement.md`

**Testler:** `cargo test` **489 passed / 0 failed** (`NEN-051` kapanışında
453'tü) · `swift test --package-path platforms/macos` seri **56 test / 7 suite**,
art arda 2/2, paralel 4/4.

**Bu task bir mimari kararı zorunlu kıldı — ADR-0034.** Sidecar keşfi, kod
yazılmadan önce ölçüldü ve sandbox altında **çalışmadığı** görüldü: medyanın
security scope'u açıkken kardeş `.srt` `EPERM`, dizin listeleme `EPERM`, ve
Apple'ın bu iş için gösterdiği related-item koordinasyonu
(`NSIsRelatedItemType` + `NSFileCoordinator`) da üç ayrı Info.plist kurulumunda
`EPERM`. Yani planlanan çözüm ölçümle elendi. Kullanıcı kararıyla App Sandbox
kaldırıldı; Mac App Store non-goal listesine eklendi, notarization
(`NEN-043`) etkilenmedi. Karşılığı: `security-policy.md` §4 kapıları artık
**tek** savunma hattı, bu yüzden negatif testleri tartışmasız zorunlu.

**Boyut sınırı icat edilmedi.** Plan 16 MiB'lık yeni bir sabit öngörüyordu;
kodu okurken `nen_subtitle::encoding::MAX_INPUT_BYTES`'ın (10 MiB,
NEN-015/ADR-0008) zaten var olduğu görüldü. İkinci ve daha büyük bir sınır
10–16 MiB bandındaki dosyanın önce tamamen okunup sonra reddedilmesi demek
olurdu — §4 #4'ün yasakladığı davranış. Kapı aynı sayıyı `stat`'tan soruyor;
`the_file_gate_and_the_decoder_share_one_limit` ayrışmayı engelliyor.

**Kapının "okumuyor" iddiası kanıtlandı.** `an_oversized_file_is_refused_without_being_opened`
dosyayı hem sınır üstü hem `chmod 000` yapıyor: kapı açmayı deneseydi
`Unreadable` dönerdi, `TooLarge` dönmesi ancak hiçbir şey açılmadıysa mümkün.

**Bir DoD maddesi beklenenden farklı yoldan karşılandı.** `NSOpenPanel`
symlink'i **çözüyor** (ölçüldü: `panelGaveSymlink=false`), yani kullanıcı panel
üzerinden symlink teslim edemiyor. Symlink kapısı tarama yolunda ve gelecekteki
panel-dışı girişlerde iş görüyor; gerçek `.app`'te symlink sidecar sessizce
elendi (`rejected(symlink) count=0`) ve `a_symlinked_subtitle_is_refused_and_never_followed`
kapıyı doğrudan kanıtlıyor.

**Kanıtın kontrolü de yapıldı.** Redaction guard'ında derive edilmiş bir ikiz
aynı değerleri sızdırıyor; dedup testlerinde `two_different_files_are_two_entries`
her şeyi tek girişe indiren bir digest'i eliyor; boyut kapısında
`a_file_exactly_at_the_size_limit_is_still_admitted` hiçbir negatif testin
yakalamayacağı bir yanlış reddi engelliyor.

**Sınır:** `NEN-025`'in gözle görülür tek çıktısı reddedilen dosyanın
bildirimi — katalog `NEN-026`'ya kadar görünmüyor. Bu yüzden gerçek `.app`
kanıtı ekran kaydı değil, ölçülmüş log satırları (prob commit edilmedi).

**Ayrılanlar:** `NEN-056` (ADR-0031 Karar 5'in üretilemeyen `çok büyük`
etiketi) · `NEN-057` (dosya adından dil ipucu) · `NEN-058` (yanında symlink
`.srt` olan medya açılmıyor — gözlendi, teşhis edilmedi). `NEN-049`'a yük
altında gözlenen bir paralel kırmızı işlendi.
