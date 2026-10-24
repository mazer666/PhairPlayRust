// The plugins of the Gradle build of the Android shell.
//
// WHAT: the versions of the three plugins that the application module
// uses. They are declared here with `apply false`, so that the module can
// switch them on without repeating a version.
//
// WHY: a version that is written in two places drifts. The Android Gradle
// Plugin and the Kotlin plugin have to agree with each other, because the
// Compose compiler of the Kotlin version has to understand the Compose
// runtime of the Android Gradle Plugin.
//
// EXAMPLE: `./gradlew :app:tasks` lists the tasks of the application.

plugins {
    id("com.android.application") version "8.7.2" apply false
    id("org.jetbrains.kotlin.android") version "2.0.21" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.0.21" apply false
}
