---
id: NEN-076
title: The STATUS freshness check reads every done task as today on CI
milestone: M3
size: S
state: backlog
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

## Kanıt (DoD)

- [ ] Karar yazılı: hangi seçenek, neden
- [ ] `scripts/tests/` altında shallow bir depo kurup denetimin doğru cevabı
      verdiğini gösteren test
- [ ] Negatif: bayat bir STATUS tarihi shallow depoda da yakalanıyor —
      düzeltme denetimi sağırlaştırmıyor
- [ ] `bash scripts/test.sh` yeşil

## Kanıt kaydı

<!-- done olurken doldurulacak -->
