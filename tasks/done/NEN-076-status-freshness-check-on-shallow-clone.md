---
id: NEN-076
title: The STATUS freshness check reads every done task as today on CI
milestone: M3
size: S
state: done
closed: 2026-09-07
depends_on: [NEN-040]
blocks: []
adr: []
---

# NEN-076 — STATUS güncellik denetimi CI'da her done task'ı bugün sanıyor

## Sonuç

`check-docs.sh` adım 9, done task'ların gerçek kapanış tarihini CI'da da okur;
aynı ağaç yerelde geçip CI'da kalamaz.

## Bağlam

`NEN-075` çalışılırken ölçüldü. Yalnız ADR dosyaları içeren bir commit
(`1ad194d`, 2026-09-07) CI'ı kırdı:

```
HATA  docs/STATUS.md Son doğrulama tarihi 2026-09-06;
      NEN-074 (2026-09-07) daha yeni.
```

Oysa `NEN-074`'ün kapanış commit'i **2026-09-06T20:19:36+03:00**. Aynı denetim
yerelde `ok` veriyordu.

**Sebep shallow clone.** `.github/workflows/ci.yml` `actions/checkout@v4`'ü
varsayılan `fetch-depth: 1` ile kullanıyor, yani CI'da geçmişte tek commit var.
Adım 9 her done task için

```sh
git log -1 --format=%cs -- "$f"
```

çağırıyor; tek commit'lik bir geçmişte o commit **bütün** dosyaları eklemiş
görünür, dolayısıyla her done task **HEAD'in** tarihini bildirir. Yerelde
yeniden üretildi:

```
$ git clone --depth 1 file://…/nen-player shallow
$ git -C shallow log -1 --format=%cs -- tasks/done/NEN-074-smooth-live-resize.md
2026-09-07          # gerçek kapanış: 2026-09-06
$ git -C shallow log -1 --format=%cs
2026-09-07
```

Yani denetim CI'da aslında şunu soruyor: *"STATUS'un Son doğrulama tarihi
HEAD'in commit tarihinden küçük mü?"* — kastettiği soru bu değil.

**Neden bugüne kadar görünmedi:** bugüne kadar her commit ya STATUS'u da
güncelliyordu ya da STATUS'un tarihiyle aynı gün atılıyordu. `1ad194d` ilk kez
**yeni bir günde, STATUS'a dokunmayan** bir commit oldu. Bu yüzden her ADR-only
ve her tooling commit'i, atıldığı gün STATUS güncellenmemişse CI'ı kıracak.

`NEN-075`'in kapanış commit'i STATUS'u 2026-09-07'ye taşıdığı için CI o
commit'te yeşile döner — ama kusur durur ve bir sonraki yeni-gün commit'inde
aynen tekrarlar.

## Karar verilecek

1. `fetch-depth: 0` — CI tam geçmişi çeksin. Tek satır; bedeli klonlama süresi.
2. Adım 9 shallow geçmişi tespit edip (`git rev-parse --is-shallow-repository`)
   denetimi atlasın ve bunu açıkça söylesin — denetim yerelde kalır.
3. Tarih git geçmişinden değil, task dosyasının kendi frontmatter'ından
   okunsun — geçmişten bağımsız olur ama task formatına yeni bir zorunlu alan
   ekler.

## Karar (kullanıcı, 2026-09-07)

**Seçenek 3** — kapanış tarihi `done` task'ın kendi `closed` frontmatter
alanından okunur, git geçmişinden değil.

Gerekçe: geçmişten tümüyle bağımsız olan tek seçenek. `fetch-depth: 0` CI'ı
bugün yeşil tutar ama denetimin kendisi shallow bir depoda yanlış cevap
vermeye devam eder — mekanizma dokunulmadan durur, yalnız CI'ın bugünkü
geçmiş derinliği yeterli olduğu için görünmez kalır. Shallow'da atlamak ise
denetimi tam olarak koşması gereken yerde (CI, `fetch-depth: 1`) hiç
koşturmaz. Yalnız seçenek 3 DoD'un iki maddesini birlikte karşılıyor: shallow
depoda **doğru** cevap **ve** bayat STATUS'un shallow'da da yakalanması.

`scripts/check-docs.sh` adım 9 artık `fm_get "$f" closed` okuyor, `git log`
çağırmıyor — adım git'e hiç dokunmuyor. Yeni adım 3c, `state: done` olan her
task'ın `closed`'ı `YYYY-MM-DD` biçiminde doldurduğunu zorluyor. 64 done
task'a `closed:` backfill edildi; değer git'in bugüne kadar verdiği cevapla
birebir aynı — iki bağımsız sorgu (`git log -1` ve `git log --reverse | head
-1`, yani "kapanış commit'i" tanımının iki okuması) 64 dosyanın **hepsinde**
aynı tarihi verdi, göç yeni bir doğru/yanlış getirmedi, yalnız kaynağı
değiştirdi.

Kabul edilen bedel: `closed` kendi beyanıdır, git gibi otoriter değil. Onu
dürüst tutan `/finish-task`'ın aynı adımda yazması ve adım 3c'nin biçim
kapısı; git ile çapraz doğrulama eklenmedi çünkü bu shallow duyarlılığını
geri getirirdi.

## Kanıt (DoD)

- [x] Karar yazılı: hangi seçenek, neden — yukarıda "Karar (kullanıcı, 2026-09-07)"
- [x] `scripts/tests/` altında shallow bir depo kurup denetimin doğru cevabı
      verdiğini gösteren test — T14
- [x] Negatif: bayat bir STATUS tarihi shallow depoda da yakalanıyor —
      düzeltme denetimi sağırlaştırmıyor — T15
- [x] `bash scripts/test.sh` yeşil

## Kanıt kaydı

**`scripts/check-docs.sh` adım 9 artık git'e hiç dokunmuyor.** `fm_get "$f"
closed` ile her `done` task'ın kendi frontmatter'ından okuyor; eski `git log -1
--format=%cs -- "$f"` çağrısı ve `is-inside-work-tree` guard'ı kaldırıldı. Yeni
adım 3c, `state: done` olan her task'ın `closed`'ının dolu ve `YYYY-MM-DD`
biçiminde olduğunu zorluyor.

**64 done task'a `closed:` backfill edildi, elle değil ölçülerek.** Her dosya
için iki bağımsız git sorgusu karşılaştırıldı: `git log -1 --format=%cs -- <f>`
(denetimin bugüne kadarki cevabı) ve `git log --reverse --format=%cs -- <f> |
head -1` (dosyanın `done/` altına ilk taşındığı commit). **64 dosyanın hepsinde
birebir aynı tarih** (`differ: 0`) — backfill mevcut davranışı aynen taşıdı,
yeni bir doğru/yanlış üretmedi. En yeni kapanış `NEN-075` / `2026-09-07`.

**Yeni test paketi: `scripts/tests/check-docs.test.sh` T14–T17 (toplam 20
doğrulama, 0 kırmızı).**

- **T14 / T15 — gerçek CI senaryosu (adım 9).** Fixture iki commit'lik bir
  geçmiş kazandı: commit 1 (2026-09-05) fixture durumunu kurar, commit 2
  (2026-09-07) **yalnız doküman** ekler — `1ad194d`'nin (ADR-0041 kabulü)
  şekli, STATUS.md'ye ve `tasks/`'a dokunmuyor. `git clone --depth 1` ile
  üretilen tek-commit'lik kopya — CI'ın `fetch-depth: 1`'inin aynısı —
  üzerinde denetim koşuluyor. T14: güncel STATUS'ta shallow klon **geçiyor**
  (bugüne kadarki koddan farkı: eski adım 9 burada `NEN-901 (2026-09-07) daha
  yeni` derdi — commit 2'nin tarihini done task'ın kendi tarihi sanardı, tam
  olarak canlı CI kusurunun aynısı). T15 (negatif): shallow klonda STATUS
  tarihi bilinçli olarak bayatlatılınca (`2026-09-04`) denetim yine yakalıyor
  ve gerçek `NEN-901 (2026-09-05)` diyor — düzeltme denetimi sağırlaştırmadı.
- **T16 / T17 — adım 3c biçim kapısı (negatif).** `closed` alanı silinince
  ("boş") ve bozuk biçimde yazılınca (`2026-9-5`) ikisi de ayrı ayrı
  yakalanıyor.
- Mevcut T12/T13 (tam klonda güncel/bayat tarih) aynen geçmeye devam ediyor,
  artık git yerine frontmatter'dan.

**Üç ayrık negatif kontrol, izole ölçüldü** (her biri tek başına geri alınıp
hangi testin kırmızıya döndüğü kaydedildi):

1. Adım 9 eski git-tabanlı hâline döndürüldü → yalnız **T14 ve T15** kırmızı
   (T15'in mesajı da farklı bir yanlış tarihe düşüyor — `git`'in shallow'da
   verdiği tek cevap), tam-klon testleri (T1–T13) yeşil kaldı — düzeltme
   izole.
2. Adım 3c biçim kapısı kaldırıldı → yalnız **T16 ve T17** kırmızı.
3. Adım 9'un karşılaştırması sabit `elif false` yapıldı (hep "güncel" der) →
   **T13 ve T15** kırmızı — denetim gerçekten bir şey ölçüyor, sağır değil.

Her kontrolden sonra `scripts/check-docs.sh` orijinal hâline geri alındı ve
`diff` ile birebir aynı olduğu doğrulandı.

**Gerçek CI kusuru bağımsız olarak zaten doğrulanmıştı** (bu task açılırken):
`1ad194d` (yalnız ADR-0041 kabulü) GitHub Actions run **34086963805**'i
`failure` ile bitirdi; aynı ağaç yerelde (tam klon) `ok` veriyordu.
`NEN-075`'in kapanışı STATUS'u ileri taşıdığı için run **34088324092**
yeşile döndü, ama mekanizma bu task'a kadar düzeltilmeden durdu.

**Çalıştırılan komutlar ve sonuçları:**

```
$ bash scripts/tests/check-docs.test.sh
  … (20/20 doğrulama, 0 FAIL)

$ bash scripts/test.sh
  ▶ check-docs.test.sh
    ✓ check-docs.test.sh geçti
  ▶ doctor.test.sh
    ✓ doctor.test.sh geçti
  SONUÇ: 2 test dosyasının hepsi geçti.

$ bash scripts/check-docs.sh
  == 1 … 9 == hepsi ok (adım 8 yalnız bu task'ın kendi kapanışıyla düzelir)
```

**Kapsam dışı bırakılan:** `.github/workflows/ci.yml`'e dokunulmadı —
`fetch-depth: 1` kalıyor; kararın anlamı tam olarak denetimin artık geçmiş
derinliğini önemsememesi. Rust ve Swift'e dokunulmadı (yalnız shell + doküman
+ task dosyaları); o paketler bu task kapsamında koşturulmadı.
