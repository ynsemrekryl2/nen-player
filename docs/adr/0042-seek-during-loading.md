---
adr: 0042
title: Yüklenirken verilen seek reddedilmez, ertelenir
status: accepted
milestone: M3
tasks: [NEN-052]
date: 2026-09-07
---

# ADR-0042 — Yüklenirken verilen seek reddedilmez, ertelenir

## Durum

`accepted`

## Bağlam

`ADR-0011` Karar 4 kontrat senaryolarını veri olarak tanımlıyor ve M3'ün çıkış
kriteri gerçek adapter'ın **aynı** kiti geçmesini şart koşuyor. Bugün kitin
hiçbir senaryosu yüklenirken (`.buffering`/`.loading`) seek etmiyor —
`NEN-051` sırasında yol üstünde ölçülüp `NEN-052` olarak dosyalandı.

Bu boşlukta iki motor **anlaşmıyor**:

- `FakeEngine::load` senkron olarak `Buffering → Ready`'ye geçiyor
  (`core/crates/nen-ports/src/playback/fake.rs`), yani fake'te "yüklenirken"
  diye gözlenebilir bir an yok — seek her zaman kabul ediliyor.
- `MPVPlaybackEngine.requireMedia()` `.loading` fazını **kabul ediyor**
  (`platforms/macos/Sources/NenPlaybackMPV/MPVPlaybackEngine+Internals.swift`),
  yani port seviyesinde meşru görünüyor. Ama gerçek mpv `seek` komutunu
  reddediyor: `check()` bunu `EngineFailure(code: -12)`'ye çeviriyor.
  `EngineFailure` motorun kendi sayısıdır (product-spec §4), kabuk bunu
  "beklenmeyen motor hatası" diye gösterir — oysa durum tamamen normal:
  kullanıcı dosya açılırken slider'ı sürüklemiş veya `→` tuşuna basmış.

`evidence/M3/NEN-052-measurement.md` gerçek adapter'da ölçtü: `load()`
döndükten sonra `state()` beş denemenin beşinde de `.buffering`, pencere
**2.5–12 ms** sürüyor — bir insanın tuşa basıp motoru yakalaması için
fazlasıyla dar, yani bu durum nadiren değil **her zaman** oluşabilir bir
yarış. Aynı pencerede `position()` de `EngineFailure(code: -10)` fırlatıyor
(`positionMs()`'in `try double("time-pos")` çağrısı `try?` değil `try`).

Karar verilmezse: kontrat kiti gerçek adapter'ı bu noktada hiç sınamaz,
kullanıcı arada bir gerçek bir motor hatası bildirimi görmeye devam eder ve
`NEN-051`'in bıraktığı boşluk kapanmaz.

## Karar

Başarılı bir `load`'dan sonra verilen bir seek, medyanın hâlâ açılıyor olması
**gerekçesiyle asla reddedilmez**. Yüklenirken verilen seek **tutulur** ve
medya `Ready`'ye ulaştığı anda (mpv'nin `FILE_LOADED` olayında) uygulanır;
çağırana giden cevap normal `SeekCompleted`'dır — kabul, motorun o an
yükleniyor olup olmamasına göre değişmez. Ard arda gelen birden çok ertelenmiş
seek'te yalnız sonuncusu uygulanır, ama çağıranın hak ettiği cevap sayısı
korunur (bugünkü `pendingSeeks` sayacının bir `playback-restart`'tan N cevap
ürettiği mekanizmanın aynısı). Yükleme başarısız olursa veya `stop`/`shutdown`
gelirse ertelenmiş seek sessizce düşürülür — `SeekCompleted` üretilmez, çünkü
seek'in hedeflediği yükleme artık yok.

Göreli seek (`seek_relative` = `position()` + `seek()`, ADR-0011 Karar 3) bu
kararla **değişmiyor**: `position()`'ın yüklenirken motor hatası fırlatması
önceden var olan, ayrı bir davranış ve bu task'ın kapsamı dışında (bkz.
Sonuçlar). Gerçek kullanıcı yüzeyinde göreli seek zaten kendi tuttuğu
pozisyondan mutlak `seek(to:)`'a gidiyor (`PlayerModel.seekRelative`), yani bu
kararın kapsadığı yoldan geçiyor.

## Gerekçe

**Neden erteleme, neden tipli ret değil.** Tipli ret kullanıcının isteğini
kaybeder — slider'ı sürükleyen veya `→`'a basan kullanıcı, isteğinin sessizce
yok sayıldığını görür ya da kabuk bunu ikinci bir "henüz hazır değil" mesajıyla
karşılamak zorunda kalır; ikisi de bugünkü "beklenmeyen motor hatası"ndan daha
iyi ama gerçek isteği yerine getirmiyor. Erteleme isteği tutar ve normal
akışın ürettiği aynı cevabı (`SeekCompleted`) verir.

**Neden bu aynı zamanda teknik bir zorunluluk.** Ölçülen 2.5–12 ms'lik pencere
o kadar dar ki "Load; hemen Seek" sırası kontrat senaryosunda **yarışa girer**:
bazen `.loading`'e bazen zaten `.ready`'ye denk gelir. Tipli bir ret
seçilseydi, senaryo hangi dala düşeceğini bilemeyeceği için deterministik
yazılamazdı. Erteleme ile iki dal da **aynı** gözlenen cevabı üretir — kabul +
sonunda hedefte iniş — yani senaryo motorun o an nerede olduğuna bakmaz.

**Neden port/adapter seviyesinde, kabuk seviyesinde değil.** ADR-0011 Karar 4
kontrat kitinin fake ile gerçek adapter'ı **aynı** listeyle sınamasını
zorunlu kılıyor. Erteleme kabukta (`PlayerModel`) yapılsaydı, `FakeEngine`
zaten anında `Ready`'ye geçtiği için kontrat kiti bu davranışı hiç sınamazdı —
tam olarak bugünkü boşluğun kendisi. Adapter seviyesinde tutmak, `NEN-022`'nin
"gerçek adapter, fake'in geçtiği kitin aynısını geçer" ilkesini bozmadan
kapatıyor.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Tipli ret (`NotLoaded` benzeri) | Kullanıcının isteğini kaybediyor; ölçülen dar pencere yüzünden kontrat senaryosu deterministik yazılamıyor (bazen kabul bazen ret). |
| Şimdilik yalnız belgele, davranış değiştirme | En ucuz ama kullanıcı gerçek bir motor hatası bildirimi görmeye devam eder ve fake/gerçek adapter anlaşmazlığı kapanmaz — `NEN-051`'in bıraktığı boşluk açık kalır. |
| Kabukta (`PlayerModel`) erteleme | Kontrat kiti `FakeEngine` üzerinde bu davranışı hiç göremez (fake zaten anında `Ready`), ADR-0011 Karar 4'ün "aynı kit" ilkesini fiilen boşaltır. |
| Kaldığı yerden devam (medya hazır olunca son bilinen konuma otomatik dön) | Bambaşka bir ürün özelliği — kullanıcı seek etmemişken bile devreye girer. Bu task erteleme *mekanizmasını* kurar, yeni bir özellik açmaz. |

## Sonuçlar

**Olumlu:** Fake ve gerçek adapter yüklenirken-seek konusunda artık aynı
cevabı veriyor; kontrat kiti bunu hem pozitif hem negatif yönde sınayabiliyor.
Kullanıcının yükleme sırasında verdiği seek isteği kaybolmuyor. Mekanizma
(`pendingSeeks` sayacı, bir restart'tan N cevap) zaten var olan koda ek bir
kavram getirmiyor.

**Olumsuz / kabul edilen maliyet:** Ard arda birden fazla erteleme yalnız
sonuncuyu uygular — ara isteklerin hedefi kaybolur, yalnız cevap sayısı
korunur. Bu, yüklenmemiş bir medyada ard arda seek etmenin zaten anlamsız
olmasından (henüz görülecek bir şey yok) kabul edilebilir bir maliyet.
`position()`'ın yüklenirken motor hatası fırlatması bu kararla **düzeltilmiyor**
— ayrı, önceden var olan bir davranış olarak kayıtlı kalıyor.

**Geri dönüş maliyeti: ucuz.** Erteleme tek bir alan (`deferredSeekMs`) ve
`FILE_LOADED` olayındaki bir dal; kaldırılması kontrat kitinin yeni
senaryosunu kırmızıya döndürür ama başka hiçbir yüzeyi etkilemez.

## İlgili task'lar

`NEN-051`, `NEN-052`

## Notlar

<!-- Karar sonrası gözlemler -->
