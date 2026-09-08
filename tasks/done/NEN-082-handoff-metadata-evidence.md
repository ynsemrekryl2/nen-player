---
id: NEN-082
title: Treat handoff metadata as optional evidence
milestone: M4
size: M
state: done
closed: 2026-09-08
depends_on: [NEN-080]
blocks: [NEN-084]
adr: [9]
---

# NEN-082 — Treat handoff metadata as optional evidence

## Sonuç

Handoff'la gelen metadata kimlik kanıtı olarak değerlendirilir; gelmezse veya
bozuksa akış hiç bozulmaz — medya yine oynar, kimlik diğer kanıtlardan çıkar.

## Bağlam

Şartname §5: *"Stremio canonical media identity gönderirse optional güçlü kanıt
olarak kullanılabilir. Her zaman geleceği varsayılmamalıdır."* §6 aynı şeyi
kanıt listesinde tekrar ediyor: *"optional handoff metadata"* zaten listenin ilk
maddesi. Yani model tarafı ADR-0009'da hazır; eksik olan handoff'un o listeye
bağlanması.

`security-policy.md` §2 "handoff extras"ı **adıyla** düşman girdi sayıyor:
doğrulanmadan kullanılmaz, parse eden kod `unwrap`/`expect` kullanmaz, untrusted
metin doğrudan bir yola/komuta/format string'ine gömülmez.

ADR-0043 Karar 5 ayrı bir metadata kanalı açmaz: handoff metadata'sı locator'ın
kendisidir ve ADR-0009'un mevcut uzak URL path / sunucu beyanı katmanlarına
girer.

## Kapsam

- Handoff locator'ının mevcut `MediaEvidence` uzak-kanıt akışına bağlanması
- Kanıt katmanlarının ADR-0009'daki değerlendirme sırasının korunması —
  handoff yeni bir sıra icat etmez, var olan sıraya bir katman olarak girer
- Eksik, boş, bozuk ve düşmanca locator ipuçlarının playback'i bozmadan
  sınırlandırılması veya yok sayılması
- Kimlik çıkarılamasa da medyanın oynaması (şartname §6)

## YAPILMAYACAK

- Kimlik çıkarımının kendisini değiştirmek — `nen-identity` mevcut haliyle
  kullanılır
- Kullanıcıya teknik ID göstermek veya sordurmak — şartname §6 ve non-goal
- Medya URL'sinin query/fragment/host'unu kimliğe katmak — ADR-0009 ve §6'nın
  kesin yasağı
- Metadata'yı loglamak → `NEN-083`
- Provider sorgusu tetiklemek → M6
- Yeni yapılandırılmış metadata alanı veya yeni kanıt hata tipi eklemek

## Kanıt (DoD)

- [x] Metadata taşıyan handoff `MediaEvidence`'a beklenen alanları koyuyor
      (unit + golden)
- [x] **Metadata yokken akış bozulmuyor** — medya oynuyor, kimlik diğer
      kanıtlardan çıkıyor (M4 çıkış kriteri #3)
- [x] Negatif: locator/HTTP doğrulama hataları mevcut tipli hatalarıyla
      sonuçlanıyor, panik yok, `unwrap` yok; kanıt hatası playback'i reddetmiyor
- [x] Negatif: aşırı uzun, fazla segmentli, kontrol karakterli ve traversal
      ipuçları sınırlandırılıyor veya yok sayılıyor
- [x] Negatif: query · fragment · host kimliğe hiç girmiyor
- [x] Negatif kontrol: metadata katmanı geri alındığında yalnız bu task'ın
      testleri kırmızı

## Kanıt kaydı

- Core `nen-app` testleri, workspace testleri, fmt, clippy ve deny yeşil;
  handoff path title/year/season/episode, Content-Disposition önceliği, no-header
  `Unknown`, redirect basename, hostile filename ve mevcut URL-hints
  golden/limit testleri geçti.
- `bash scripts/test-macos.sh`: 235 test, 0 failure. Yeni üç handoff testi
  detached collector, typed hata izolasyonu ve local/ordinary-open ayrımını
  doğruluyor.
- `bash scripts/build-macos-app.sh`: exit 0; ad-hoc `NenPlayer.app` üretildi.
- Negatif kontrolde evidence çağrısı kaldırılınca hedefli NEN-082 testi exit 1
  ile kırmızı oldu; çağrı geri konunca aynı test yeşil oldu.
- `git diff --check` ve `bash scripts/check-docs.sh` kapanış öncesi yeniden
  çalıştırıldı; ayrıntılı checklist: `evidence/M4/NEN-082-checklist.md`.
