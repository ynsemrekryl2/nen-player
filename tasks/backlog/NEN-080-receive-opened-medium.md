---
id: NEN-080
title: Receive a medium opened by another application
milestone: M4
size: M
state: backlog
closed:
depends_on: [NEN-079]
blocks: [NEN-081, NEN-082, NEN-083]
adr: [43]
---

# NEN-080 — Receive a medium opened by another application

## Sonuç

Başka bir uygulamanın Nen Player'a verdiği `file`/`http`/`https` medyası
açılıyor ve `⌘O` ile açılmış gibi aynı yoldan oynuyor — uygulama kapalıyken de,
açıkken de.

## Bağlam

Bugün alıcı yüzey **hiç yok**: `platforms/macos/Resources/Info.plist` ne
`CFBundleDocumentTypes` ne `CFBundleURLTypes` taşıyor,
`platforms/macos/Sources/NenPlayerApp/NenPlayerApp.swift`'in `AppDelegate`'i
yalnız `applicationShouldTerminateAfterLastWindowClosed` ve
`applicationShouldHandleReopen` uyguluyor, `CommandLine` hiç okunmuyor.

Yüzeyin **hangisi** olduğu `NEN-079`'un ADR'sinde kararlaştırılır; bu task onu
uygular. Medya bir kez çözüldükten sonra yeni bir yol açılmaz: mevcut
`PlaybackSession::load(locator)` (`core/crates/nen-app/src/session.rs:150`)
çağrılır, uzak URL'ler `remote_evidence::validate_url`'ün şema kapısından
geçer, açılan medya `RecentMediaStore`'a normal kaydıyla girer.

## Kapsam

- ADR'nin seçtiği alıcı yüzey(ler): `Info.plist` tipleri ve/veya
  `application(_:open:)` ve/veya argv okuması
- Gelen locator'ın ayrıştırılması ve tipli hataya bağlanması — `unwrap`/`expect`
  yok (`security-policy.md` §2)
- Reddedilenler: desteklenmeyen şema, boş/bozuk argüman, medya olmayan girdi —
  hepsi sessizce değil, kullanıcının gördüğü mevcut hata yüzeyiyle
- Uygulama **kapalıyken** açılış ve **zaten açıkken** ikinci medya, aynı davranış
- Açılan medyanın son açılanlar listesine normal kaydı

## YAPILMAYACAK

- Başlangıç pozisyonu → `NEN-081`
- Handoff metadata'sının kimliğe bağlanması → `NEN-082`
- Log denetimi → `NEN-083` (K23 kapalı-küme testi orada)
- Yeni bir medya açma yolu kurmak — mevcut `load` yolu kullanılır
- Yeni HTTP politikası — şema kapısı `validate_url`'de, genişletilmez
- Android tarafı → M10

## Kanıt (DoD)

- [ ] Platform testi: verilen `file://` locator'ı medyayı yüklüyor
- [ ] Platform testi: `http`/`https` locator'ı aynı yoldan geçiyor
- [ ] Negatif: desteklenmeyen şema (ör. `ftp://`, `javascript:`) ve bozuk
      argüman tipli hatayla reddediliyor, panik yok
- [ ] Uygulama kapalıyken açılan medya ve açıkken açılan ikinci medya için
      ayrı testler
- [ ] Gerçek `.app` kabulü: `open -a` ile verilen fixture oynuyor, medya adı
      kromda doğru, son açılanlar listesine giriyor
- [ ] Regresyon: `⌘O` yolu ve `bash scripts/test-macos.sh` yeşil

## Kanıt kaydı

<!-- done olurken doldurulacak -->
