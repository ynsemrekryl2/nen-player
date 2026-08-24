---
id: NEN-030
title: Milestone-aware doctor and STATUS consistency checks
milestone: M1
size: S
state: backlog
depends_on: []
blocks: []
adr: []
---

# NEN-030 — Milestone-aware doctor and STATUS consistency checks

## Sonuç

`scripts/doctor.sh` yalnız hedeflenen milestone'un araçlarına göre kapı görevi
görür — bugün M1'e başlamak isteyen biri, M3'ün Xcode/libmpv eksikleri yüzünden
bloke görünmez. `scripts/check-docs.sh` ise `docs/STATUS.md`'nin bayatlamasını
yakalar.

## Neden

İki tutarsızlık var. (1) Parametresiz `doctor.sh` gelecek milestone eksiklerini
mevcut blocker gibi sunup çıkış kodu 1 veriyor. (2) NEN-011 M1 içinde Kotlin/JVM
parity testi istiyor, ama doctor Java'yı yalnız M10 opsiyoneli gösteriyor.

## Kapsam

### doctor.sh

```bash
bash scripts/doctor.sh        # tüm durum, DAİMA çıkış 0 (bilgilendirici)
bash scripts/doctor.sh M1     # yalnız M1 kapısı — eksikse çıkış 1
bash scripts/doctor.sh M3
bash scripts/doctor.sh M10
```

Parametresiz çıktı her aracın hangi milestone'da gerektiğini gösterir ve sona
`M1'e başlamak için: bash scripts/doctor.sh M1` ipucunu basar.

Araç → milestone haritası (revize):

| Araç | Yeni konum | Not |
|---|---|---|
| `cargo`, `rustc` | M1 zorunlu | değişmedi |
| `cargo-deny` | M1 / **NEN-005 öncesi** | M1'e başlamanın blocker'ı değil |
| JDK | M1 / **NEN-011 öncesi** | Kotlin/JVM parity M1 içinde |
| Gradle | **hiçbir zaman blocker değil** | Wrapper tercih edilir |
| Tam Xcode | M3 | **M1 blocker'ı değil** |
| `swift` | M3 | değişmedi |
| libmpv | M3 — *development/binding prerequisite* | system `mpv` executable ürün prerequisite'i **değil**; linked/bundled strateji ADR-0012 |
| Android SDK / Studio | M10 | yeni satır |

### check-docs.sh — denetim 8

`docs/STATUS.md`'de yazan "sıradaki READY task", `task-index.sh`'in hesapladığı
listeyle uyuşmalı. Uyuşmuyorsa hata + ne yapılacağı.

### Shell testleri

`PATH`'in başına sahte `cargo`, `rustc`, `java`, `xcode-select`, `pkg-config`
shim'leri koyan geçici bir dizin kurulur. Böylece senaryolar gerçek makine
durumundan **bağımsız** ve tekrarlanabilir doğrulanır.

## YAPILMAYACAK

- Hiçbir şey kurmak — script yalnız raporlar
- Kullanıcı onayı olmadan `curl` / `brew` / `cargo install` çalıştırmak
- Mevcut `perl alarm` timeout'unu veya `pkg-config` → dylib arama davranışını
  kaldırmak (mpv CLI çalıştırılmaz — takılıyor)
- `task-index.sh`'in çalışan davranışını değiştirmek
- Sürüm eşiği/minimum sürüm kontrolü — araçlar kurulduktan sonra, gerçek
  sürümlerle ayrı bir task

## Kanıt (DoD)

- [ ] Shell testi: "hepsi var" senaryosu → `doctor.sh M1` çıkış 0
- [ ] Shell testi: "Rust yok" senaryosu → `doctor.sh M1` çıkış 1
- [ ] Shell testi: "yalnız Xcode/libmpv yok" → `doctor.sh M1` çıkış **0**,
      `doctor.sh M3` çıkış **1**
- [ ] Parametresiz çağrı her senaryoda çıkış **0** veriyor
- [ ] Negatif: bilinmeyen milestone argümanı (`doctor.sh M99`) anlaşılır hata + çıkış 1
- [ ] `check-docs.sh` denetim 8: STATUS'taki READY task kasten bozulduğunda
      hata veriyor, düzeltilince geçiyor
- [ ] Hiçbir testte kurulum komutu **çalıştırılmıyor** (yalnız raporlanıyor)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
