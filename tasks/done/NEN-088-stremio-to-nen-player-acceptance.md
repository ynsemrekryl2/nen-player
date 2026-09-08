---
id: NEN-088
title: Stremio to Nen Player acceptance
milestone: M4
size: S
state: done
closed: 2026-09-08
depends_on: [NEN-087]
blocks: []
adr: [44]
---

# NEN-088 — Stremio to Nen Player acceptance

## Sonuç

Stremio 5.1.26'da tek bir "MPV içinde oynat" eylemi, gerçek Nen Player
uygulamasını soğuk ve sıcak açılışta medyayı oynatır hâlde gösterir.

## Kapsam

- Gerçek Stremio 5.1.26 + gerçek Nen Player `.app` kabul koşusu
- Stremio'nun gönderdiği başlangıç değerinin aynen uygulanması; mevcut sürümün
  gönderdiği `0` değerinin baştan oynatma olarak kaydı
- Metadata yokluğunda akışın bozulmadığı senaryo
- Argüman, locator ve özel yol için negatif log taraması
- Ekran ve güvenli adım kaydı

## YAPILMAYACAK

- Stremio'dan mevcut oynatma konumunu tahmin etmek veya geri almak
- Stremio'ya sonuç/pozisyon döndürmek
- Gerçek medya URL'sini veya özel metadata'yı kanıta yazmak

## Kanıt (DoD)

- [ ] Soğuk açılışta gerçek Stremio eylemi Nen Player'ı açıyor ve oynatıyor
- [ ] Uygulama zaten açıkken aynı eylem doğru handoff yolunu kullanıyor
- [ ] Gelen nonzero pozisyon varsa uygulanıyor; ölçülen `0` davranışı kayda
      geçiyor
- [ ] Metadata yokken oynatma sürüyor
- [ ] Negatif log taraması tüm yasak parçalar için sıfır eşleşme veriyor
- [ ] `evidence/M4/NEN-088-checklist.md` ekran/adım kaydını içeriyor

## Kanıt kaydı

Kabul koşusu 2026-09-08'de Apple Silicon / macOS 27.0 üzerinde gerçek
Stremio 5.1.26 ve taze üretilmiş Nen Player `.app` ile tamamlandı.

- Soğuk handoff: Nen Player kapalıyken gerçek Stremio “MPV içinde oynat”
  eylemi uygulamayı açtı; transport şeridi oynatmayı `00:02` konumunda
  gösterdi: `evidence/M4/NEN-088-cold-transport.png`.
- Sıcak handoff: Nen Player açıkken aynı eylem mevcut süreci korudu
  (`warm_nenplayer_process_count=1`); transport şeridi oynatmayı `00:03`
  konumunda gösterdi: `evidence/M4/NEN-088-warm-transport.png`.
- Stremio 5.1.26'nın NEN-086'da ölçülen `--start=0` sözleşmesi bu koşuda
  baştan oynatma olarak gözlendi. Nonzero konum gelmedi; olmayan bir devam
  konumu tahmin edilmedi.
- Handoff metadata alanı olmadan oynatma sürdü; akış kesilmedi ve hata yüzeyi
  oluşmadı.
- Geçici unified-log ve stdout/stderr taraması (ham kayıt saklanmadan):
  `log_url_scheme_matches=0`, `log_query_token_matches=0`,
  `log_private_path_matches=0`, `log_handoff_arg_matches=0`,
  `stdout_stderr_capture_matches=0`.
- Önkoşullar: `bash scripts/doctor.sh M3` çıkış 0, köprü durumu `installed`,
  Stremio sürümü `5.1.26`, `bash scripts/build-macos-app.sh` çıkış 0.

Ürün kodu, public API ve şema değişmedi; kanıt ekranları yalnız transport
şeridini içerir, gerçek medya adı/URL'si veya özel metadata içermez.
