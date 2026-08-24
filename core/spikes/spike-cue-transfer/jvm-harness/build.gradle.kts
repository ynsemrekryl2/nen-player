// NEN-011 JVM ölçüm harness'ı — ÜRÜN KODU DEĞİLDİR (CLAUDE.md kural 7).
//
// Ürünün Kotlin/Gradle modülü yok (platforms/android hâlâ boş iskelet) ve bu
// paketi tanımaz; ADR-0028'in birinci sınırı: spike binding'i ürün paketine
// linklenmez.
//
// `generated/` YOKTUR ve commit EDİLMEZ: derlemeden önce
// `bash scripts/spike-cues-jvm.sh` çalıştırılmalıdır — Swift eşdeğerinin
// `apple-harness/Package.swift`'teki uyarısıyla aynı.
//
// Sistemde Gradle kurulu olması gerekmez: bu proje kendi wrapper'ını
// (`gradlew`) taşır. `doctor.sh`: `gradle:* → info`, hiçbir milestone'da
// blocker değil.

plugins {
    kotlin("jvm") version "2.4.0"
    application
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("net.java.dev.jna:jna:5.15.0")
    testImplementation(kotlin("test"))
}

kotlin {
    sourceSets["main"].kotlin.srcDir("generated")
}

application {
    mainClass.set("MainKt")
}

// UniFFI'ın ürettiği Kotlin, JNA üzerinden native kütüphaneyi çalışma
// zamanında `jna.library.path`'ten yükler — Swift'in `-L/-l` static-link
// ettiği yerin JVM karşılığı. `spike-cues-jvm.sh` `.dylib`'i buraya kopyalar.
val nativeLibDir = layout.projectDirectory.dir("generated/lib")

tasks.withType<JavaExec> {
    systemProperty("jna.library.path", nativeLibDir.asFile.absolutePath)
}

tasks.test {
    useJUnitPlatform()
    systemProperty("jna.library.path", nativeLibDir.asFile.absolutePath)
}
