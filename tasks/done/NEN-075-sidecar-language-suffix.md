---
id: NEN-075
title: Sidecar discovery does not see a language-suffixed subtitle
milestone: M3
size: S
state: done
closed: 2026-09-07
depends_on: [NEN-025, NEN-057]
blocks: []
adr: [31, 34, 41]
---

# NEN-075 — Sidecar taraması dil alt-uzantılı dosyayı görmüyor

## Sonuç

`Film.mkv`'nin yanındaki `Film.tr.srt`, kullanıcı ⇧⌘O ile açıkça seçmeden de
sidecar olarak bulunur ve kataloğa girer — bugün yalnız tam basename eşleşmesi
(`Film.srt`) taranıyor.

## Bağlam

`NEN-057` uygularken ölçüldü: `subtitle_files::sidecar_of(media)` yalnız
`media.with_extension("srt")` döndürüyor, yani `Film.mkv` için aranan tek
dosya `Film.srt`. `Film.tr.srt`, `Film.en.srt` gibi dil alt-uzantılı
sidecar'lar — kullanıcı kütüphanelerinde yaygın — bu taramadan **hiç**
geçmiyor; kullanıcı onları yalnız dosya seçiciyle elle yükleyebiliyor.

`NEN-025`'in bilinçli kapsam kararıydı (tek dosya, dizin listelemesi yok —
`sidecar_of`'un kendi dokümanı: "Deliberately not a directory listing").
`NEN-057`'nin Kapsam listesinde yoktu, Kural 5 gereği oraya eklenmedi.

## Karar (ADR-0041, accepted 2026-09-07)

**Seçenek 1 — dizin listelemesi.** Sidecar keşfi medyanın kendi dizinini bir
kez, özyinelemesiz listeler; `<basename>.` önekiyle başlayıp `.srt` ile biten
her girdi adaydır. Her aday `admit()`'in symlink, regular-file, traversal ve
boyut kapılarından **teker teker** geçer — dizin listelemesi aday kümesini
büyütür, kapıları gevşetmez. Aday sayısı **16** ile sınırlıdır
(`MAX_SIDECAR_CANDIDATES`); tam basename eşleşmesi (`Film.srt`) sıralamada
her zaman ilk gelir ve sınırdan her zaman kurtulur. Dil ikinci bir tablo ile
değil, var olan `from_file_name` ipucuyla okunur — adından dil çözülemeyen
bir aday (`Film.backup.srt`) reddedilmez, dili içerikten gelir.

Bu, ADR-0034 Karar 3'ün "dizin listelemez" cümlesini değiştirir. ADR-0034
supersede edilmedi; gövdesi duruyor, Notlar'a ADR-0041'e işaret eden madde
eklendi (ADR-0035 precedent'i).

Reddedilen: sabit dil-kodu tablosu (ikinci ISO tablosu, kombinatoryal patlama);
bugünkü hâli koru (yaygın adlandırma ürüne görünmez kalır).

## Kanıt (DoD)

- [x] Karar yazılı: hangi seçenek, neden — ADR-0041 (`accepted`)
- [x] Seçilen davranış negatif ve pozitif testle kanıtlı (gerçek dosya
      sistemi, `subtitle_file_gates.rs`'in deseninde)
- [x] Negatif: sidecar taraması hâlâ symlink/traversal/boyut kapılarından
      geçiyor — yeni yüzey eski kapıları atlamıyor

## Kanıt kaydı

`sidecar_of` yerine `sidecars_of` geldi
([`subtitle_files.rs`](../../core/crates/nen-app/src/subtitle_files.rs)):
medyanın kendi dizini `fs::read_dir` ile bir kez, özyinelemesiz listeleniyor;
`<basename>.` önekli, `.srt` ile biten (ASCII case-insensitive) her girdi
aday. Tam basename eşleşmesi her zaman ilk sırada, kalan adaylar ada göre
sıralanıp `MAX_SIDECAR_CANDIDATES = 16` ile kırpılıyor. `read_dir` hata
verirse (silinmiş/erişilemeyen dizin) tek tam eşleşmeye düşülüyor — yeni yüzey
bugünkünden hiçbir zaman dar değil. `admit()` ve `load()` dokunulmadı; her
aday aynı dört kapıdan geçiyor.

`SubtitleLibrary::add_sidecar_of` → `add_sidecars_of` (tek `Option<AddOutcome>`
yerine `Vec<AddOutcome>`), FFI'de `add_sidecar_for` → `add_sidecars_for`
(`Vec<FfiSubtitleOutcome>`). `bash scripts/build-apple.sh` ile
`nen_ffi.swift` yeniden üretildi; macOS kabuğunda tek satır değişti
(`PlayerModel.swift:349`, sonucu zaten `_ =` ile atıyordu).

**Dokuz yeni test** `subtitle_file_gates.rs`'e eklendi: dil alt-uzantılı
sidecar bulunuyor ve dili dosya adından çözülüyor; tam eşleşme + iki dilli
sidecar bir arada; dili çözülemeyen alt-uzantı (`Film.backup.srt`) yine de
kabul ediliyor; tarama yalnız medyanın kendi basename'ini alıyor (komşu
`Baska.tr.srt` alınmıyor); `.srt` olmayan uzantılar reddediliyor; 20 adaylık
bir dizinde tam 16 işleniyor ve tam eşleşme her zaman içeride; alt-uzantılı
symlink ve aşırı büyük dosya reddediliyor; alt-uzantılı bir dizin
`NotRegularFile` ile reddediliyor. Yol üstünde ölçülen bir kenar durum
(`Film.SRT` büyük/küçük harf farkıyla ikinci bir aday olarak sayılmaması)
ayrı bir testle kapatıldı.

**Dört negatif kontrol, ayrık:** dizin listelemesi geri alınınca (yalnız tam
eşleşme) **8** kırmızı; sınır sıralaması geri alınınca (tam eşleşme de düz
alfabetik) **1** kırmızı — sınır testinin tam olarak ölçtüğü şey; `admit()`'in
symlink kapısı kaldırılınca **2** kırmızı (yeni yüzey eski kapıyı atlamıyor);
harf-duyarsız dedup geri alınınca **1** kırmızı. Her biri geri yüklendi.

Rust workspace **594 → 605** (bu task ile +11 test, dokuzu NEN-075'in kendi,
biri kenar-durum, cargo-deny'ın `metadata` çıktısı değişmedi), üç ardışık
temiz koşuda **0 kırmızı**. fmt, clippy (`-D warnings`), cargo-deny (yeni dış
bağımlılık yok) yeşil. macOS Swift paketi **195/195**. `.app` build'i ve
`codesign --verify --strict` → `valid on disk`.

**Gerçek `.app` kabulü** (`evidence/M3/NEN-075-checklist.md`): `Film.mkv`'nin
yanına `Film.srt`, `Film.en.srt`, `Film.tr.srt`, ve symlink `Film.fr.srt`
(farklı bir "medyaya" ait `Baska.tr.srt` ile birlikte) konup uygulama
`⌘O`'dan açıldı — `⇧⌘O`'ya hiç dokunulmadan. Medya oynamaya başladı, CC
etiketi otomatik `Film.tr.srt` gösterdi (sistem dili Türkçe — ADR-0031 Karar
4.3), replik ekranda çizildi (*"Turkce sidecar: NEN-075 tarama ile
bulundu."*). Altyazı menüsünde `Kullanıcı Altyazıları  3` — üç sidecar da
listede; symlink `Film.fr.srt` sessizce yok; `Baska.tr.srt` hiç aday olmadı.

**Yol üstünde bulunan, kapsam dışına ayrılan kusur:** `check-docs.sh` adım
9'un STATUS güncellik denetimi, CI'ın shallow clone'unda (`fetch-depth: 1`)
her done task'ı `git log`'un tek gördüğü commit'in tarihiyle (yani her zaman
HEAD'in tarihi) okuyor — `NEN-076` olarak dosyalandı.

**Doküman:** `security-policy.md` §4'e sidecar keşfinin sınırı (tek dizin,
özyineleme yok, 16 aday) ilk kez açıkça yazıldı. ADR-0041 `accepted`,
ADR-0034 Notlar'ına işaret eklendi (kendi commit'i, `1ad194d`).
