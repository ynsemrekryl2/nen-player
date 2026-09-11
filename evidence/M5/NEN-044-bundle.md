# NEN-044 — Bundle kapanışı yeniden ölçümü (ADR-0045 Karar 5)

Tarih: 2026-09-11. Makine: Apple Silicon, Homebrew `/opt/homebrew`.

## Komut

```bash
bash scripts/build-macos-app.sh
bash scripts/bundle-macos.sh
```

## Sonuç

```
▶ 6/6  bundle taranıyor: kalan Homebrew referansı var mı?
  temiz — hiçbir Mach-O /opt/homebrew veya /usr/local'a işaret etmiyor

Gömülü dylib sayısı: 48
```

**48 — `NEN-043`'ün orijinal kapanışıyla birebir aynı** (`evidence/M3/NEN-043-checklist.md`).
ADR-0045 Karar 5'in beklentisi doğrulandı: `libavformat`/`libavcodec`/`libavutil`
zaten `libmpv`'nin geçişli kapanışındaydı, doğrudan linkleme yeni bir dylib
ailesi eklemedi.

## Doğrudan linkleme kanıtı

`otool -L` ana ikili dosyanın artık üç kütüphaneyi de **doğrudan** (yalnız
`libmpv` üzerinden transitif değil) bağladığını gösteriyor:

```
$ otool -L .build/NenPlayer.app/Contents/MacOS/NenPlayer | grep -Ei 'avformat|avcodec|avutil|mpv'
	@rpath/libavformat.63.dylib (compatibility version 63.0.0, current version 63.1.101)
	@rpath/libavcodec.63.dylib (compatibility version 63.0.0, current version 63.1.101)
	@rpath/libavutil.61.dylib (compatibility version 61.0.0, current version 61.1.101)
	@rpath/libmpv.2.dylib (compatibility version 2.0.0, current version 2.0.0)
```

`Contents/Frameworks/` dizinindeki dylib sayısı da bağımsız olarak `ls | wc -l`
ile **48** doğrulandı — script'in kendi raporuyla aynı.

## Codesign / Homebrew taraması

`bundle-macos.sh`'in kendi bağımsız taraması (adım 6) her Mach-O dosyasını
gezip `/opt/homebrew` veya `/usr/local`'a işaret eden tek bir yol kalmadığını
doğruladı — `libavformat`/`libavcodec`/`libavutil` dahil, hepsi
`@rpath`/`@loader_path`'e çevrilmiş. `codesign --verify --deep --strict` ve
`valid on disk` / `satisfies its Designated Requirement` geçti.
