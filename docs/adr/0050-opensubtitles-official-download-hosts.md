---
adr: 0050
title: OpenSubtitles resmî indirme hostları
status: accepted
milestone: M6
tasks: [NEN-135]
date: 2026-09-17
---

# ADR-0050 — OpenSubtitles resmî indirme hostları

## Durum

`accepted` — 2026-09-17, kullanıcı onayı.

## Bağlam

ADR-0021, OpenSubtitles `POST /api/v1/download` yanıtındaki geçici linki
yalnız `api.opensubtitles.com`, `vip-api.opensubtitles.com` ve
`dl.opensubtitles.com` hostları için kabul eder. Canlı provider aynı resmî
indirme akışında `https://www.opensubtitles.com/download/...` linki de
döndürebilir. Bu link mevcut allowlist'e girmediği için Nen Player ağ isteği
yapmadan `RedirectRejected` döndürür.

Resmî destek örneklerinde `/api/v1/download` yanıtının
`www.opensubtitles.com/download/...` linki taşıdığı gösterilir; resmî API
dokümanı ise aynı alanı geçici download linki olarak tanımlar. Approved-host
listesini değiştirmek bir güvenlik kuralı gevşetmesi olduğu için ADR-0001
gereği yeni karar gerekir.

## Karar

OpenSubtitles geçici altyazı linki ve redirect zinciri için kabul edilen tam
host listesine `www.opensubtitles.com` eklenecektir. Bu hostta yalnız
`/download/` ile başlayan, boş olmayan HTTPS path'leri kabul edilecektir.
Mevcut `api.opensubtitles.com`, `vip-api.opensubtitles.com` ve
`dl.opensubtitles.com` hostları korunacak; tüm hostlar tam eşleşme, en fazla
beş hop, hop başına yeniden doğrulama, 10 MiB, content-type ve archive
kapılarına tabi kalacaktır. Credential header'ı geçici link isteğine
taşınmayacaktır.

ADR kabul edilirse ADR-0021'in §3 host listesi bu kararla supersede edilir;
arama, private `file_id`, kota ve seçimde indirme kararları değişmez.

## Gerekçe

`www.opensubtitles.com` provider'ın kendi resmî ve tam eşleşen hostudur;
wildcard veya registrable-domain eşleşmesi kullanılmaz. `/download/` path
kısıtı host genişlemesini provider'ın geçici dosya yüzeyiyle sınırlar. Böylece
canlı link çalışırken liste dışı host, alt alan adı, HTTP downgrade ve
credential sızıntısı reddedilmeye devam eder.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Tüm `*.opensubtitles.com` hostlarını kabul etmek | Gereksiz geniş SSRF/redirect yüzeyi açar; provider'ın verdiği bilinen hostlar tam eşleştirilebilir |
| URLSession'ın redirect'leri otomatik izlemesine izin vermek | Hop başına host ve HTTPS doğrulamasını core'dan çıkarır |
| `www` linkini `dl` hostuna yeniden yazmak | Provider'ın imzalı/geçici URL otoritesini değiştirir ve linki geçersiz kılabilir |
| Mevcut listeyi koruyup hatayı genel transport hatasına çevirmek | İndirmeyi düzeltmez ve gerçek policy reddini gizler |

## Sonuçlar

**Olumlu:** Resmî `www.opensubtitles.com/download/...` linkleri bounded
indirme akışından geçer; mevcut negatif güvenlik kapıları korunur.

**Olumsuz / kabul edilen maliyet:** Allowlist bir host genişler ve bu hostun
resmî indirme davranışı provider tarafında değişirse yeni bir karar/test
güncellemesi gerekir.

**Geri dönüş maliyeti:** ucuz — tek host/path kuralı ve ona bağlı provider
contract testleri geri alınır.

## İlgili task'lar

`NEN-119` · `NEN-122` · `NEN-126` · `NEN-135`

## Notlar

Resmî referanslar: [OpenSubtitles Download API](https://opensubtitles.stoplight.io/docs/opensubtitles-api/6be7f6ae2d918-download)
ve [resmî destek yanıtındaki `www` download linki](https://forum.opensubtitles.org/viewtopic.php?t=18314).

## Onay kaydı

2026-09-17: Kullanıcı `evet` mesajıyla ADR-0050 kararını onayladı.
