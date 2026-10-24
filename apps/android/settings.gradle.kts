// The settings of the Gradle build of the Android shell.
//
// WHAT: the name of the build, the modules of the project and the
// repositories that the plugins and the libraries come from.
//
// WHY: the repositories are declared **here** and not in the modules, with
// `FAIL_ON_PROJECT_REPOS`. A module that adds a repository of its own
// would pull a different version of a library into the build, and the
// result would depend on the order in which the modules were built.
//
// EXAMPLE: `./gradlew :app:assembleDebug` builds the debug variant of the
// Android application.

pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "PhairPlayAndroid"
include(":app")
