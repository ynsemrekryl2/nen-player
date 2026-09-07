# NEN-075 — Gerçek `.app` kabul kontrol listesi

Tarih: 2026-09-07 · Build: `scripts/build-macos-app.sh` (ad-hoc imzalı,
`codesign --verify --strict` → `valid on disk`).

## Fixture

```
nen075-accept/
  Film.mkv              (fixtures/media/aspect-4x3-clip.mkv kopyası)
  Film.srt              düz sidecar, İngilizce metin
  Film.tr.srt           dil alt-uzantılı sidecar, İngilizce metin
                         (dil yalnız dosya adından çözülebilir — ölçüm amaçlı)
  Film.en.srt           dil alt-uzantılı sidecar, İngilizce metin
  real-fr.srt           geçerli SRT, symlink hedefi
  Film.fr.srt -> real-fr.srt   symlink (reddedilmesi beklenen aday)
  Baska.tr.srt          başka bir "medyanın" sidecar'ı (aday olmaması beklenen)
```

## Koşu

1. Uygulama `Film.mkv` ile açıldı (`⌘O` → `NSOpenPanel`, `⇧⌘O`'ya **hiç
   dokunulmadan**).
2. Medya oynamaya başladı; CC etiketi otomatik olarak `Film.tr.srt` gösterdi
   (makine dili Türkçe — ADR-0031 Karar 4.3 otomatik seçimi).
3. Ekranda çizilen replik: *"Turkce sidecar: NEN-075 tarama ile bulundu."* —
   `Film.tr.srt`'nin içeriği, tarama tarafından bulunup yüklendiğinin kanıtı.
4. Altyazı menüsü açıldı: `Kullanıcı Altyazıları  3` — `Film.srt`,
   `Film.en.srt`, `Film.tr.srt` üçü de listede.
5. **`Film.fr.srt` (symlink) listede yok** — sessizce reddedildi
   (ADR-0031 Karar 5, FileRejection::Symlink).
6. **`Baska.tr.srt` listede yok** — farklı basename, hiç aday olmadı.

## Sonuç

DoD'un ürün yüzeyi maddesi karşılandı: `Film.tr.srt` kullanıcı hiçbir ek eylem
yapmadan bulundu, kataloglandı ve ekranda çizildi; symlink kapısı yeni
yüzeyde de çalışıyor.

Not: menü grubu `Kullanıcı Altyazıları` (dile göre değil) — bu bir kusur
değil, `nen-catalog::menu::MenuGroup::UserSubtitles`'ın §8'de belgelenen
tasarımı: kullanıcı kaynakları diline bakılmaksızın tek grupta toplanır.
Dilin doğru çözüldüğü, otomatik seçimin `tr` sidecar'ını açmasından görülüyor.
