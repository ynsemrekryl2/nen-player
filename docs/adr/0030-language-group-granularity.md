---
adr: 0030
title: Subtitle menu language grouping and matching use the primary subtag
status: proposed
milestone: M3
tasks: [NEN-039]
date: 2026-08-25
---

# ADR-0030 — Subtitle menu language grouping and matching use the primary subtag

## Durum

`proposed`

## Bağlam

ADR-0010 accepted; onu düzenlemek yerine bu ADR onun Karar 4/5/9'unu
**daraltıyor** (ADR-0001 "Değişiklik" akışı).

ADR-0029 (NEN-020) Karar 5, metadata'nın **region**'ını bilinçli olarak
koruyor: `en-us` metadata + `en` metin tespiti çelişki sayılmıyor ve nihai
etiket `en-us` kalıyor. Bu, `LanguageTag`'in gerçek üretilebilir bir değeri.
Ama `nen-catalog`'un gruplama (`menu.rs`) ve otomatik seçim (`auto_select.rs`)
kodu **tam `LanguageTag`** eşitliğiyle çalışıyor:

- `menu.rs`, `BTreeMap<&LanguageTag, …>` ile grupluyor — `en` ve `en-us` **iki
  ayrı** `MenuGroup::Language` üretir.
- `auto_select.rs`, `s.language() == Some(language)` tam eşitlik karşılaştırıyor
  — tercihi `tr` olan kullanıcı, `tr-tr` etiketli bir track için otomatik
  seçim **alamaz**.

Elimizdeki kısıtlar:

- Product-spec §8: "diğer kaynaklar dillere göre" — dil kavramı, günlük
  kullanımda region'ı ayırt etmeyen bir kavram (kullanıcı "İngilizce altyazı"
  arar, "ABD İngilizcesi" değil).
- ADR-0010 Karar 4/9: tercih edilen diller menüde üstte ve otomatik seçimde
  kullanılıyor; bu mekanizmanın region farkına takılıp çalışmaması ürünün
  gerçek kullanım biçimini bozar.
- ADR-0029 Karar 5: region'ı **atmak** yerine korumak bilinçli bir karar
  (`pt-br` / `pt-pt`, `zh-hans` / `zh-hant` gerçek ve kullanıcının umursadığı
  farklar) — bu ADR o kararı geri almıyor, yalnız gruplama/eşleşme
  granülerliğini ayırıyor.
- Değişiklik yapılmazsa: aynı dilin iki metadata kaynağı menüde iki başlık
  açar (kullanıcı için anlaşılmaz görünür), ve region'lı bir metadata etiketi
  otomatik seçimi sessizce devre dışı bırakır — kullanıcı tercih ayarladığını
  sanır ama hiçbir şey otomatik açılmaz.

## Karar

**1. Grup anahtarı primary subtag'tir.** `MenuGroup::Language`, temsilci olarak
region'sız bir `LanguageTag` taşır (`LanguageTag::primary_tag()`). `en-us` ve
`en` metadata'lı kaynaklar **tek** grupta toplanır.

**2. Tercih ve otomatik seçim eşleşmesi primary subtag'tir.**
`SubtitlePreferences` bir dille eşleşirken (menü sırasında öne alma,
`auto_selection`'da tür önceliği) `LanguageTag::primary()` karşılaştırılır.
`tr` tercihi `tr-tr` kaynağı bulur.

**3. Kaynağın tam etiketi korunur.** `SubtitleSource::language()` hâlâ tam
`LanguageTag`'i (`en-us`) döndürür — bu ADR onu değiştirmiyor. Bir girişin
region'ını görünür kılmak (`"English (US)"`) platform UI'ının işidir
(NEN-026); çekirdek yalnız hangi kaynağın hangi region'a ait olduğunu
kaybetmeden taşır.

**4. Grup içi sıra değişmedi.** Ekleme sırası, öncelik taşımaz (ADR-0010
Karar 5) — bu ADR onu etkilemiyor.

## Gerekçe

Mevcut yardımcı zaten doğru sınırı çiziyordu: `nen-subtitle::language`'ın
`resolve_candidate`'i, metin tespitiyle metadata'yı karşılaştırırken zaten
primary subtag kullanıyordu (region farkının çelişki sayılmaması, ADR-0029
Karar 5). Aynı ayrımın katalog tarafında da gerekli olduğu, bu ADR'yi
gerektiren gözlemdi — yani iki kod yolu (dil çözümleme, katalog gruplama)
aynı soruyu ("bu iki etiket aynı dil mi?") farklı granülerlikte cevaplıyordu.
Bu ADR onları hizalıyor ve tek bir `LanguageTag::primary()` üzerinden ifade
ediyor.

Region'ı **atmamak** (bkz. Reddedilenler) ADR-0029'un kararını korurken,
gruplama ve eşleşmeyi **primary'ye indirmek** ürünün gerçek davranış
beklentisini karşılıyor: kullanıcı "İngilizce" arar, region bir alt ayrımdır.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| `LanguageTag`'i baştan primary-only yapmak (region'ı hiç saklamamak) | ADR-0029 Karar 5'i geri alır; `pt-br`/`pt-pt` gibi gerçek farkları kalıcı olarak kaybeder |
| Tam etiketle grupla, yalnız eşleşmeyi gevşet | Menüde "English" ve "English (US)" ayrı başlıklar olarak kalır — kullanıcı için anlaşılmaz; §8'in "dillere göre" ifadesiyle uyuşmaz |
| Region'ı UI katmanında normalize etmek (her platform kendi kuralını yazar) | Üç platformun aynı soruyu farklı cevaplama riski; ADR-0010 Karar 7'nin "sıralama çekirdekte tamamlanır" ilkesiyle çelişir |

## Sonuçlar

**Olumlu:** menü ve otomatik seçim, kullanıcının "dil" dediği şeyle hizalanıyor;
region bilgisi kaybolmuyor, yalnız gruplama/eşleşme seviyesinden iniyor.
`resolve_candidate`'in zaten kullandığı ayrımla tutarlı hale geliyor.

**Olumsuz / kabul edilen maliyet:** iki farklı region'lı kaynağın **aynı**
grupta görünmesi, kullanıcının hangi girişin hangi region'a ait olduğunu
etiketten (girişteki tam ad) okuması gerektiği anlamına gelir — NEN-026 bunu
girişte göstermek zorunda, ADR bunu garanti etmiyor.

**Geri dönüş maliyeti:** ucuz. Tek fonksiyon (`LanguageTag::primary`) ve iki
çağrı yerinde (`menu.rs`, `auto_select.rs`) değişiyor; mevcut iki golden
(region'sız fixture) değişmeden geçiyor.

## İlgili task'lar

`NEN-039`

## Notlar

<!-- karar sonrası gözlemler -->
