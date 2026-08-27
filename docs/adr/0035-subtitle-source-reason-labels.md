---
adr: 0035
title: Altyazı kaynağı sebep etiketleri — kapalı küme iki elemanlıdır
status: accepted
milestone: M3
tasks: [NEN-056, NEN-026]
date: 2026-08-27
---

# ADR-0035 — Altyazı kaynağı sebep etiketleri: kapalı küme iki elemanlıdır

## Durum

`accepted`

## Bağlam

[ADR-0031](0031-macos-shell-interaction-model.md) Karar 5 iki madde taşıyor ve
ikisi aynı anda doğru olamaz:

- Birinci madde, **kataloğa giren ama kullanılamayan** kaynağın taşıyacağı
  kapalı etiket kümesini sayıyor: `okunamadı` · `biçim hatalı` · **`çok büyük`**.
- İkinci madde, boyut sınırını **güvenlik kapısına** koyuyor
  (`security-policy.md` §4 #4); kapıdan dönen dosya kataloğa **hiç girmiyor**.

Boyut kapıda eleniyorsa, katalogdaki bir kaynağın `çok büyük` etiketi taşıması
mümkün değildir. Çelişki `NEN-025` uygulanırken görüldü ve `NEN-056` olarak
ayrıldı (Kural 5).

**Bugünkü kod ikinci maddeyle tutarlı.** `nen_app::subtitle_files::admit`
sınırı, dosya açılmadan önce elde olan `symlink_metadata`'ya soruyor ve aşan
dosyayı `FileRejection::TooLarge` ile geri çeviriyor — kataloğa girmiyor, hiç
okunmuyor. Sınır ayrıca icat edilmiş bir sayı değil:
`MAX_SUBTITLE_BYTES == nen_subtitle::encoding::MAX_INPUT_BYTES` (10 MiB,
NEN-015) ve iki sınırın ayrışmasını bir test engelliyor. Yani tutarsız olan
**kod değil, doküman**.

Karar verilmezse `NEN-026` üretilemeyecek bir durum için UI yazar: menüde hiçbir
zaman görünmeyecek bir etiketin görsel tasarımı, yerleşimi ve testi.

`docs/product-spec.md` §4 (altyazı sorunu playback'i durdurmaz) ve §7–§8
(katalog ve tek menü) bu ADR'nin sınırlarını çiziyor.

## Karar

**Karar 1 — Boyut bir güvenlik kapısıdır.** `security-policy.md` §4 #4'ün
sınırını aşan dosya **okunmaz**, kataloğa **girmez**, menüde izi olmaz.
Kullanıcı dosyayı **kendisi seçtiyse** ADR-0031 Karar 1'in *geçici* sınıfından
bildirim alır ("Bu altyazı dosyası çok büyük."); sidecar taramasında sessizce
elenir (ADR-0031 Karar 5, ikinci madde — değişmiyor).

**Karar 2 — Kapalı etiket kümesi tam olarak iki elemanlıdır.** Kataloğa giren
fakat kullanılamayan kaynağın menüde taşıyacağı sebep etiketleri:

| Etiket | Çekirdek karşılığı | Ne zaman |
|---|---|---|
| `okunamadı` | `SourceDefect::Unreadable` | Baytlar okunamadı veya metne çözülemedi (ADR-0008) |
| `biçim hatalı` | `SourceDefect::Malformed` | Metin geçerli SRT değil (NEN-013 katı parser) |

`çok büyük` bu kümede **değildir** ve ADR-0031 Karar 5'in birinci maddesinden
düşer.

**Karar 3 — Kümeye eleman eklemek yeni bir ADR ister** ve eklenen her elemanın,
onu gerçekten üreten bir testi bulunmak zorundadır. Üretilemeyen etiket
yazılmaz.

**Karar 4 — ADR-0031'in diğer beş kararı yürürlüktedir.** Bu ADR yalnız Karar
5'in **birinci maddesinin etiket kümesinin** yerine geçer; Karar 5'in geri
kalanı (hatalı kaynak menüde kalır, soluk ve seçilemez gösterilir; güvenlikten
dönen dosya kataloğa hiç girmez) aynen geçerlidir.

## Gerekçe

**Neden "boyut kapıdır" yarısı seçildi.** Bugünkü kod, `NEN-025`'in dört kapı
testi ve `.app` üzerinde alınan elle kanıt bu yarıyı zaten uyguluyor ve
kanıtlıyor (`an_oversized_file_is_refused_without_being_opened`; 11 MiB'lık
dosya elle yüklendiğinde geçici bildirim çıktı ve oynatma sürdü). Diğer yarıyı
seçmek çalışan ve kanıtlanmış davranışı geri almak olurdu.

**Ölçülen sayı kararı destekliyor.** Bu depoda ölçülmüş en büyük gerçek altyazı
belgesi M1'in 50 000 cue'luk dokümanı: **3.1 MiB** — ve o zaten tek bir filmin
üretebileceğinin çok ötesinde. Sınır 10 MiB, yani ölçülen en büyük belgenin 3
katından fazlası. Kapının eleyeceği dosya, pratikte altyazı olmayan bir
dosyadır.

**Kapının konumu bilinçli.** Sınır `stat`'a soruluyor, dosya açılmadan önce.
§4 #4'ün "boyut sınırını aşan dosya **okunmaz**" ifadesinin tek dürüst
uygulaması bu: bandın önce tamamen belleğe alınıp sonra reddedilmesi, kuralın
önlemek istediği şeyin ta kendisi.

**Karar 3'ün sebebi bu ADR'nin kendi hikâyesi.** ADR-0031 Karar 5'in üçüncü
etiketi, onu üretecek bir kod yolu hiç olmadığı için yazıldığı gün ölüydü ve
bunu ancak `NEN-025` uygulanırken fark ettik. Etiketi teste bağlamak, aynı
hatanın sessizce tekrarlanmasını engelliyor.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Boyut aşımı kataloğa `çok büyük` işaretli, seçilemez bir satır olarak girsin | `NEN-025`'in kapıları değişir; dahası ADR-0031 Karar 1'e göre kaynak-düzeyi hata **yalnız menüde** görünür, yani kullanıcının kendi seçtiği büyük dosya artık geçici bildirim almaz — `NEN-025`'in `.app` üzerinde kanıtladığı davranış geri alınır. Kazanç yalnız 10 MiB üstü bir sidecar için, ölçülen en büyük gerçek altyazının 3 katından büyük bir dosya için |
| ADR-0031'i bütünüyle `superseded` yapıp altı kararı yeniden yazmak | `adr: [.., 31]` referansı taşıyan **7 done task** var (`NEN-024` · `NEN-025` · `NEN-046` · `NEN-048` · `NEN-053` · `NEN-054` · `NEN-055`); `scripts/check-docs.sh` adım 6 done bir task'ın ADR'si `accepted` değilse kırmızıya döner. Beş sağlam kararı, bir etiket için sirkülasyondan çekmek |
| ADR yazmadan yalnız ADR-0031'in Notlar'ına düzeltme eklemek | Menüde görünen sebep kümesi **kullanıcı-görünür davranış sözleşmesidir**; ADR-0001'in "ADR gerekir" listesi bunu açıkça sayıyor. Karar dipnotta yaşarsa altı ay sonra Karar 5'in gövdesini okuyan kişi yanlış kümeyi uygular |
| ADR-0031 Karar 5'in gövdesini düzeltmek | ADR-0001: kabul edilmiş ADR düzenlenmez. Geçmişin ne dediği kaydın bir parçası |
| Boyut sınırını yükseltip sorunu ortadan kaldırmak | Sınırı belirleyen şey menü etiketi değil, okunmadan reddetme kuralı; yükseltmek `MAX_INPUT_BYTES` ile paylaşılan tek sınırı da bozar (NEN-015) |

## Sonuçlar

**Olumlu:**

- `NEN-026` yalnız üretilebilen iki etiket için UI yazar; ölü bir durumun
  tasarımı, yerleşimi ve testi hiç yazılmaz.
- Doküman koda uyuyor: ADR-0031 Karar 5 ↔ ADR-0035 ↔ `NEN-026` kapsamı ↔
  `subtitle_files.rs` tek şeyi söylüyor.
- Karar 3, kapalı kümeyi kanıta bağlıyor — bir etiket ancak onu üreten bir test
  varsa var olabilir.

**Olumsuz / kabul edilen maliyet:**

- 10 MiB'den büyük bir **sidecar** kullanıcıya hiç görünmez; neden
  çalışmadığını menüden öğrenemez. Sessizlik bilinçli (ADR-0031 Karar 5):
  tarama kullanıcının eylemi değildir ve reddedilen bir yolun varlığını menüde
  teyit etmek istemiyoruz. Kullanıcı aynı dosyayı **kendi** yüklerse sebebi
  geçici bildirimde görür.
- Kapalı küme iki elemana indiği için, ileride kataloğa giren yeni bir kusur
  sınıfı (örneğin M6'da bozuk bir indirme) kendi ADR'sini gerektirecek.

**Geri dönüş maliyeti: orta.** Dönmek, boyut kararını `admit`'ten çıkarıp
`SourceDefect`'e üçüncü bir varyant eklemek (`nen-app` · `nen-ffi` · menü UI)
ve ADR-0031 Karar 1'in geçici bildirim yolunu bu dosya sınıfı için kaldırmak
demek. Port kontratına ve domain modeline dokunmuyor.

## İlgili task'lar

`NEN-056` · `NEN-026` · kapıların sahibi `NEN-025`

## Notlar

<!-- Karar sonrası gözlemler buraya. -->
