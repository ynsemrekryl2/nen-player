---
id: NEN-109
title: Remote embedded text extraction
milestone: M6
size: M
state: done
closed: 2026-09-14
depends_on: [NEN-044]
blocks: []
adr: [0046]
---

# NEN-109 — Remote embedded text extraction

## Sonuç

Bir Stremio HTTP akışının (M4) gömülü bir metin altyazı track'i, `NEN-044`'ün
yerel-dosya yolunun kapsam dışı bıraktığı durumda, sınırlı ve iptal edilebilir
bir okumayla çıkarılabilir.

## Bağlam

`NEN-044`, ADR-0045'in demux yolunu (`libavformat`/`libavcodec`) yalnız
**yerel** medya için uyguladı: `MPVPlaybackEngine.extractText`, locator'ı
yerel bir yol değilse (`/` ile başlamıyorsa) `Unsupported` ile döner ve
`EmbeddedTextExtractor` hiç çağrılmaz. Bu, kullanıcı kararıyla bilinçli bir
kapsam sınırıydı — gerekçe:

- `avformat_open_input` bir uzak URL'yi açabilir, ama tüm container'ı demux
  edip bir alt akışı okumak GB'larca veri indirmek anlamına gelebilir.
- Bugünkü çağrı yolunun (`PlaybackSession::prepare_embedded_document`) hiçbir
  iptal mekanizması yok — `NEN-093`'ün çeviri işi için kurduğu
  `TranslationCall` gate'i burada devrede değil.
- Ölçülmüş bir üst sınır (kaç MB okunacak, ne zaman vazgeçilecek) yoktu;
  varsayımla bir sınır koymak yerine kapsam dışı bırakıldı.

## Kapsam

- Uzak medya için sınırlı/aralıklı okuma stratejisi (tasarım kararı — ADR
  gerekebilir: `avio` özel I/O callback'i ile bounded byte-window mi,
  yoksa `NEN-072`'nin uzak evidence yolunun genişletilmesi mi)
- İptal edilebilirlik: kullanıcı çeviri komutunu iptal ederse veya medya
  değişirse okuma durur
- Üst sınır: ne kadar indirilebileceğinin ölçülmüş, dokümante edilmiş bir
  tavanı
- `MPVPlaybackEngine.extractText`'in yerel-dosya kapısının genişletilmesi

## YAPILMAYACAK

- Sınırsız/iptal edilemez tam indirme — `NEN-044`'ün bilerek reddettiği yol
- Yerel dosya yolunun davranışını değiştirmek

## Kanıt (DoD)

- [x] Gerçek bir Stremio HTTP akışında gömülü metin track'i çıkarılabiliyor
- [x] İptal, sürmekte olan bir okumayı gerçekten durduruyor (gerçek testle
      ölçülmüş)
- [x] Üst sınır aşıldığında tipli bir red — sessiz kesme yok
- [x] `NEN-044`'ün yerel-dosya testleri değişmeden yeşil kalıyor

## Kanıt kaydı

Kanıt: `evidence/M6/NEN-109-checklist.md`. Gerçek Stremio EngineFS HTTP proxy
akışı üzerinden embedded English track extraction geçti; bounded Range/cap,
typed refusal, measured URLSession cancellation ve NEN-044 local regression
testleri yeşil.
