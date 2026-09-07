# NEN-078 — Stremio'nun harici oynatıcıyı nasıl çağırdığı ölçüldü

**Tarih:** 2026-09-07 · **Makine:** Apple M5 / macOS 27 (Darwin 27.0.0)
**Stremio:** Uygulama sürümü `6.0.1-beta` (Ayarlar ekranında "Uygulama Sürümü"),
**kabuk (shell) sürümü `5.1.26`** (`/Applications/Stremio.app`,
`CFBundleIdentifier=com.westbridge.stremio5-mac`; kabuk sürümü = `Info.plist`
`CFBundleShortVersionString`). K23 nedeniyle bu raporda hiçbir gerçek medya
URL'si, token, port numarası, host adı (genel add-on adları hariç) veya özel
dosya yolu yer almıyor — yalnız **şekil** kaydedilir.

## Soru

Stremio macOS'ta harici oynatıcıyı hangi mekanizmayla çağırıyor, medyayı hangi
biçimde veriyor, başlangıç pozisyonunu taşıyor mu, yanında metadata geliyor mu.

## Yöntem

### 1. Statik inceleme (canlı ölçümden önce)

- `/Applications/Stremio.app/Contents/Info.plist`: `CFBundleURLTypes` yalnız
  `stremio` scheme'ini taşıyor; **`CFBundleDocumentTypes` yok** — Stremio'nun
  kendisi open-with/doküman tipi almıyor.
- Shell binary'sinin webview'a enjekte ettiği JS (`strings` ile okundu):
  `window.playExternal = url => sendShellRPC('play-external', String(url))`,
  tıklanan `<a href>`'in **ham** `getAttribute('href')`'i okunuyor (kod
  yorumu: `link.href` normalize edip `scheme://https://...` şeklini bozduğu
  için). Bu, embedded webview'daki **harici link tıklaması** yolu — aşağıdaki
  gerçek "Harici oynatıcıda oynat" akışından **ayrı** bir yol olduğu ölçümle
  doğrulandı (bkz. Bulgu 3).
- Binary `/usr/bin/open` string'ini taşıyor.
- Bu makinede `mpv:` `iina:` `outplayer:` `infuse:` `vlc:` `potplayer:`
  `nplayer:` scheme'lerinin **hiçbiri**, prob kaydından önceki taramada,
  `lsregister -dump`'ta kayıtlı bir handler'a sahip değildi. Bu, canlı ölçümde
  görülen gerçek mekanizmanın (Bulgu 4) scheme claim'iyle **hiç** ilgisi
  olmadığını doğruluyor — `mpv:`'nin daha önce kimseye ait olmaması, "MPV
  içinde oynat"ın sonunda gerçek bir uygulamaya ulaşmasını (Bulgu 6)
  engellemedi, çünkü o yol scheme resolution'dan hiç geçmiyor.

### 2. Probe (geçici, depoya girmedi)

`NEN-025` emsali izlendi: `$SCRATCHPAD/HandoffProbe.app` adında, ad-hoc
imzalı, tek kullanımlık bir Swift/Cocoa uygulaması yazıldı. Üç giriş noktasını
(`application(_:open:)`, `application(_:openFile(s):)`, `CommandLine.arguments`)
kaydedip **yalnız şekli** `/tmp` altında geçici bir log'a yazıyordu: giriş
noktası · scheme · host sınıfı (`loopback`/`remote-ip`/`remote-host`) · path
segment sayısı · son segmentin uzantısı · query anahtar **adları** · toplam
uzunluk. `Info.plist`'i `mpv/iina/outplayer/infuse/vlc/potplayer/nplayer`
scheme'lerini ve `public.movie`/`public.m3u-playlist`/`public.url` doküman
tiplerini claim ediyordu. `codesign --sign -` ile ad-hoc imzalandı,
`lsregister -f` ile kaydedildi.

**Sağırlık kontrolü** (Stremio'ya dokunmadan): `open "mpv://..."` argv yolu,
`open -a <probe> <yerel dosya>` doküman yolu ve doğrudan `exec` + argv yolu
ayrı ayrı denendi. Doküman yolu ve argv yolu **log ürettti** (prob çalışıyor).
URL-scheme yolu `open`/`NSWorkspace` üzerinden **hiç çözülmedi**
(`kLSApplicationNotFoundErr`) — bu makinede `security find-identity -v -p
codesigning` **0 kimlik** veriyor (NEN-043'ün de kaydettiği durum) ve
`spctl -a -vv` prob'u `rejected` buluyor; ad-hoc imzalı, gerçek geliştirici
kimliği olmayan bir uygulama bu makinede **hiçbir** uygulamanın çağırabileceği
bir URL-scheme handler'ı olamıyor (kontrol grubu: `claude://`, `bitwarden://`,
`chatgpt://` gibi `/Applications`'ta düzgün imzalı kurulu uygulamaların
scheme'leri normal çözülüyor). Bu, sağırlık kontrolünün **kendisinin** ürettiği
bir bulgu — aşağıya taşındı (Bulgu 5).

### 3. Canlı ölçüm

Kullanıcının hesabındaki mevcut "İzlemeye devam edin" öğeleri kullanıldı.
Ayarlar → Oynatıcı → Gelişmiş → **"Harici oynatıcıda oynat"** bulundu ve
sırasıyla `MPV`, `IINA` seçilerek video oynatıcısının `...` menüsündeki
"VLC içinde oynat" / "MPV içinde oynat" eylemleri tetiklendi. Her tetiklemeden
hemen önce/sonra `ps aux` ile ortaya çıkan process'in **yalnız bayrak
adları** (`--start=…`, `--no-terminal`, `--start-time=…`,
`--no-video-title-show`) okundu; URL argümanı **hiçbir zaman** dosyaya
yazılmadı, yalnız bu oturumun geçici komut çıktısında görüldü ve şekli
aşağıda anlatılıyor. Ölçüm sonunda ayar `Etkisizleştirildi`'ye geri alındı ve
seek edilen konum yaklaşık özgün konumuna geri getirildi.

### 4. Temizlik

`lsregister -u` ile prob'un claim'leri geri alındı, `~/Applications`'a alınan
geçici kopya silindi (dizinin kendisi kullanıcının mevcut uygulamalarını
taşıdığı için dokunulmadı), `lsregister -dump` prob referansı **0** ile
doğrulandı, geçici log ve test dosyaları silindi.

## Bulgular

**1. Ayar var, ama serbest metin/özel komut alanı yok.** "Harici oynatıcıda
oynat" (Ayarlar → Oynatıcı → Gelişmiş) beş seçenekli **kapalı** bir liste:
`Etkisizleştirildi · MPV · IINA · Infuse · M3U Playlist`. Nen Player bu
listede **yer alamaz** — Stremio'nun kendi ayar yüzeyinden çağrılabilmek bu
sabit kümeye girmeyi gerektirir ve bu, Nen Player'ın kontrolünde değil.

**2. Bu ayarı değiştirmek, oynatmanın kendisini değiştirmiyor.** Bir medya
açıldığında ayar ne olursa olsun oynatma **Stremio'nun kendi gömülü
oynatıcısında** başlıyor (embedded libmpv). Harici oynatıcı yalnız oynatıcı
ekranındaki `...` (diğer seçenekler) menüsünden **elle** tetikleniyor.

**3. `...` menüsü ayardan bağımsız, sabit iki seçenek taşıyor: "VLC içinde
oynat" ve "MPV içinde oynat".** Ayar `MPV` iken de `IINA` iken de bu menüde
aynı iki seçenek görüldü — IINA/Infuse için ayrı bir menü girdisi hiç
çıkmadı. Yani Ayarlar'daki beşli liste ile oynatıcı ekranındaki bu ikili menü
**bağımsız** iki mekanizma; bu ölçümün kapsamında IINA/Infuse/M3U Playlist
seçeneklerinin gerçekte neyi tetiklediği **gözlenemedi** (bu makinede IINA ve
Infuse kurulu değil) — açık soru olarak aşağıya taşındı.

**4. Mekanizma: custom URL scheme değil, doğrudan positional argv exec.**
Prob'un kayıtlı olduğu `mpv:`/`iina:`/`vlc:` scheme'lerinin **hiçbirine**
tetikleme düşmedi (prob log'u boş kaldı) — Stremio `open`/LaunchServices
üzerinden bir URL scheme açmıyor. Bunun yerine, hedef oynatıcıyı **doğrudan,
kendi CLI biçimiyle** başlatıyor:
  - "MPV içinde oynat" → gerçek `/Applications/VLC.app` yerine, bu makinede
    **önceden kurulu** ayrı bir uygulama `--start=<saniye> --no-terminal
    <url>` argv'ı ile başladı (mpv'nin kendi CLI biçimi — bkz. Bulgu 6, bu
    uygulamanın kimliği).
  - "VLC içinde oynat" → gerçek `/Applications/VLC.app`
    `--start-time=<saniye> --no-video-title-show <url>` argv'ı ile başladı
    (VLC'nin kendi CLI biçimi — farklı bayrak adı, `ps aux` ile ayrı ayrı
    doğrulandı).
  Bu, Stremio'nun her hedef için **kendi bildiği CLI sözleşmesini** taklit
  eden, sabit-kodlanmış bir eşleme kullandığını gösteriyor — genel bir "her
  scheme'i/uygulamayı kabul eden" yüzey değil.

**5. Pozisyon taşınmıyor — iki bağımsız denemede de `--start(-time)=0`.**
Aynı, oynatılmakta olan medya üzerinde iki ayrı anda ("00:00:48" ve
"00:15:29" gerçek oynatma konumlarında) harici oynatıcı tetiklendi; her
ikisinde de argv'daki başlangıç bayrağı **0** geldi — gömülü oynatıcının o
anki konumu hiç iletilmedi. Bu, M4'ün "medya doğru pozisyondan oynuyor" çıkış
kriterini doğrudan etkiliyor: bu mekanizma (harici oynatıcı `...` menüsü) üzerinden
pozisyon **kaynakta hiç yok**, alıcı tarafta işlenecek bir şey yok.

**6. "MPV içinde oynat"ın gerçek hedefi, önceden kurulu bir Stremio yardımcı
uygulaması — genel bir mpv binary'si değil.** Bu makinede gerçek bir "MPV.app"
kurulu değil; hedef `Info.plist`'i okunduğunda `CFBundleIdentifier` **`local.stremio.*`**
öneki taşıyan, ayrı ve önceden kurulu bir uygulama çıktı. Bu uygulamanın
`lsregister -dump`'taki kendi scheme claim'i `mpv:`/`vlc:`/`iina:` **değil**,
kendine özgü, Stremio-namespaced bir scheme — yani Stremio'nun ona ulaşması da
Bulgu 4'ün gösterdiği gibi scheme resolution'dan **geçmiyor**. Bu, iki şeyi
birden doğruluyor: (a) "MPV içinde oynat" etiketi bir CLI-sözleşmesi seçimi,
gerçek bir uygulama kimliği değil — bu makinede o sözleşmeyi karşılayan **her
uygulama** hedef olabilir; (b) Stremio'nun resmi olarak tanıdığı/desteklediği,
kendi ekosistemine ait yardımcı uygulamalar var olabilir ve bunlar genel
LaunchServices aramasının dışında, Stremio'ya özgü bir mekanizmayla
bulunuyor olabilir — bu mekanizmanın kendisi bu ölçümün kapsamı dışında kaldı.

**7. Metadata: ayrı bir alan yok, yalnız URL'nin kendi dosya adı segmentinde
zımni olarak var.** Argv'da başlık/sezon/bölüm/canonical ID için ayrı bir
bayrak veya ortam değişkeni **gözlenmedi**. URL'nin son path segmenti orijinal
sürüm adını (release name) taşıyordu — yıl, çözünürlük, dil etiketi gibi bilgi
oradan **çıkarılabilir** ama yapılandırılmış bir alan olarak gelmiyor.

**8. Medya URL'si her zaman Stremio'nun kendi loopback sunucusundan
gelmeyebilir.** Ana ekranda `http://localhost:<port>` üzerinden bir "streaming
server" bildirimi görüldü (Stremio'nun kendi yerel proxy'si), ama harici
oynatıcıya giden URL bu ölçümde **uzak, https bir add-on host'undan** geldi
(genel bir torrent-çözücü add-on servisi — host adı kendisi gizli değil, ama
tam URL/token K23 gereği kaydedilmedi). Yani alıcı taraf hem `file://`/loopback
hem uzak `https` biçimini karşılamaya hazır olmalı; bu ölçüm ikisini de
**gözlemleyemedi** aynı oturumda (yalnız uzak https görüldü) — kapsam sınırı
olarak not edildi.

**9. Ad-hoc/notarize edilmemiş bir alıcı, bu makinede LaunchServices
tarafından hiçbir uygulamaya scheme handler olarak sunulamıyor.** Sağırlık
kontrolünün kendisi bunu ortaya çıkardı (Yöntem §2): `security find-identity`
0 kimlik veriyor, prob `spctl -a -vv` altında `rejected`. Bulgu 4'ün zaten
gösterdiği gibi Stremio bu yolu (custom scheme) kullanmıyor, dolayısıyla bu
task'ın ölçümünü **engellemedi** — ama Nen Player kendi `Info.plist`'ine bir
`CFBundleURLTypes` eklese bile, gerçek Developer ID imzası olmadan (NEN-043'ün
kaydettiği, YAPILMAYACAK kapsamındaki kısıt) bu makinede o scheme'in **hiçbir
göndericiye** görünmeyeceği ayrıca doğrulandı. Argv/doküman yollarında bu kısıt
gözlenmedi (prob ad-hoc haliyle ikisinde de çalıştı).

## `NEN-079`'a taşınan

- **Argv/doküman tabanlı alıcı, custom URL scheme'den daha güvenilir
  görünüyor** — Stremio'nun kendisi zaten scheme kullanmıyor (Bulgu 4) ve
  scheme yolu bu makinede ad-hoc imza için ayrıca kısıtlı (Bulgu 9, geçici;
  gerçek Developer ID alınırsa kalkar).
  Ama Stremio'nun sabit-kodlanmış olan şey **isim/uygulama kimliği değil,
  CLI-sözleşmesi** (Bulgu 4+6) — bu makinede "MPV" hedefi gerçekte mpv
  olmayan, farklı bir bundle kimliğine sahip bir uygulamaydı ve yalnız
  mpv'nin argv biçimini karşıladığı için orada durdu. Bu, Nen Player'ın bu iki
  sabit yuvadan birine **taklit yoluyla** girmesi gerektiği anlamına gelmez —
  Ayarlar listesindeki `MPV`/`IINA`/`Infuse` seçenekleri de aynı ölçümde
  davranışsal bir fark üretmedi (Bulgu 3); bu yüzden bu ölçüm **hangi yüzeyin
  kurulacağına dair kesin bir öneri üretmiyor**, yalnız adayları (custom
  scheme vs. bilinen bir CLI sözleşmesini taklit etmek vs. Bulgu 6'nın işaret
  ettiği, Stremio'ya özgü olası bir üçüncü keşif yolu) ölçülmüş verilerle
  karşılaştırıyor.
- **Pozisyon** (Bulgu 5): bu mekanizmadan hiç gelmiyor. `NEN-081`'in kapsamı
  ("handoff'un taşıdığı başlangıç pozisyonunu uygula") bu yol için **boş
  küme** olabilir — M4'ün "doğru pozisyondan oynuyor" kriteri, pozisyonu
  handoff'tan değil (yoksa) başka bir kaynaktan (örn. Nen Player'ın kendi
  içerik-kimliği eşleşmesi, kapsam dışı) karşılamayı gerektirebilir; bu karar
  `NEN-079`'a bırakılıyor.
- **Metadata** (Bulgu 7): yapılandırılmış alan yok; yalnız dosya adının
  kendisi zayıf bir ipucu. `NEN-082`'nin "opsiyonel kanıt" çerçevesi bunu
  zaten öngörüyor — güçlendiriyor.
- **Kapsam sınırı, ölçülemedi:** IINA/Infuse/M3U Playlist seçeneklerinin
  gerçek davranışı (Bulgu 3), Stremio'nun yerel yardımcı uygulamaları nasıl
  keşfettiği (Bulgu 6) ve loopback-biçimli bir medyanın handoff şekli
  (Bulgu 8) bu makinede tam gözlenemedi; `NEN-079` bu noktalarda spekülatif
  kalıyor.

## K23 uyumluluk notu

Bu raporda hiçbir gerçek medya URL'si, token, port numarası veya özel dosya
yolu yer almıyor. `ps aux` çıktısında görülen tam argv (URL dahil) hiçbir
dosyaya yazılmadı; yalnız bayrak adları (`--start=`, `--no-terminal`,
`--start-time=`, `--no-video-title-show`) ve yukarıdaki **kategorik** şekil
tanımları kaydedildi.
