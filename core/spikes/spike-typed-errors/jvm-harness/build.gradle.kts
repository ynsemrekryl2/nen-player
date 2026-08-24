// NEN-011 JVM ölçüm harness'ı — ÜRÜN KODU DEĞİLDİR (CLAUDE.md kural 7).
//
// Ürünün Kotlin/Gradle modülü yok (platforms/android hâlâ boş iskelet) ve bu
// paketi tanımaz; ADR-0028'in birinci sınırı: spike binding'i ürün paketine
// linklenmez.
//
// `generated/` YOKTUR ve commit EDİLMEZ: derlemeden önce
// `bash scripts/spike-typed-errors-jvm.sh` çalıştırılmalıdır.
//
// Sistemde Gradle kurulu olması gerekmez — kendi wrapper'ını taşır.

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

val nativeLibDir = layout.projectDirectory.dir("generated/lib")

tasks.withType<JavaExec> {
    systemProperty("jna.library.path", nativeLibDir.asFile.absolutePath)
}

tasks.test {
    useJUnitPlatform()
    systemProperty("jna.library.path", nativeLibDir.asFile.absolutePath)
}
