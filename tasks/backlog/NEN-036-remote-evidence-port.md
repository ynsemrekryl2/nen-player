---
id: NEN-036
title: Remote media evidence port (HEAD, Content-Disposition, Range)
milestone: M3
size: M
state: backlog
depends_on: [NEN-018]
blocks: []
adr: []
---

# NEN-036 — Remote media evidence port (HEAD, Content-Disposition, Range)

## Sonuç

Uzak medyanın (http/https) kimlik kanıtları toplanır: sunucunun beyan ettiği
dosya adı, yönlendirme zincirinin sonu, boyut, ve `Range` ile çekilen ilk/son
64 KiB. NEN-018'in `MediaEvidence` alanlarını dolduran adapter budur.

## Kapsam

- Port trait'i (`nen-ports`) + macOS adapter
- `Content-Disposition` filename (RFC 6266 + RFC 5987 `filename*`), yoksa
  bounded redirect sonrası URL'in basename'i
- `Content-Length` + `Range: bytes=0-65535` / `bytes=-65536` → OpenSubtitles
  uyumlu hash ve container başlığı
- Sunucu `Content-Disposition` vermediğinde / `Accept-Ranges: none` dediğinde
  akışın bozulmaması
- Deterministic fake HTTP client

**Ön koşul — politika netleştirmesi (ADR-0009 → Notlar):**
`docs/security-policy.md` §3'ün "approved host — liste dışına istek atılmaz"
kuralı **sağlayıcı API'leri** için yazılmış. Kullanıcının veya Stremio'nun
verdiği medya URL'i keyfi bir host olabilir ve player onu açmak zorundadır. Bu
ayrım politikada açıkça yazmadan uzak kanıt toplayan kod yazılmaz.

## YAPILMAYACAK

- Medyanın kendisini indirmek — yalnız header ve iki 64 KiB pencere
- Query/fragment/host'u evidence'a koymak — **yasak** (§6, K23 #1/#2)
- Sunucu beyanı adını sanitize etmeden kullanmak
- Torrent/debrid acquisition — **non-goal**

## Kanıt (DoD)

- [ ] Fake client'la `Content-Disposition`'dan ad çıkıyor; yoksa yönlendirme
      sonundaki basename kullanılıyor
- [ ] `Range` ile hesaplanan hash, aynı içeriğin yerel hash'iyle birebir aynı
- [ ] `Accept-Ranges: none` ve `Content-Disposition` yokluğunda akış bozulmuyor
- [ ] Negatif: `../`, kontrol karakteri ve `filename*=UTF-8''…` içeren beyan
      adı yola dönüşmüyor, tek segmente indirgeniyor
- [ ] Negatif: URL, query ve host hiçbir log/`Debug` çıktısında yok
- [ ] Redirect sayısı sınırlı ve her adımda doğrulanıyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
