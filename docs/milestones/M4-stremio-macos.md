# M4 — Stremio Handoff (macOS)

> **İskelet.** Task kırılımı, bir önceki milestone kapanırken üretilir
> (bkz. `tasks/README.md` → "Yeni task açma").

## Amaç

Stremio'dan external player olarak açılan medyanın macOS'ta doğru pozisyondan oynaması. Nen Player bir Stremio add-on'u **değildir**; burada yalnız handoff alıcısı tarafı yapılır.

## Kapsam

- External-player launcher veya open-with handoff
- Positional file/http/https argümanı
- Optional başlangıç pozisyonu
- Handoff metadata'sının opsiyonel güçlü kanıt olarak kullanılması

## Kapsam dışı

- Android Intent tarafı → M10
- Stremio'ya sonuç/pozisyon döndürme → M10 (Android senaryosu)
- Canonical ID'nin **her zaman** geleceğini varsaymak — yasak

## Çıkış kriterleri

- [ ] Stremio'dan açılan medya doğru pozisyondan oynuyor
- [ ] Negatif: argüman ve medya URL'si **loglanmıyor** (log denetimi)
- [ ] Handoff metadata yoksa akış bozulmuyor (kanıt opsiyoneldir)

## Task'lar

_(henüz kırılmadı)_

## Bağımlılıklar

M3

## Retro

<!-- kapanışta doldurulacak -->
