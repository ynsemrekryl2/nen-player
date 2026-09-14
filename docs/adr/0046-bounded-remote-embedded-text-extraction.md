---
adr: 0046
title: Uzak medyadan bounded gömülü altyazı metni çıkarımı
status: accepted
milestone: M6
tasks: [NEN-109]
date: 2026-09-14
---

# ADR-0046 — Uzak medyadan bounded gömülü altyazı metni çıkarımı

## Durum

`accepted` — 2026-09-14, M6 devam talebi kapsamında onaylandı.

## Bağlam

ADR-0045, yerel medyadaki gömülü subtitle track'inin
`libavformat`/`libavcodec` ile çıkarılmasını kabul etti ve uzak locator'ı
bilinçli olarak `Unsupported` bıraktı. M4 Stremio HTTP akışında aynı lazy
çeviri yolu kullanılmak istendiğinde, bütün uzak medyayı sınırsız ve
iptalsiz indirmek kabul edilemez: medya büyük olabilir, kullanıcı medyayı
değiştirebilir ve `cancel` çağrısı uzun bir demux'ü durduramayabilir.

`NEN-109`, uzak durumu küçük ve ölçülebilir bir genişletme olarak ele almayı
ister; yerel extraction davranışı ve core'un document parse sınırı
değişmemelidir.

## Karar önerisi

1. **Sabit byte bütçesi:** macOS adapter'ı uzak extraction için `GET` isteğini
   `Range: bytes=0-(8 MiB - 1)` ile gönderir. `Content-Length` veya
   `Content-Range` toplamı 8 MiB'den büyükse ve gövde bütçeyi aşarsa işlem,
   URL/path/başlık gövdesi taşımayan tipli `RemoteResponseTooLarge` reddiyle
   sonlanır. Bilinmeyen uzunlukta gövde de bütçeye ulaştığında kabul edilmez;
   böylece kesilmiş bir container başarı gibi görünmez.
2. **Tek hop ve kontrollü taşıma:** Foundation `URLSession`, cookie'leri
   kapalı ve redirect takip etmeyecek şekilde kullanılır. Gövde chunk'lar
   halinde toplanır; bütçe aşılınca task derhal cancel edilir. Yanıt tamamlanıp
   bütçe içinde kaldığında adapter, aynı mevcut local FFmpeg extractor'ına
   özel geçici dosya üzerinden verir ve dosyayı her durumda siler. Subtitle
   diyaloğu, URL, özel yol veya token loglanmaz.
3. **Gerçek iptal:** Extraction çağrısı sürerken adapter'da yaşayan iptal
   token'ı tutulur. Core session'ın `cancel_embedded_document_preparation`
   out-of-band çağrısı bu token'ı set eder ve URLSession task'ını cancel eder.
   `load` ve `shutdown` da etkin extraction'ı iptal eder. İptal sonucu UI'da
   yeni bir güvenlik/payload yüzeyi açmadan mevcut çeviri iptali olarak
   birleşir.
4. **Port sınırı:** `ShellEngine`/FFI engine'e yalnız preparation iptali için
   payload'suz, idempotent bir kontrol metodu eklenir; extraction metni yine
   playback adapter'ından core'a döner ve core `SubtitleDocument` parse eder.
   Provider, `nen-identity` ve remote evidence sınırları değişmez.
5. **Hata sınırı:** Byte bütçesi aşımı, `Unsupported` ile karıştırılmayan
   payload'suz `RemoteResponseTooLarge` varyantıdır. Diğer taşıma, HTTP veya
   FFmpeg sorunları mevcut engine failure sınıfında kalır; kullanıcıya
   yalnızca kapalı Türkçe cümle gösterilir.

## Alternatifler

| Alternatif | Neden önerilmedi |
|---|---|
| Sınırsız `avformat_open_input` ile doğrudan URL açmak | Büyük medya için ölçülebilir byte sınırı ve derhal iptal garantisi yok |
| Tüm medyayı önce kalıcı cache'e indirmek | Gereksiz disk saklama, cleanup/crash yüzeyi ve kullanıcı medyası yaşam döngüsü riski |
| Rust core'un remote demux yapması | ADR-0045'ün playback adapter I/O sınırını ve `nen-identity` saflığını bozar |
| Sadece mevcut `Unsupported` yanıtını korumak | M6'nın gerçek HTTP akışındaki lazy extraction kabul kriterini karşılamaz |

## Sonuçlar

**Olumlu:** Uzak extraction gerçek bir byte bütçesiyle ölçülür, iptal edilebilir
ve mevcut local decoder yolu yeniden kullanılır. Büyük akışlarda typed refusal
üretildiği için yanlışlıkla sınırsız indirme yapılmaz.

**Kabul edilen maliyet:** Subtitle stream'i ilk 8 MiB içinde değilse extraction
reddedilir; adapter geçici dosya yaşam döngüsünü yönetir; Swift/FFI sınırına
bir cancellation metodu ve yeni kapalı hata varyantı eklenir. Bu sınırın gerçek
Stremio HTTP stream'i, bounded başarı, cap reddi ve cancellation ölçümüyle
kanıtlanması NEN-109'un kapanış kanıtıdır.

## Onay kaydı

2026-09-14: Kullanıcının M6 tamamlanana kadar proje akışını sürdürme ve
gereken task/ADR adımlarını uygulama talebi, bu ADR'de önerilen bounded byte
bütçesi, iptal ve typed refusal kararları için onay olarak kaydedildi.
