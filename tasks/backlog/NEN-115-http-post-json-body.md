---
id: NEN-115
title: HttpClient port carries POST with a JSON body
milestone: M6
size: M
state: backlog
closed:
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

- [ ] Unit (`nen-ports`): `post_json` alanları birebir; `FakeHttpClient` POST
      isteğini method+url+body ile eşliyor
- [ ] Swift test: `URLSessionRemoteEvidenceClient` POST'u gövde ve
      `Content-Type` ile gönderiyor (yerel `URLProtocol` stub — ağa çıkmıyor);
      `max_body_bytes` üstü cevap `BodyTooLarge` ile kesiliyor
- [ ] Negatif (zorunlu, K23): sentinel gövde ve `Authorization` header'ı
      `HttpRequest`/`FfiHttpRequest`/`HttpError` `Debug`'ında yok
      (`guard_*_debug.rs` emsali)
- [ ] Negatif: timeout'u dolan istek `HttpError::Transport` (veya ADR'nin
      varyantı) ile payload'sız dönüyor
- [ ] `NEN-036`/`NEN-072`'nin mevcut remote evidence testleri değişmeden yeşil
- [ ] `bash scripts/build-apple.sh` binding üretimi kırılmadı

## Kanıt kaydı

<!-- done olurken doldurulacak -->
