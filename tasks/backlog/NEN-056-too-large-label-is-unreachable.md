---
id: NEN-056
title: Resolve the unreachable "çok büyük" reason label
milestone: M3
size: S
state: backlog
depends_on: [NEN-025]
blocks: [NEN-026]
adr: [31]
---

# NEN-056 — Resolve the unreachable "çok büyük" reason label

## Sonuç

ADR-0031 Karar 5'in kendi içindeki çelişki giderilir; `NEN-026` üretilemeyecek
bir etiketi uygulamaya çalışmaz.

## Bağlam

`NEN-025` uygulanırken bulundu. ADR-0031 Karar 5 iki madde içeriyor ve ikisi
aynı anda doğru olamaz:

- Birinci madde, **kataloğa giren ama kullanılamayan** kaynağın taşıyacağı
  kapalı etiket kümesini sayıyor: `okunamadı` · `biçim hatalı` · **`çok büyük`**.
- İkinci madde, boyut sınırını **güvenlik kapısına** koyuyor: kapıdan dönen
  dosya kataloğa **hiç girmiyor**.

Boyut aşımı kataloğa hiç girmiyorsa, katalogdaki bir kaynağın `çok büyük`
etiketini taşıması mümkün değildir. `NEN-026` bu etiketi listeliyor, yani
üretilemeyecek bir durum için UI yazmak üzere.

`NEN-025` bu çelişkiyi **kendi DoD'una göre** çözdü — orası net: boyut → kapı →
katalogda yok, ve `an_oversized_file_is_refused_without_being_opened` bunu
kanıtlıyor. Yani bugünkü kod tutarlı; tutarsız olan doküman.

## Kapsam

- ADR-0031 Karar 5'in hangi yarısının doğru olduğuna karar ver
- Karar "boyut kapıdır" ise (bugünkü kod): Karar 5'in etiket kümesinden
  `çok büyük` çıkarılır, `NEN-026`'nın kapsamı buna göre düzeltilir
- Karar "boyut kataloğa girer" ise: `security-policy.md` §4 #4'ün "okunmaz"
  ifadesiyle çakışır, o yüzden **yeni bir ADR** gerekir ve `NEN-025`'in kapıları
  değişir
- ADR düzenlenmez — karar değişiyorsa `superseded` yapılıp yenisi yazılır
  (ADR-0001)

## YAPILMAYACAK

- Menü UI'ı → `NEN-026`
- `NEN-025`'in kapılarını bu task içinde değiştirmek — önce karar

## Kanıt (DoD)

- [ ] ADR-0031 Karar 5 kendi içinde çelişmiyor (tutarlılık okuması)
- [ ] `NEN-026`'nın etiket kümesi, üretilebilen durumlarla birebir eşleşiyor
- [ ] Seçilen kapalı kümenin her elemanı için onu üreten bir test var

## Kanıt kaydı

<!-- done olurken doldurulacak -->
