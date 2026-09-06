---
adr: 0039
title: Uzak medya HTTP sınırı ve macOS adapter'ı
status: accepted
milestone: M3
tasks: [NEN-036]
date: 2026-09-06
---

# ADR-0039 — Uzak medya HTTP sınırı ve macOS adapter'ı

## Durum

`accepted`

## Bağlam

`NEN-036`, kullanıcı veya handoff tarafından verilen uzak medya URL'inden
kimlik kanıtı toplar. Provider API'leri için yazılmış approved-host kuralı,
kullanıcının oynatmak istediği medya host'una uygulanırsa `file/http/https`
kapsamını ve localhost Stremio akışını bozar; medya isteği ile provider
isteği aynı güvenlik sınıfı değildir. Ayrıca uzak kanıt toplama redirect,
header ve `Range` doğrulaması ister. Bu politika adapter'a kopyalanırsa
platformlar arasında güvenlik davranışı ayrışır.

## Karar

Provider API'leri HTTPS ve sabit approved-host listesiyle sınırlı kalacaktır;
kullanıcı veya handoff medya URL'leri keyfi `http` veya `https` hostlarına,
localhost ve private ağlar dahil, gönderilebilecektir. Medya redirect zinciri
en fazla 5 adım sürecek, her hedef yalnız `http/https` olacak ve HTTPS'ten
HTTP'ye düşüş reddedilecektir. HTTP güvenlik politikası ve evidence toplama
akışı shared core'daki port/contract katmanında tutulacak; ilk gerçek adapter
macOS'ta Foundation `URLSession` olacaktır. Adapter redirect'leri otomatik
izlemeyecek, her cevabı core'un karar verdiği bir sonraki istek olarak
uygulayacaktır.

Container byte'larından Matroska/MP4 metadata ayrıştırması `NEN-036` içinde
yapılmayacak; bu iş ayrı bir task olarak ele alınacaktır.

## Gerekçe

Provider allowlist'ini medya URL'lerine uygulamak S5'i ve Stremio'nun
localhost senaryosunu reddederdi. Medya URL'i yine de yalnız iki scheme ile
sınırlanır; redirect hedefleri yeniden doğrulanır ve scheme downgrade kapatılır.
URLSession macOS'ta sistem API'sidir, yeni üçüncü taraf dependency getirmez ve
task'ın macOS adapter şartını karşılar. Core-owned policy sayesinde ilerideki
adapter aynı negatif contract testlerini paylaşır.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Medya URL'lerini provider approved-host listesine almak | Keyfi kullanıcı/Stremio hostlarını ve localhost medya akışını engeller |
| Medya için yalnız HTTPS kabul etmek | Kabul edilmiş S5 `http` kapsamını ve yerel Stremio HTTP akışını bozar |
| Redirect'leri URLSession'a bırakmak | Her adımın scheme ve redirect bütçesiyle doğrulanmasını core'dan çıkarır |
| İlk adapter olarak Rust HTTP/rustls seçmek | Yeni dependency/ADR yükü getirir ve NEN-036'nın macOS adapter kapsamını genişletir |
| Container parser'ı bu taska eklemek | Byte-level demux/parser ayrı güvenlik ve kapsam incelemesi gerektirir |

## Sonuçlar

**Olumlu:** Provider güvenliği ile kullanıcı medya erişimi ayrılır; redirect,
body ve redaction kuralları adapter'dan bağımsız kalır; URLSession ile yeni
üçüncü taraf dependency eklenmez.

**Olumsuz / kabul edilen maliyet:** Keyfi medya hostlarına istek gönderilir;
bu nedenle medya URL'leri kullanıcı girdisi olarak kabul edilir ve her redirect
adımı strict scheme kontrolünden geçer. İlk API senkron bir port olduğundan
URLSession adapter'ı çağrı başına bounded bekleme kullanır.

**Geri dönüş maliyeti:** orta — port ve contract korunarak adapter değişebilir;
ancak async FFI yüzeyi veya medya/provider politika ayrımı değişirse platform
ve security testleri birlikte güncellenmelidir.

## İlgili task'lar

`NEN-036`, `NEN-072`

## Notlar

Medyadan dönen URL, query, host, Content-Disposition filename ve hash hiçbir
log veya `Debug` çıktısında gösterilmez. `NEN-072`, ilk 64 KiB'dan container
metadata çıkarmanın ayrı kabul kriterlerini taşıyacaktır.
