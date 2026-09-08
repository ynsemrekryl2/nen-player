---
adr: 0004
title: Async iş sahipliği, ilerleme ve cancellation delivery gate'i
status: accepted
milestone: M1
tasks: [NEN-009, NEN-090, NEN-093, NEN-100]
date: 2026-09-08
---

# ADR-0004 — Async iş sahipliği, ilerleme ve cancellation delivery gate'i

## Durum

`accepted`

## Bağlam

Uzun süren çeviri işleri platform kabuğunu bloklamadan çalışmalı, sayısal
ilerleme bildirmeli ve kullanıcı tarafından iptal edilebilmelidir. Ürün
şartnamesi cancellation sonrası **late commit**'i ve progressive/yarım altyazı
yayınını yasaklar (`docs/product-spec.md` §10, §16). M1'deki `NEN-009`
spike'ı daha sıkı sınırı da ölçtü: `cancel()` döndükten sonra hiçbir callback
gelmemeli ve geç sonuç teslim edilmemelidir.

`NEN-090`, ilk ürün provider portunu açarken bu sözleşmenin sahipliğini
netleştirmek zorundadır. Async yaşam döngüsü her adapter'a bırakılırsa gerçek
provider, deterministic mock ve ilerideki FFI katmanı ayrı worker/handle
modelleri üretir. Porta boxed future konursa bugün bulunmayan bir executor ve
async runtime seçimi public API'ye girer. Sözleşme yazılmazsa iptal bayrağını
gören bir adapter, o sırada havada olan callback'i veya sonucu yine de teslim
edebilir.

## Karar

1. **Async işin sahibi caller'dır.** Provider portu senkron, object-safe ve
   runtime bağımsız kalır. Üst seviye translation session çağrıyı uygun worker
   üzerinde yürütür; FFI `JobHandle` yüzeyi bu sahipliğin dış kabuğudur.
2. **Cancellation caller-owned bir delivery gate ile uygulanır.** Gate hem
   iptal durumunu hem progress/result teslimatını tek eşzamanlama sınırında
   korur. `cancel()` bu sınırı alır, gate'i kalıcı kapatır ve döner. Bir
   callback sürüyorsa `cancel()` onun bitmesini bekler; döndükten sonra hiçbir
   callback veya sonuç teslim edilemez.
3. **Adapter kooperatif checkpoint yapar.** Çağrı başlamadan, maliyetli iş
   sınırlarında ve cevap tesliminden hemen önce gate kontrol edilir. Kapalı
   gate tipli `Cancelled` hatasıdır; caller kapalı gate arkasından gelen bir
   adapter sonucunu ayrıca düşürür. Böylece kusurlu/geç adapter cevabı da late
   result üretemez.
4. **Progress yalnız şekil taşır:** kapalı bir faz enum'u ile `done`/`total`
   sayaçları. Sayaçlar monoton ve `done <= total` olur. Job kimliği provider
   portunun değil üst seviye session/FFI yüzeyinin sorumluluğudur. Cue metni,
   provider isteği/cevabı, URL, path veya serbest metin progress'e girmez.
5. **Callback ve result aynı gate'i paylaşır.** İptalden sonra ilerleme kadar
   sonuç/commit teslimi de yasaktır. Checkpoint ve kalıcı artifact yazımı
   sonraki katmanlarda aynı gate'in açık olduğunu doğrulamadan yapılamaz.
6. **Panik veya zehirlenmiş kilit güvenlik sınırını açmaz.** Gate kilidi
   zehirlenirse iç durum geri alınır; varsayılan davranış teslimata devam etmek
   değil, kapalı/cancelled saymaktır.

## Gerekçe

`NEN-009` delivery gate desenini iki platform binding'i üzerinden ölçtü.
`checkpoint_every=1` için cancellation p50 **33–37 µs**, 100 için yaklaşık
**2,02 ms** çıktı; `cancel()` döndükten sonraki callback sayısı ve late commit
sayısı sıfır kaldı. Bu karar spike kodunu ürün koduna terfi ettirmez; kanıtlanan
eşzamanlama ilkesini ürün portuna taşır.

Senkron port mevcut `HttpClient` ve `MediaIdentityLookup` portlarıyla aynı
object-safe yapıyı korur. Async scheduling'i caller'da tutmak, M5'te yalnız
deterministic mock varken runtime bağımlılığı eklemez ve `NEN-099`/`NEN-100`ün
tek iş yaşam döngüsünü sahiplenmesine izin verir.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Provider-owned worker ve `JobHandle` | Her adapter scheduling, thread ve shutdown politikasını tekrarlar; translation session ve FFI handle'ıyla iki sahip üretir |
| Boxed async future döndüren port | Object safety, executor ve runtime seçimini gerçek provider gerektirmeden public API'ye taşır |
| Yalnız atomik cancellation bayrağı | `cancel()` ile havadaki callback/result arasında karşılıklı dışlama kurmaz; çağrı iptalden sonra teslim edilebilir |
| Progress/result için ayrı gate'ler | İlerleme kapanırken sonucun açık kalabildiği yarış üretir; late commit yasağını tek değişmez olmaktan çıkarır |
| Adapter sonucuna güvenmek | Provider cevabı ve adapter davranışı untrusted kabul edilir; kusurlu adapter'ın geç cevabını caller'ın düşürebilmesi gerekir |

## Sonuçlar

**Olumlu:** cancellation sınırı tek yerde test edilir; port object-safe ve
runtime bağımsız kalır; mock ile gerçek provider aynı ilerleme/hata kontratını
taşır; FFI katmanı ikinci bir iş sahipliği modeli icat etmez.

**Olumsuz / kabul edilen maliyet:** senkron provider çağrısını worker'a almak
ve gate'i çağrı boyunca yaşatmak caller'ın sorumluluğudur. `cancel()` o anda
çalışan kısa bir callback'in dönmesini bekleyebilir.

**Geri dönüş maliyeti:** **orta.** Provider trait'i ve translation session
public sınırları değişir; gerçek provider'lar eklenmeden önce dönmek mümkün,
M6 sonrasında bütün adapter'ların eşzamanlama modeli etkilenir.

## İlgili task'lar

`NEN-009` · `NEN-090` · `NEN-093` · `NEN-099` · `NEN-100`

## Notlar

Karar, `NEN-009` spike'ının ürün kodunu değil ölçülmüş delivery-gate ilkesini
kullanır; spike `core/spikes/` altında kalır.

Karar maddeleri kullanıcı tarafından `NEN-090` uygulama planıyla birlikte
2026-09-08'de onaylandı; proposed dosya aynı maddelerle oluşturulup
karşılaştırıldıktan sonra `accepted` durumuna geçirildi.
