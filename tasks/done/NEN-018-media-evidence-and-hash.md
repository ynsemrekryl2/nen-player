---
id: NEN-018
title: Media evidence, OS-compatible hash, release name parser
milestone: M2
size: L
state: done
depends_on: [NEN-012]
blocks: [NEN-019, NEN-033, NEN-034, NEN-035, NEN-036]
adr: [9]
---

# NEN-018 — Media evidence, OS-compatible hash, release name parser

## Sonuç

Bir medya dosyasından kimlik kanıtları toplanır ve dosya adından başlık, yıl,
sezon, bölüm çıkarılır; çıkarılamadığında hata değil "bilinmiyor" döner.

## Kapsam

- `MediaEvidence`: filename, sunucunun beyan ettiği ad, boyut,
  OpenSubtitles-uyumlu hash, container metadata, release name, `.nfo` sidecar,
  klasör/kardeş/URL ipuçları, opsiyonel handoff metadata
- Release-name parser: film (title + year) ve dizi (series + S/E) desenleri
- ADR-0009'un katmanlı kanıt sırası (`resolve`) ve §6 fallback'i
  (`identity_candidates`)
- ≥30 gerçekçi dosya adı için golden

**ADR-0009 ile eklenen katmanlar** (`size: M` → `L` bu yüzden):

| Katman | Bu task'ta yapılan |
|---|---|
| Handoff metadata | Alan + sıralamadaki yeri |
| `.nfo` sidecar | Parser (XML + tek satır URL), IMDb/TMDb ID, başlık/yıl/S-E |
| Container metadata | Şekil + başlık/yıl çıkarımı (gerçek demuxer M3) |
| Dosya adı beyanı | Yerel ad **veya** sunucu beyanı; sanitization + testi (dolduran adapter NEN-036) |
| Üst klasör adları | `dir_hints`, yakından uzağa |
| Kardeş mutabakatı | Doğrular, karar üretmez |
| URL path segmentleri | Tüm segmentler, basename'den köke; query/fragment/host hariç |

## YAPILMAYACAK

- OpenSubtitles sorgusu → NEN-033 (M6)
- AI ile release-name normalizasyonu → NEN-034
- Güven skoru / aday sıralama → NEN-035
- Uzak kanıtları fiilen toplayan HTTP portu → NEN-036 (M3); burada yalnız alanlar
- Kullanıcıya düzeltme UI'ı → M3+ (burada yalnız model)
- Herhangi bir I/O — dosya okuma, ağ (ADR-0009 Karar 2)
- Media URL query'sinin kimlik kaynağı olarak kullanılması — **yasak** (§6)
- `.torrent` metadata'sı — **non-goal** (CLAUDE.md; ADR-0009 → Notlar)

## Kanıt (DoD)

- [x] Hash, bağımsız referans implementasyonuyla eşleşiyor (sentetik korpus +
      sınır vakaları: minimum altı → `Err`, tam eşik, eşik+1, tek byte değişimi)
- [x] ≥30 dosya adı golden: başlık/yıl/sezon/bölüm doğru
- [x] Çözümlenemeyen ad için hata değil `Unknown` dönüyor
- [x] ADR-0009 Karar 6'nın katman sırası test edildi: her katman tek başına
      kazanıyor, üstteki varken alttakiler yok sayılıyor, hiçbiri yokken `Unknown`
- [x] Negatif: URL query'si evidence'a **girmiyor** (test)
- [x] Negatif: sunucu beyanı adı sanitize ediliyor — `../`, kontrol karakteri,
      RFC 5987 `filename*` yüzde-encode'u yola dönüşmüyor
- [x] Negatif: evidence'ın `{:?}` çıktısı tam yol/hash/filename/NFO içeriği
      sızdırmıyor
- [x] Negatif kontrol: `#[derive(Debug)]`'lı kasıtlı bozuk fixture aynı deseni
      gerçekten sızdırıyor (denetim boşta dönmüyor)

## Kanıt kaydı

Ortam: Apple M5 · arm64 · macOS 27.0 · rustc/cargo 1.98.0 (workspace pin).
Tarih: 2026-08-25.

### Ne yazıldı

`nen-identity` boş bir iskeletti; dokuz modül eklendi, **yeni dış bağımlılık
yok** — `cargo tree -p nen-identity --edges normal` tek kenar gösteriyor
(`nen-domain`), `nen-domain` hâlâ tek düğüm. `core/deny.toml`'a dokunulmadı.

| Modül | İş |
|---|---|
| `os_hash.rs` | OSDb hash, I/O'suz: `of(file_size, head, tail)` + `of_bytes` |
| `release_name.rs` | Dosya adı → `ParsedName { title, year, season, episode, kind }` |
| `url_hints.rs` | URL path segmentleri, basename'den köke (ADR-0009 Karar 5) |
| `dir_hints.rs` | Üst klasör zinciri, yakından uzağa |
| `declared_name.rs` | `Content-Disposition` (RFC 6266 + RFC 5987) ayrıştırma ve sanitization |
| `siblings.rs` | Kardeş dosya mutabakatı (rapor eder, ezmez) |
| `nfo.rs` | Kodi/Plex sidecar: XML + tek satır URL biçimi |
| `container.rs` | `ContainerMetadata` şekli + tag → identity |
| `evidence.rs` | `MediaEvidence`, Karar 6 katman yürüyüşü, §6 aday üretimi |

### Test sayısı

Workspace **121 → 284** (+163, hepsi `nen-identity`), 1 `ignored`
(NEN-017'nin `lookup_bench`'i). Dağılım:

```
nen-identity lib          125
guard_evidence_debug        7
nfo_and_container           8
os_hash_reference           6
release_name_golden         4
resolution_layers          13
```

### DoD #1 — hash, bağımsız referansla eşleşiyor

Kanonik OSDb vektörleri 12.9 MB'lık gerçek video dosyaları; depoya konamaz.
Üç ayrı kanıt kullanıldı:

1. **Elle doğrulanabilir bilinen cevap** (referans implementasyonu hiç
   kullanmadan): tamamı sıfır olan 131 072 byte'lık dosyanın hash'i kendi
   boyutudur → `0000000000020000`. İlk kelime `1` yapılınca `…00020001`.
2. **Bağımsız referans implementasyonu** (`os_hash_reference.rs` →
   `reference_hash`): kasıtlı olarak farklı bir yoldan yazıldı — `as_chunks`/
   `from_le_bytes` yerine açık indeks aritmetiği ve elle bit kaydırma, hex
   basımı da formatlama yardımcısı yerine nibble aritmetiğiyle. 7 sentetik
   boyutta (sabit tohumlu xorshift64\*) ikisi birebir aynı.
3. **Commit edilmiş golden** (`fixtures/media/os-hash.golden`, 7 satır) — iki
   implementasyon birlikte değiştirilse bile wire biçimi değişimi yakalanır.

Sınır vakaları ayrıca test edildi: `0` · `1` · `CHUNK_BYTES` ·
`MIN_FILE_BYTES - 1` → hepsi `Err(TooSmall)`; `MIN_FILE_BYTES` tam → `Ok`;
head/tail pencerelerinin herhangi birinde tek bit çevrilince hash değişiyor
(5 pozisyon), **pencereler arasındaki byte değişince değişmiyor** — algoritmanın
asıl özelliği bu ve testle sabitlendi. Pencereli form (`of`) ile tam dosya formu
(`of_bytes`) birebir aynı sonucu veriyor — NEN-036'nın iki `Range` isteğinden
yerel okumayla aynı hash'i üretebilmesinin ön koşulu.

**Kapanışta bulunan gerçek kusur:** ilk sürüm hash'i `to_le_bytes()` ile
saklıyordu; OpenSubtitles'ın referans implementasyonları `%016x` ile *büyük
endian* basıyor. Elle doğrulanabilir sıfır-dosya vektörü bunu ilk koşuda
yakaladı (`0000020000000000` beklenen `0000000000020000` yerine) — yani wire
biçimi baştan yanlış olacaktı. `to_be_bytes()`'a geçildi; kelimeler hâlâ
little-endian toplanıyor, yalnız basım big-endian.

### DoD #2 — ≥30 dosya adı golden

`fixtures/media/release-names.tsv`: **42 ad** (≥30 şartı karşılandı),
`release-names.golden` 42 satır. Kapsam: 12 scene film · 4 parantezli yıl ·
11 dizi (SxxEyy, sNeN, NxNN, ayrı `S02 E05`, `Season 1 Episode 3`, başlıksız
`S01E02`) · 2 anime · 5 Türkçe/non-ASCII (Amélie, Ayla, Kış Uykusu, Das Boot) ·
**8 kasıtlı çözülemeyen**.

**Golden üç gerçek parser kusuru buldu** (hiçbiri elle fark edilmemişti):

| Girdi | İlk sonuç | Düzeltme |
|---|---|---|
| `Blade.Runner.2049.2017.1080p…` | başlık "Blade Runner", yıl **2049** | Arka arkaya iki yıl-benzeri token varsa ilki başlığın parçası, ikincisi yıl → "Blade Runner 2049" / 2017 |
| `[SubGroup] Steins Gate - 05 …` | başlık "SubGroup Steins Gate" | Baştaki `[Group]` etiketi atılıyor; kapanmamış `[` ile yanlış eşleşmemesi için aday `[` içeremez ve ≤40 karakter |
| `VID_20240115_143022.mp4` | `Unknown` ama başlık dolu | `Unknown` ⇒ `title = None` (değişmez hale getirildi) |

Dördüncü kusuru `dir-layouts` golden'ı buldu: `Severance (2022)/Season 02/`
`movie` dönüyordu — sezon var, bölüm yok. Sezon da artık dizi işareti sayılıyor
(sezon paketi `Show.S02.COMPLETE` de aynı kategoride).

### DoD #3 — çözülemeyen ad hata değil

`release_name::parse`'ın **hata tipi yok**; iddianın boş olmaması için korpusta
gerçekten pes edilen adların bulunduğu ayrıca test ediliyor: 42 addan **8'i**
`Unknown` (`video.mkv`, `stream.mkv`, `movie.mp4`, `0`, `a1b2c3d4e5f6a7b8.mp4`,
`8f2a9c1d`, `untitled`, `VID_20240115_143022.mp4`) ve hiçbiri `is_usable()`
değil. Ters yönde de bir kapı var: korpusun en az %75'i çözülmezse test kırılır
(bir parser her şeye pes ederek ilk testi geçebilirdi).

### DoD #4 — katman sırası

`resolution_layers.rs`, her katmanda **ayırt edilebilir bir başlık** taşıyan tek
bir evidence kuruyor (`HandoffLayer` … `UrlLayer`) ve üstteki katmanları teker
teker kaldırarak kazananın adını okuyor: handoff → nfo → container → dosya adı
→ klasör → URL path. Ayrıca "hiçbir alt katman üsttekini ezmedi" testi
kazananın çıktısında diğer beş adın hiçbirinin geçmediğini doğruluyor.

Boşluk doldurma ayrı test edildi: `Breaking Bad (2008)/S01E02.mkv` →
başlık+yıl klasörden, sezon+bölüm dosya adından, tür `Series`.

### DoD #5 — URL query'si evidence'a girmiyor

`url-hints.tsv` korpusunda (17 URL) kasıtlı olarak token'lı query, fragment ve
tanımlayıcı host var. Test her URL için evidence'ın `{:?}`'sini, hint listesini,
çözülen başlığı ve aday listesini tek stringde birleştirip
`SUPERSECRET · alice · token · auth= · example.com · 127.0.0.1` desenlerini
arıyor — hiçbiri yok. `guard_evidence_debug.rs` aynısını bir JWT'li gerçekçi
debrid URL'iyle tekrarlıyor ve **path'in işini yaptığını** da doğruluyor
(kimlik yine çıkıyor: "Inception" / 2010).

Yapısal kanıt: `MediaEvidence`'ın alanları private ve yalnız
`for_local_file`/`for_remote_url` üzerinden doldurulabiliyor — query'yi tutacak
bir alan **yok**, `path_hints` `?`/`#`'i her şeyden önce atıyor.

### DoD #6 — sunucu beyanı sanitize ediliyor

7 düşman girdi (`../../../etc/passwd`, `..\..\Windows\System32\config`,
`/etc/shadow`, `..`, `a/b/c/Movie.2010.mkv`, bidi override'lı ad, `\r\n`
enjeksiyonlu ad) evidence'a verilip çıktıda `/` · `\` · `\n` · `etc` · `passwd`
· `shadow` aranıyor — hiçbiri yok. RFC 5987 yüzde-encode'unun arkasına
gizlenmiş traversal (`filename*=UTF-8''%2e%2e%2f%2e%2e%2fpasswd`) da tek
segmente (`passwd`) iniyor. Bilinmeyen charset'te tahmin yürütülmüyor, düz
`filename`'e düşülüyor.

### DoD #7/#8 — redaction ve negatif kontrol

`guard_evidence_debug.rs` 10 yasak deseni (K23 #1/#2/#3/#8) kontrol ediyor:
`ynsemre` · `/Users/` · `Library` · `Inception` · `SUPERSECRET` ·
`eyJhbGciOiJIUzI1NiJ9` · `USER7781` · `private-host.example` ·
`ynsemre@example.com` · `tt1375666`. Yalnız `mkv` (uzantı) ve `huge` (boyut
sınıfı) basılıyor — ikisi de `security-policy.md`'nin "Loglanabilecekler"
listesinde; tam boyut (`9876543210`) basılmıyor.

**Negatif kontrol iki biçimde:**

1. **Kalıcı, CI'da koşan:** `DerivedEvidence` — aynı değerleri taşıyan,
   `#[derive(Debug)]`'lı kasıtlı bozuk ikiz. Test, onun 10 desenin **hepsini**
   artı tam boyutu gerçekten sızdırdığını doğruluyor. Sızdırmazsa test kırılır
   — yani "yasak desen yok" iddiasının boşta dönmediği sürekli kanıtlanıyor.
   (NEN-006'nın `BadFixtureWithDerivedDebug`'ı ve NEN-010'un sanitizing-
   constructor bypass'ıyla aynı biçim.)
2. **Tek seferlik mekanik doğrulama:** `MediaEvidence`'ın elle yazılmış
   `Debug`'ında `.field("file_name", &presence(…))` → `.field("file_name",
   &self.file_name)` yapıldı; `guard_evidence_debug` **2/7 testte kırmızıya
   döndü** (`K23 violation: "Inception" appeared in MediaEvidence { …
   file_name: Some("Inception.2010.1080p.BluRay.x264.mkv") … }`), dosya geri
   alındı, 7/7 yeşile döndü.

### Tasarım kararları (ADR-0009'un bıraktığı boşluklar)

**Boşluk doldurma yalnız sayısal alanlarda.** Karar 6 "ilk `Unknown` olmayan
katman kazanır" diyor; kazanan başlığı ve türü sahipleniyor, ama alt katmanlar
hâlâ eksik yıl/sezon/bölüm verebiliyor (`fill_gaps_from`). Başlık ve tür asla
ezilmiyor — aksi halde "kazanan" ifadesi anlamını yitirirdi.

**Bölüm veya sezon türü kesinleştirir.** Klasör adı bölüm işareti taşımadığı
için `Breaking Bad (2008)` tek başına `Movie` okunuyor; birleşince `Series`.
Sezonlu bir `Movie` döndürmek tutarsız olurdu.

**Kardeş mutabakatı ezmiyor, yalnız raporluyor.** Task açılışında "yanlış
`SxxEyy` yakalamalarını eler" yazmıştım; bu, ADR-0009'un "düşürmez" ilkesiyle
çelişiyordu. Daraltıldı: `corroborate` bir verdict döndürüyor (girdiyi
değiştirmediği ayrıca test ediliyor), `consensus_title` yalnız **eksik** bir
dizi adını dolduruyor. Yanlış bir çoğunluğun doğru bir çıkarımı ezmesi, elediği
hatadan kötü bir başarısızlık biçimi.

**Yerel dosya adı, sunucu beyanının önünde.** İkisi de katman 4; gerçek bir
yerel dosyanın kendi adı daha doğrudan bir beyan, sunucu beyanı yerel ad
yokken devreye giriyor.

### Doğrulama çıktıları

```
$ cargo test --manifest-path core/Cargo.toml --workspace
284 passed, 0 failed, 1 ignored              → exit 0 (121 → 284)
  ignored = lookup_bench (NEN-017 baseline)

$ cargo clippy --workspace --all-targets --manifest-path core/Cargo.toml -- -D warnings
                                              → exit 0, uyarı yok
$ cargo fmt --all --check --manifest-path core/Cargo.toml   → exit 0
$ (cd core && cargo deny check)
  advisories ok · bans ok · licenses ok · sources ok        → exit 0
  (deny.toml'a dokunulmadı — yeni bağımlılık yok)

$ cargo tree -p nen-domain --edges normal
nen-domain v0.1.0                             → tek düğüm, sıfır bağımlılık
$ cargo tree -p nen-identity --edges normal
nen-identity → nen-domain                     → tek kenar

$ bash scripts/check-docs.sh
  8/8 denetim geçti                           → exit 0
$ bash scripts/test.sh
  check-docs.test.sh ✓ · doctor.test.sh ✓     → exit 0
```

### Fixture envanteri

```
fixtures/media/release-names.tsv   42 ad     + .golden (42 satır)
fixtures/media/url-hints.tsv       17 URL    + .golden (17 satır)
fixtures/media/dir-layouts.tsv     12 yol    + .golden (12 satır)
fixtures/media/os-hash.golden       7 satır  (girdi sentetik, tohum kodda)
fixtures/media/nfo/                 4 dosya  (kodi-movie · kodi-episode ·
                                              url-only · malformed-unclosed)
```

Golden'lar `UPDATE_GOLDEN=1 cargo test -p nen-identity` ile yenilenir.

### Bilinen sınırlar (kabul edilen)

- `a%2Fb%2FInception.2010.1080p` segmenti başlığı "a b Inception" yapıyor —
  encode'lu `/` bilerek segment sınırı **oluşturmuyor**, düzleştiriliyor.
  Golden'da kayıtlı.
- `The.Office.US.3x07` → başlık "The Office US". Dosya adının söylediği bu.
- Release tag listesi kapsamlı değil, **sınır bulmaya** yetecek kadar; sonrası
  zaten atılıyor.
- Container demuxer'ı yok — `ContainerMetadata` şekli ve ayrıştırması var,
  gerçek etiketleri okuyan taraf M3'te (libmpv) bağlanacak.
