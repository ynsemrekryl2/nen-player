---
adr: 0046
title: Dosya adının AI destekli normalizasyona gönderilmesi — gizlilik ve izin modeli
status: accepted
milestone: M6
tasks: [NEN-124]
date: 2026-09-14
---

# ADR-0046 — Dosya adının AI destekli normalizasyona gönderilmesi

## Durum

`accepted` (2026-09-14, kullanıcı onayı, `NEN-124`)

## Bağlam

`NEN-018`'in deterministik release-name parser'ı `Unknown` döndürdüğünde
`NEN-034`, dosya adından başlık/yıl/sezon/bölüm çıkarımı için bir LLM fallback'i
öneriyor. Dosya adı K23 #8 kapsamında özel filename metadata'sıdır; bir
sağlayıcıya gönderilmesi loglamadan ayrı ve açık bir açıklama kararıdır.

Karar verilmezse AI çağrısı varsayılan olarak açılabilir, tam dosya yolu veya
URL ayrıntıları istemeden dışarı çıkabilir ve güvenilmez model cevabı kesin
kimlik gibi kullanılabilir.

## Karar

AI filename normalization yalnız deterministik kanıt katmanları `Unknown`
olduktan sonra ve kullanıcı o medya için tek seferlik açık izin verdikten sonra
çalıştırılacaktır. İstek, yerel veya sunucunun beyan ettiği dosya adının yalnız
sanitize edilmiş basename stem'ini (uzantı ve üst dizinler çıkarılmış) taşıyacak;
tam yol, uzantı, boyut, hash, içerik, container metadata'sı, subtitle diyaloğu,
URL host/path/query/fragment ve URL path segmentleri hiçbir koşulda taşınmayacaktır.
İstek kullanıcının seçtiği translation provider ve mevcut credential yolu
üzerinden gidecek, ayrı bir AI sağlayıcı seçimi veya Nen Player backend'i
olmayacaktır. LLM sonucu untrusted, düşük öncelikli `MediaEvidence` olarak
kalacak; daha güçlü kanıtları ezmeyecek, `MediaIdentity`'yi sessizce kesinleştirmeyecek
ve kullanıcı onayı olmadan otomatik seçim yapmayacaktır.

## Gerekçe

Varsayılan kapalılık ve medya-başına izin, filename metadata'sının her açılışta
veya kullanıcı başka bir işlem yaparken dışarı gitmesini engeller. Stem ile
sınırlamak, release-name parser'ın ihtiyaç duyduğu sinyali korurken uzantı,
dizin yapısı, URL token'ı ve medya parmak izini gereksiz yere açmaz. Aynı
translation provider yolunu kullanmak yeni credential, yeni veri işleyicisi ve
ayrı bir ayar yüzeyi eklemez; çağrı translation değildir ve `NEN-034`'ün
evidence katmanında kalır. `ADR-0019`'un bounded HTTP, `store: false` ve
payload'sız hata kuralları geçerlidir.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Varsayılan açık gönderim | Özel filename metadata'sını açık izin olmadan dışarı çıkarır; §16 ve K23 #8'in ruhuna aykırıdır |
| Kalıcı global “her medya için izin ver” ayarı | Kullanıcı niyetini yeni medyalara sessizce genişletir; izin kapsamını medya ile sınırlamak daha denetlenebilirdir |
| Tam yerel dosya adı ve uzantıyı göndermek | Uzantı ve biçim sinyali için tam değer gerekmez; gereksiz metadata paylaşılır |
| URL path segmentlerini veya query'yi göndermek | Query/fragment token taşıyabilir; URL path de medya geçmişini ve özel adları açığa çıkarabilir (ADR-0009) |
| Boyut, hash, container veya subtitle metnini aynı isteğe eklemek | Filename normalizasyonu için gereksizdir; K23 #4/#8 yüzeyini büyütür |
| Ayrı AI provider/backend seçimi | Credential ve veri işleyicisini çoğaltır; kullanıcının seçtiği provider yeterlidir |
| LLM cevabını kesin `MediaIdentity` saymak | Provider cevabı düşmancadır; yanlış eşleşme playback ve OpenSubtitles akışını sessizce bozabilir |

## Sonuçlar

**Olumlu:** Kullanıcı neyin, ne zaman ve hangi provider'a gittiğini bilir;
izin kapsamı dar, istek veri minimizasyonlu ve resolver sırası denetlenebilirdir.
AI fallback'i mevcut provider/credential ve redaction sınırlarını yeniden
kurmaz.

**Olumsuz / kabul edilen maliyet:** Her yeni medya için tekrar izin gerekir;
izin verilmezse AI normalization çalışmaz ve kullanıcı daha düşük seviyeli
aday/manuel akışa düşer. Stem sanitization bilgi kaybı yaratabilir; bu kabul
edilebilir, çünkü AI yalnız fallback'tir.

**Geri dönüş maliyeti:** ucuz. İzin akışı ve gönderilecek alan kümesi tek
uygulama kararında tutulur; ancak bir kez dışarı çıkmış filename metadata'sı
geri alınamayacağı için varsayılan veya kapsam genişletmesi yeni bir ADR ister.

## İlgili task'lar

`NEN-034` · `NEN-116` · `NEN-118` · `NEN-124`

## Notlar

- Kullanıcı izni, çeviri komutuna veya OpenSubtitles seçimine verilen izinden
  türetilemez; AI normalization kendi açık izin kapısına sahiptir.
- Log, `Debug`, crash ve artifact yüzeylerinde gönderilen stem veya LLM cevabı
  görünmez. Güvenli yüzey yalnız sonuç varyantı ve izin durumu gibi payload'sız
  sınıflardır.
- ADR-0009'un katman sırasına yeni türetilmiş katman eklenmesi aşağıdaki notta
  kayıtlıdır; bu ADR mevcut deterministic katmanların önceliğini değiştirmez.
