---
adr: 0020
title: Secure credential storage haritası
status: accepted
milestone: M6
tasks: [NEN-110]
date: 2026-09-11
---

# ADR-0020 — Secure credential storage haritası

## Durum

`accepted` (2026-09-11, kullanıcı onayı, `NEN-110`)

## Bağlam

M6'nın üç kolu — OpenSubtitles kimlik sorgusu (`NEN-120`), OpenSubtitles
indirme (`NEN-121`…`NEN-123`), gerçek çeviri sağlayıcıları (`NEN-116`…`NEN-118`)
— kullanıcının kendi API anahtarına muhtaçtır ve hepsi tek bir credential
kapısına bağlanır. `NEN-033`'ün `OpenSubtitlesApiKey`'i bugün "credentials are
supplied by the platform secure-storage adapter later" diyor; o adapter'ın
sınırı henüz kararlaştırılmamış. Şartname §15 platform başına depoyu (Apple →
Keychain) ve secret kurallarını (loglanmaz · UI'da tam gösterilmez · artifact'e
girmez · crash/telemetriye girmez) zaten sabitliyor; `docs/security-policy.md`
§5 aynı yaşam döngüsünü tekrarlıyor. Karar verilmemiş olan **sınır**: port
nerede durur, adapter hangi tarafta (Swift/Rust), anahtar core'a nasıl ulaşır,
UI anahtarı bir daha okur mu.

`nen-ports` zaten üç örnek taşıyor — `HttpClient` (senkron, object-safe,
policy core'da), `ArtifactStore`/`persistence` (contract kiti + fake),
`PlaybackEngine` (capability tabanlı, fake adapter) — bu ADR aynı deseni
credential'a uygular.

## Karar

**1. Port.** `nen-ports::credentials` yeni bir `SecureCredentialStore` trait'i
tanımlayacaktır — senkron, object-safe (`HttpClient`/ADR-0004 emsali):
`get(kind) -> Result<Option<ApiKey>, CredentialStoreError>` ·
`contains(kind) -> Result<bool, CredentialStoreError>` ·
`set(kind, ApiKey) -> Result<(), CredentialStoreError>` ·
`delete(kind) -> Result<(), CredentialStoreError>`. `CredentialKind` kapalı bir
enum olacaktır: `OpenSubtitles` · `OpenAi` · `OpenRouter`; her varyantın sabit
bir slug'ı (`opensubtitles` · `openai` · `openrouter`) platform deposundaki
hesap adı olarak kullanılacaktır. Silinmiş veya hiç yazılmamış bir anahtar
hata değil `Ok(None)` dönecektir.

**2. Secret tipi.** Port düzeyinde, kind'dan bağımsız tek bir `ApiKey` newtype'ı
olacaktır. `OpenSubtitlesApiKey`'in (`NEN-033`) doğrulama kuralları — trim,
≤512 karakter, kontrol karakteri reddi — bu tipe taşınacaktır.
`Debug`/`Display` `<redacted>` yazacak, `Serialize` implementasyonu
**olmayacaktır**; değere tek erişim yolu açık isimli `expose()` metodu
olacaktır. `Drop` implementasyonu belleği best-effort sıfırlayacak, bunun için
yeni bir dış bağımlılık (`zeroize`) eklenmeyecektir. Provider'a özel mevcut
newtype'lar (`OpenSubtitlesApiKey`) adapter sınırında `ApiKey`'den kurulacak,
port bunları bilmeyecektir. Core hiçbir anahtarı global veya cache'lenmiş
durumda tutmayacaktır — bir iş (kimlik sorgusu, indirme, çeviri) kurulurken
okunacak, iş bitince adapter'la birlikte düşecektir.

**3. Hata tipi.** `CredentialStoreError` tipli ve payload'suz olacaktır:
`Unavailable` · `Denied` · `Corrupt`. Geçersiz giriş porta hiç ulaşmayacaktır
— `ApiKey::new` kendi doğrulamasında reddeder (`IdentityLookupError::
InvalidCredential` emsali). macOS Keychain hata eşlemesi: `errSecItemNotFound`
→ `Ok(None)`; `errSecAuthFailed` / `errSecInteractionNotAllowed` /
`errSecUserCanceled` → `Denied`; kayıtlı baytlar UTF-8 veya `ApiKey`
doğrulamasından geçmezse → `Corrupt`; diğer her Keychain hatası → `Unavailable`.

**4. Adapter sınırı.** Gerçek depo Swift tarafında olacak, core'a reverse-FFI
ile bağlanacaktır: `nen-ffi`'de `#[uniffi::export(with_foreign)] trait
ForeignSecureCredentialStore` (`ForeignHttpClient`, ADR-0039'un aynı deseni) +
düz `FfiCredentialKind` / `FfiCredentialError` enum'ları. Rust core hiçbir
platform secure-storage API'sine doğrudan bağlanmayacaktır. Kompozisyon kökü
tek yerde olacaktır (`PlaybackSessionClient` / `TranslationEnvironment`
emsali). **Tek yol**: ayarlar UI'ı da `nen-ffi`'nin dışa açtığı store objesi
üzerinden okuyacak/yazacak/silecektir — Swift Keychain adapter'ının tek
çağıranı Rust'tır, UI Keychain'e asla doğrudan gitmeyecektir.

**5. macOS deposu.** Login keychain, `kSecClassGenericPassword`; service sabit
uygulama bundle id'si (`player.nen.macos`), account = kind slug'ı;
`kSecAttrSynchronizable` **kullanılmayacaktır** (iCloud Keychain senkronu
kapalı — cloud sync non-goal); `kSecUseDataProtectionKeychain`
**kullanılmayacaktır**. Testler ayrı bir service adıyla (`player.nen.macos.test`)
gerçek Keychain'de koşacak ve kendi kayıtlarını temizleyecek, başka hiçbir
service'e dokunmayacaktır.

**6. UI sözleşmesi.** Giriş alanı maskeli (`SecureField`) olacaktır; kayıtlı
bir anahtar bir daha **okunmayacak ve gösterilmeyecektir** — yalnız `contains`
sorgusundan gelen "kayıtlı" göstergesi. "Değiştir" yeni değeri üzerine
yazacak, "sil" `delete` çağıracaktır. Boş veya yalnız whitespace giriş
kaydedilmeyecektir. ADR-0031 Karar 6'nın tek ayar penceresi üç yeni satırla
(OpenSubtitles · OpenAI · OpenRouter) genişleyecek, genel bir "Ayarlar" ekranı
açılmayacaktır.

**7. Test kuralı.** `InMemoryCredentialStore` fake'i `nen-ports::credentials`
içinde olacaktır (`FakeHttpClient`/`FakeEngine`/`FakeRenderer` emsali); bir
contract test kiti (`nen-ports::credentials::contract`) set→get birebir,
delete→get `None`, kind'ların birbirine karışmaması ve `contains`'in `get`
çağırmadan doğru cevap vermesini zorlayacaktır. K23 guard testi sentinel bir
anahtarla port ve FFI tiplerinin `Debug`/`Display` çıktısını tarayacaktır.
Gerçek Keychain yalnız `NEN-112`'nin Swift contract testinde koşacaktır
(Kural 8).

**8. Diğer platformlar.** Bu ADR yalnız haritayı çizer: Android → Keystore
destekli encrypted storage (M10), Windows → Credential Manager (M9/M11),
Linux → Secret Service (M9/M11); hepsi aynı `SecureCredentialStore` portunu ve
aynı contract kitini kullanacaktır. İmplementasyon bu ADR'nin kapsamı
**değildir**.

## Gerekçe

Portu `nen-ports`'ta tutmak, gerçek ve fake adapter'ın aynı contract'ı
paylaşmasını ve `HttpClient`/`ArtifactStore`'un zaten kanıtladığı deseni
tekrarlamayı sağlar. Tek `ApiKey` tipi üç kind için üç ayrı newtype'ın aynı
doğrulama/redaction kodunu tekrarlamasını önler; K23'ün redaction denetimi tek
tip üzerinde kalır. Adapter'ı Swift'te tutup Rust'a reverse-FFI ile bağlamak
ADR-0039'un zaten kanıtladığı deseni yeniden kullanır ve Rust'ın platform
secure-storage API'sine bağlanmasını gerektiren yeni bir `cargo deny` yüzeyi
(`security-framework` gibi bir crate) açmaz. Anahtarın **tek yoldan** —
Rust portundan — geçmesi, doğrulama ve redaction kuralının iki dilde ayrı ayrı
yazılıp ayrışma riskini taşımasını önler; UI'ın Keychain'e doğrudan yazması
aynı kuralı Swift'te ikinci kez uygulamayı gerektirirdi. Yalnız "kayıtlı"
göstergesi seçmek, secret'ın UI katmanına hiç inmemesini sağlar — şartname
§15'in "UI'da tekrar tam gösterilmez" kuralını en dar biçimde uygular; son
birkaç karakteri göstermek de kurala uyardı ama anahtarı her ayarlar
açılışında bir kez daha okuyup UI state'ine taşırdı, kazanç (hangi anahtarın
kayıtlı olduğunu ayırt etmek) `contains`'in zaten verdiği bilgiyle örtüşüyor.
Login keychain + generic password, macOS'un en sıradan API'sidir ve
`kSecUseDataProtectionKeychain`'in istediği application-identifier
entitlement'ını (takım imzası) gerektirmez — ADR-0034'ün zaten kabul ettiği
ad-hoc imzalı geliştirme build'i bu API ile çalışır; data-protection keychain
ad-hoc build'de `-34018` ile düşerdi.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Rust'ın Keychain'e doğrudan bağlanması (`security-framework` crate) | Yeni `cargo deny` yüzeyi açar ve ADR-0006'nın platform-adapter-Rust ayrımını (mevcut `HttpClient`/`PlaybackEngine` deseni) ihlal eder |
| Swift UI'ın Keychain adapter'ına doğrudan yazması | Doğrulama ve K23 redaction kuralı iki dilde tekrarlanır; tek doğrulama noktası bozulur |
| Anahtarı her iş başında Swift'in parametre olarak Rust'a geçmesi | Hangi kind'ın eksik olduğuna dair tipli ret politikası core'dan çıkar, port hangi anahtarın var olduğunu asla bilemez |
| Kind'a bağlı üç ayrı secret tipi | `OpenSubtitlesApiKey`'in doğrulama/redaction kodunu üç kez tekrarlar; K23 guard'ı üç yüzeyi ayrı ayrı taramak zorunda kalır |
| Kayıtlı anahtarın son birkaç karakterini göstermek | Anahtarı ayarlar her açılışında bir kez daha okuyup UI state'ine taşır; `contains`'in verdiği bilgiden fazlasını kazandırmaz |
| `kSecUseDataProtectionKeychain` (data-protection keychain) | application-identifier entitlement'ı (takım imzası) ister; ADR-0034'ün kabul ettiği ad-hoc imzalı geliştirme build'inde `SecItemAdd` `-34018` ile düşer |
| `kSecAttrSynchronizable` (iCloud Keychain senkronu) | Cloud sync ilk ürün için non-goal (S8); anahtarın cihazlar arası taşınması ayrı bir izin/gizlilik kararı gerektirirdi |
| `zeroize` crate'i | Yeni dış bağımlılık; best-effort `Drop` sıfırlaması aynı korumayı ek `cargo deny` yüzeyi olmadan verir |
| Anahtarı `UserDefaults`/plist/plaintext config'e yazmak | Şartname §15 açıkça yasaklıyor |

## Sonuçlar

**Olumlu:** Gerçek ve fake adapter aynı contract kitini paylaşır; secret tek
bir tipte, tek bir redaction kuralında toplanır; anahtar core'da hiçbir zaman
cache'lenmez; UI katmanı secret değerini hiç görmez.

**Olumsuz / kabul edilen maliyet:** Ad-hoc imzalı geliştirme build'inde her
imza değişikliğinde Keychain bir ACL istemi gösterebilir (`NEN-112` ölçer,
dağıtımda Developer ID ile kalıcı kalır — ADR-0034). Swift `String` ve Rust
`String`'in ikisi de bellekte güvenilir biçimde sıfırlanamaz; `Drop`
sıfırlaması yalnız best-effort'tur, realloc/kopya sırasında oluşan ara
kopyaları kapsamaz.

**Geri dönüş maliyeti:** ucuz — port ve contract kiti sabit kalır, yalnız
adapter değişir (Windows/Linux/Android eklenirken). FFI trait imzası
değişirse binding yeniden üretilir (`bash scripts/build-apple.sh`), ama
tüketen kod (`NEN-116`, `NEN-120`, `NEN-113`) porttan çağırdığı için etkilenmez.

## İlgili task'lar

`NEN-110` · `NEN-111` · `NEN-112` · `NEN-113` · `NEN-116` · `NEN-120` ·
`NEN-118`

## Notlar

Bu ADR yalnız macOS için implementasyon kapsıyor (Keychain); diğer
platformların depoları (§8) M9–M11'e ertelenmiştir, yalnız port ve kit
kararlaştırılmıştır.
