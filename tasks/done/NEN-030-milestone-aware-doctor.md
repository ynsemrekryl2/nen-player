---
id: NEN-030
title: Milestone-aware doctor and STATUS consistency checks
milestone: M1
size: S
state: done
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

- [x] Shell testi: "hepsi var" senaryosu → `doctor.sh M1` çıkış 0
- [x] Shell testi: "Rust yok" senaryosu → `doctor.sh M1` çıkış 1
- [x] Shell testi: "yalnız Xcode/libmpv yok" → `doctor.sh M1` çıkış **0**,
      `doctor.sh M3` çıkış **1**
- [x] Parametresiz çağrı her senaryoda çıkış **0** veriyor
- [x] Negatif: bilinmeyen milestone argümanı (`doctor.sh M99`) anlaşılır hata + çıkış 1
- [x] `check-docs.sh` denetim 8: STATUS'taki READY task kasten bozulduğunda
      hata veriyor, düzeltilince geçiyor
- [x] Hiçbir testte kurulum komutu **çalıştırılmıyor** (yalnız raporlanıyor)

## Kanıt kaydı

Tarih: 2026-08-24 · Kanıt tipi: tooling shell testi (31 doğrulama) + gerçek çıktı

### Testler

```
$ bash scripts/test.sh
▶ check-docs.test.sh   (7 doğrulama)  ✓
▶ doctor.test.sh       (24 doğrulama) ✓
SONUÇ: 2 test dosyasının hepsi geçti.          → exit 0
```

`doctor.test.sh` senaryo matrisi — hepsi PATH shim'leri ile, gerçek makine
durumundan bağımsız (`PATH="$BIN:/usr/bin:/bin"`, `HOME` geçici dizin,
`NEN_DOCTOR_MPV_PATHS` ve `ANDROID_HOME` kontrollü):

| Senaryo | `doctor.sh` | `M1` | `M3` | `M10` |
|---|---|---|---|---|
| S1 hepsi var | 0 | 0 | 0 | 0 |
| S2 Rust yok | 0 | **1** | **1** | — |
| S3 yalnız Xcode/libmpv yok | 0 | **0** | **1** | — |
| S4 JDK yok (Rust var) | 0 | **0** (soon) | — | **1** |
| S5 `M99` / fazla argüman | — | — | — | **1** |
| S6 `m1` (küçük harf) | — | 0 | — | — |

İçerik doğrulamaları: S3'te `M1` çıktısında **BLOCKER bölümü yok** ve Xcode/libmpv
"gerekmeyenler" altında · S3 `M3` "TAM XCODE DEĞİL" diyor · S4 `M1` "YAKINDA
GEREKLİ" bölümü basıyor · S5 "bilinmeyen milestone" mesajı veriyor.

`check-docs.test.sh` (repo'nun geçici kopyasında, gerçek dosyalara dokunmadan):
uyumlu liste → geçer · uydurma task (`NEN-999`) → hata · boş liste → hata ·
READY satırı silinmiş → hata · `STATUS.md` yok → hata · düzeltilince tekrar geçer ·
**T7: gerçek repo dosyaları değişmemiş** (`shasum` parmak izi, test öncesi/sonrası).

### Negatif: hiçbir kurulum komutu çalıştırılmadı

`brew`, `curl`, `rustup`, `sdkmanager` ve `cargo install` için sentinel yazan
tuzak shim'leri kuruldu. 24 doctor çalıştırmasının sonunda sentinel dosyası
**oluşmadı** — S7 bunu doğruluyor. Kurulum komutları yalnız metin olarak
raporlanıyor.

### Gerçek makinede davranış (2026-08-24)

```
$ bash scripts/doctor.sh ; echo $?
SONUÇ: 8 araç kurulu değil (bilgilendirici — çıkış kodu 0).
0                          ← ÖNCEDEN 1'di; asıl düzeltme bu

$ bash scripts/doctor.sh M1 ; echo $?
BLOCKER — M1 başlayamaz:      cargo, rustc
YAKINDA GEREKLİ:              cargo-deny (NEN-005 öncesi), JDK (NEN-011 öncesi)
M1 için gerekmeyenler:        Gradle Xcode swift libmpv Android SDK
SONUÇ: M1 için 2 blocker eksik.
1

$ bash scripts/doctor.sh M3 ; echo $?
BLOCKER — M3 başlayamaz:      cargo, rustc, Xcode (yalnız CommandLineTools), libmpv
HAZIR:                        swift (Apple Swift version 6.4)
1

$ bash scripts/doctor.sh M99 ; echo $?
HATA: bilinmeyen milestone 'M99'.
1
```

### check-docs.sh denetim 8

Denetim eklendikten sonraki **ilk çalıştırmada gerçek bir kayma yakaladı**:
NEN-030 `active`'e alınınca INDEX'in ready listesi `NEN-007`'ye düştü ama
`STATUS.md` hâlâ `NEN-030`'u listeliyordu.

```
HATA  docs/STATUS.md 'Sıradaki READY' satırı INDEX ile uyuşmuyor.
      INDEX  : NEN-007
      STATUS : NEN-007 NEN-030
```

### Uygulama notları

- `detect_xcode` artık **rc=2** sözleşmesi kullanıyor ("var ama yanlış tür").
  İlk denemede `DETECT_DETAIL_EXTRA` global'i komut ikamesi subshell'inde
  kayboluyordu ve CommandLineTools ayrımı çıktıya hiç yansımıyordu.
- Durum işaretlerinin hepsi 3 baytlık UTF-8 (`✓ ✗ ⚠ ○`) — `printf %-3s` bayt
  saydığı için ASCII `!` kullanınca sütunlar kayıyordu.
- **Test kendi kusurunu yakaladı.** T7 ilk halinde gerçek repodaki
  `git status --porcelain docs/STATUS.md` çıktısına bakıyordu; bu task
  `STATUS.md`'yi meşru şekilde değiştirince test kırıldı. Ölçmek istediği şey
  "test dosyaya dokundu mu", git durumu değil — `shasum` parmak izine çevrildi.
- `NEN_DOCTOR_MPV_PATHS` env seam'i eklendi: PATH shim'i `pkg-config`'i taklit
  edebiliyor ama `/opt/homebrew/lib/libmpv.dylib` dosya varlığını edemiyor.
  Bu olmadan "libmpv yok" senaryosu, libmpv kurulu bir makinede yanlış sonuç
  verirdi.
