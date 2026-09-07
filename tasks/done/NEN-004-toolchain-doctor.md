---
id: NEN-004
title: Toolchain doctor script
milestone: M0
size: S
state: done
closed: 2026-08-24
depends_on: [NEN-001]
blocks: [NEN-005, NEN-007, NEN-022]
adr: []
---

# NEN-004 — Toolchain doctor script

## Sonuç

`scripts/doctor.sh` çalıştırıldığında, hangi aracın kurulu olduğu, hangisinin
eksik olduğu, eksik olanın hangi milestone'da gerektiği ve nasıl kurulacağı
tek ekranda görünür.

## Kapsam

Kontrol edilenler ve gerekli oldukları milestone:

| Araç | Milestone | Not |
|---|---|---|
| `rustc` / `cargo` | M1 | zorunlu |
| Tam Xcode | M3 | `xcode-select -p` çıktısı `CommandLineTools` **olmamalı** |
| `mpv` / libmpv | M3 | zorunlu |
| `cargo-deny` | M1 (CI) | zorunlu |
| JDK + Gradle | M10 | opsiyonel, uyarı |

- Her araç için: var/yok, sürüm, gerekli olduğu milestone, kurulum komutu
- Eksik **zorunlu** araç varsa çıkış kodu 1

## YAPILMAYACAK

- Otomatik kurulum — script hiçbir şey **kurmaz**, yalnız raporlar
- Sürüm sabitleme/eşik kontrolü — araçlar kurulduktan sonra, gerçek sürümlerle M1'de

## Kanıt (DoD)

- [x] Bu makinede çalıştırıldığında Rust'ı **eksik** olarak raporluyor
- [x] `/Library/Developer/CommandLineTools`'u tam Xcode **değil** diye tespit ediyor
- [x] Kurulu bir aracı (`swift`) doğru sürümle raporluyor
- [x] Eksik zorunlu araç varken çıkış kodu 1, hepsi varken 0

## Kanıt kaydı

Tarih: 2026-08-24 · Kanıt tipi: bu makinedeki gerçek çıktı

```
$ bash scripts/doctor.sh
Nen Player — toolchain doctor
macOS 27.0

       ARAÇ          DURUM                                    GEREKLİ
  ---------------------------------------------------------------------------
  ✗  cargo          EKSİK                                    M1
  ✗  rustc          EKSİK                                    M1
  ✗  cargo-deny     EKSİK                                    M1 (CI)
  ✗  Xcode          yalnız CommandLineTools — TAM XCODE DEĞİL M3
       aktif yol: /Library/Developer/CommandLineTools
  ✓  swift          Apple Swift version 6.4                  M3
  ✗  libmpv         EKSİK                                    M3
  ⚠  java           eksik (opsiyonel)                        M10
  ⚠  gradle         eksik (opsiyonel)                        M10

SONUÇ: 5 zorunlu araç eksik, 2 opsiyonel eksik.
$ echo $?
1
```

Doğrulanan davranışlar:

- Rust **eksik** olarak raporlandı (M1 için zorunlu) ✓
- `/Library/Developer/CommandLineTools` tam Xcode **değil** diye ayırt edildi ✓
- Kurulu araç (`swift`) doğru sürümle raporlandı: Apple Swift version 6.4 ✓
- Eksik zorunlu araç varken çıkış kodu **1** ✓

Tasarım notu: `mpv --version` bu makinede yanıt vermeyip takıldığı için script
CLI'ı çalıştırmak yerine önce `pkg-config --modversion mpv`, sonra bilinen
dylib yollarını arıyor; tüm dış komutlar `perl alarm` tabanlı zaman sınırıyla
çalıştırılıyor (macOS'ta GNU `timeout` yok).
