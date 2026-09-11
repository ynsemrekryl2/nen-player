---
id: NEN-115
title: HttpClient port carries POST with a JSON body
milestone: M6
size: M
state: done
closed: 2026-09-11
depends_on: [NEN-114, NEN-036]
blocks: [NEN-116]
adr: [19, 39]
---

# NEN-115 — HttpClient port carries POST with a JSON body

## Sonuç

`nen-ports::http::HttpRequest` bir JSON gövdeli `POST` ve istek zaman aşımı
taşıyabiliyor; `nen-ffi`'nin `FfiHttpRequest`'i ve macOS
`URLSessionRemoteEvidenceClient` adapter'ı bunu uçtan uca geçiriyor; gövde ve
header değerleri hiçbir `Debug` çıktısına düşmüyor.

## Kapsam

- `HttpMethod::Post`, `HttpRequest::post_json(url, headers, body, max_body_bytes,
  timeout)` (ADR-0019'un kararına göre alan adları); mevcut `head`/`range`/`get`
  yapıcıları değişmiyor
- `HttpRequest`'in elle yazılmış `Debug`'ı gövde uzunluğunu verir, içeriğini
  vermez (mevcut URL gizleme emsali)
- `FfiHttpRequest`'e `body: Option<Vec<u8>>` (veya `String`) + `timeout_ms`;
  `FfiHttpRequest`'in `Debug`'ı da redakte
- macOS `URLSessionRemoteEvidenceClient`: `httpBody`, `Content-Type:
  application/json`, `timeoutInterval`; mevcut `max_body_bytes` kesme
  davranışı POST cevabı için de geçerli
- Adapter'ın adı artık yalnız "remote evidence" değil — yeniden adlandırma
  **bu task'ta yapılmaz** (Kural 5 → gerekirse ayrı task), yalnız yorum notu

## YAPILMAYACAK

- Redirect izleme — trait'in kendi kuralı: uygulama katmanı her redirect
  hedefini doğrular; POST'ta redirect **izlenmez**
- Streaming cevap (SSE) — ADR-0019 structured output tek cevap
- Provider adapter'ları — `NEN-116`, `NEN-117`
- `FakeHttpClient`'ın davranışını değiştirmek — yalnız POST eşleşmesi eklenir

## Kanıt (DoD)

- [x] Unit (`nen-ports`): `post_json` alanları birebir; `FakeHttpClient` POST
      isteğini method+url+body ile eşliyor
- [x] Swift test: `URLSessionRemoteEvidenceClient` POST'u gövde ve
      `Content-Type` ile gönderiyor (yerel `URLProtocol` stub — ağa çıkmıyor);
      `max_body_bytes` üstü cevap `BodyTooLarge` ile kesiliyor
- [x] Negatif (zorunlu, K23): sentinel gövde ve `Authorization` header'ı
      `HttpRequest`/`FfiHttpRequest`/`HttpError` `Debug`'ında yok
      (`guard_*_debug.rs` emsali)
- [x] Negatif: timeout'u dolan istek `HttpError::Transport` (veya ADR'nin
      varyantı) ile payload'sız dönüyor
- [x] `NEN-036`/`NEN-072`'nin mevcut remote evidence testleri değişmeden yeşil
- [x] `bash scripts/build-apple.sh` binding üretimi kırılmadı

## Kanıt kaydı

### Uygulama

- `nen-ports::http`: `HttpMethod::Post` eklendi; `HttpRequest`'e `body:
  Option<Vec<u8>>` ve `timeout_ms: Option<u64>` alanları eklendi. Mevcut
  `head`/`range`/`get` yapıcıları davranış değiştirmeden her ikisini de `None`
  bırakıyor. Yeni `HttpRequest::post_json(url, headers, body, max_body_bytes,
  timeout_ms)` `Content-Type: application/json`'ı otomatik ekliyor;
  serialization (struct → JSON byte'ları) çağıranın sorumluluğunda kalıyor
  (ADR-0019 Karar 2). Elle yazılmış `Debug`, gövdeyi `body_length` olarak
  (`Option<usize>`), `timeout_ms`'i olduğu gibi basıyor — içerik hiçbir zaman
  yazdırılmıyor.
- `FakeHttpClient` değişmedi: eşleşme zaten tam `HttpRequest` eşitliğiyle
  yapıldığından (`method`+`url`+`headers`+`body`+... hepsi) POST istekleri
  otomatik olarak yeni alanlar dahil eşleşiyor — task'ın "yalnız POST eşleşmesi
  eklenir" notuyla tutarlı, kod değişikliği gerekmedi.
- `nen-ffi::remote_evidence`: `FfiHttpMethod::Post`, `FfiHttpRequest`'e
  `body`/`timeout_ms`, ilgili `From` dönüşümleri ve `Debug` (aynı
  body-uzunluk deseni) eklendi. `ForeignHttpClientAdapter` değişmedi —
  dönüşüm zaten `Into` üzerinden bu alanları taşıyor.
- macOS `URLSessionRemoteEvidenceClient`: `.post` case'i eklendi;
  `request.body` doluysa `urlRequest.httpBody`'e yazılıyor. `request.timeoutMs`
  doluysa hem `urlRequest.timeoutInterval`'a hem yerel tamamlanma beklemesine
  (`box.wait`) geçiyor — önceki sabit 30 sn bekleme yalnız `timeoutMs == nil`
  olan (mevcut HEAD/GET/range) istekler için korunuyor; ADR-0019'un 60.000 ms
  sağlayıcı zaman aşımı artık gerçek ağ beklemesini kısaltmıyor. Content-Type
  header'ı ayrıca özel işlenmedi — `post_json`'ın eklediği header, mevcut genel
  header döngüsünden geçiyor.
- Adapter adı bu task'ta değiştirilmedi (Kural 5); dosyanın kendi doc-comment'i
  hâlâ "remote evidence" diyor, POST artık yalnız evidence'a özgü değil —
  gerekirse ayrı bir yeniden adlandırma task'ı açılabilir.

### Test kanıtı

- `nen-ports::http` içinde dört yeni unit test: `post_json` alanlarının
  birebir kurulumu (method/url/body/max_body_bytes/timeout_ms/Content-Type
  header'ı), mevcut üç yapıcının `body`/`timeout_ms`'i `None` bıraktığı,
  `FakeHttpClient`'ın gövdesi farklı bir POST'u reddedip doğrusunu eşlediği,
  ve `Debug`'ın `body_length`/`timeout_ms` içerip gövde/`Authorization`
  içeriğini içermediği.
- Yeni K23 guard'ları: `nen-ports/tests/guard_http_debug.rs` (`HttpRequest`
  ve `HttpError` için) ve `nen-ffi/tests/guard_ffi_http_debug.rs`
  (`FfiHttpRequest` ve `FfiHttpError` için) — ikisi de sentinel gövde +
  `Authorization` header'ının `Debug`'da yokluğunu, ve kasıtlı
  `#[derive(Debug)]` ikizinin aynı sentinelleri sızdırdığını (guard sağır
  değil) kanıtlıyor.
- Swift: `URLSessionRemoteEvidenceClientTests`'e üç yeni test —
  `theAdapterSendsAPostBodyWithJsonContentType` (yerel `URLProtocol` stub,
  gerçek `httpBody`/`Content-Type`/`Authorization`'ı doğruluyor, ağa çıkmıyor),
  `aResponseOverTheBodyLimitIsCutOffWithResponseTooLarge` (mevcut
  `max_body_bytes` kesmesinin POST cevabında da çalıştığını doğruluyor — task
  metninin "BodyTooLarge" ifadesi mevcut `ResponseTooLarge` varyantına
  karşılık geliyor, yeni varyant açılmadı), ve
  `aRequestThatOutlivesItsTimeoutFailsWithTransportAndNoPayload` (zorunlu
  negatif — 100 ms `timeoutMs` ile hiç yanıt vermeyen bir `StubURLProtocol`,
  gerçek 0.1 sn'lik bekleme sonunda `FfiHttpError.Transport` ile payload'sız
  düşüyor).
- **Sağırlık kontrolü (üç ayrı mutasyon, hepsi elle geri alındı):**
  `post_json`'daki `Content-Type` ekleme satırı kaldırılınca yalnız
  `post_json_builds_a_post_request_with_body_content_type_and_timeout`
  kırmızıya döndü; `HttpRequest`'in `Debug`'ı `body_length` yerine ham
  `body`'yi basacak şekilde değiştirilince hem ilgili unit test hem
  `guard_http_debug.rs`'in K23 testi kırmızıya döndü (69/70 → 1 fail, ayrı
  guard binary'sinde 1/2 → 1 fail); Swift adapter'da `urlRequest.httpBody =
  body` satırı kaldırılınca yalnız `theAdapterSendsAPostBodyWithJsonContentType`
  kırmızıya döndü (`swift test` exit 1, 2 issue) — kontrol sağır değil.
- `cargo test --workspace` tamamı yeşil (yeni testler dahil, 0 failed);
  `NEN-036`/`NEN-072`'nin mevcut `nen-app::remote_evidence` testleri (14 test)
  değişmeden yeşil kaldı. `bash scripts/test-macos.sh` **267 passed / 33
  suites** (önceki `NEN-104` baseline 264 + bu task'ın 3 testi), tam koşu
  yeşil. fmt, clippy (`-D warnings`), `cargo deny check` (yeni dış bağımlılık
  yok — yalnız workspace içi değişiklik), `bash scripts/test.sh` **4/4** ve
  `bash scripts/build-apple.sh` (binding üretimi kırılmadı, `FfiHttpRequest`
  yeni alanlarla üretildi) hepsi yeşil.
