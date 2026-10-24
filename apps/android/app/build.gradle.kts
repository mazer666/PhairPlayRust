import org.gradle.api.tasks.TaskProvider

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

// The root of the repository. The Rust sources live two levels above the
// Gradle project.
val repositoryRoot = rootProject.projectDir.parentFile.parentFile

android {
    namespace = "org.phairplay.receiver"
    compileSdk = 35

    defaultConfig {
        applicationId = "org.phairplay.receiver"
        // Android 13 (API 33) is the lowest version.
        //
        // WHY: the Miracast engine needs the Wi-Fi Direct APIs of the
        // platform. On Android 13 and later they are covered by
        // `NEARBY_WIFI_DEVICES` with the flag `neverForLocation`, so that
        // the shell does not have to ask for a location permission at all
        // (FR-075). On older versions the platform would force a location
        // permission onto every user of the application.
        minSdk = 33
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"

        ndk {
            // The primary release ABI of the application.
            abiFilters += listOf("arm64-v8a")
        }
    }

    buildTypes {
        release {
            // The names of the native library are kept, so that the
            // UniFFI bindings find `libphairplay_ffi.so` in the package.
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    packaging {
        resources {
            excludes += setOf(
                "/META-INF/{AL2.0,LGPL2.1}",
                "/META-INF/versions/9/OSGI-INF/MANIFEST.MF",
            )
        }
    }

    testOptions {
        unitTests {
            isReturnDefaultValues = true
        }
    }
}

dependencies {
    val composeVersion = "2024.10.01"
    implementation(platform("androidx.compose:compose-bom:$composeVersion"))
    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.ui:ui-tooling-preview")
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.material:material-icons-core")
    implementation("androidx.lifecycle:lifecycle-runtime-compose:2.8.7")
    implementation("androidx.lifecycle:lifecycle-viewmodel-compose:2.8.7")
    implementation("androidx.core:core-ktx:1.15.0")

    // The UniFFI bindings of Kotlin use JNA to call the shared library of
    // the bridge. The `aar` variant is the one that the Android runtime
    // can load; the plain jar is only used by the unit tests.
    implementation("net.java.dev.jna:jna:5.15.0@aar")

    debugImplementation("androidx.compose.ui:ui-tooling")

    testImplementation("junit:junit:4.13.2")
    testImplementation("net.java.dev.jna:jna:5.15.0")
    testImplementation("org.jetbrains.kotlinx:kotlinx-coroutines-test:1.9.0")
}

/**
 * Compiles the native bridge of the receiver for Android ARM64.
 *
 * Every variant gets **its own** task. A single shared task cannot work:
 * the build profile is an environment variable of the task, and the
 * environment of a task that is configured for several variants is the one
 * of the last variant that was configured. The debug build would then
 * ship the release library, or the other way round.
 */
fun registerBuildRustBridge(name: String, profile: String, sourceSet: String): TaskProvider<Exec> {
    return tasks.register<Exec>(name) {
        description =
            "Compiles the native bridge (libphairplay_ffi.so) for Android ARM64 ($profile)"
        val ndk = android.ndkDirectory
        val jniDir = projectDir.resolve("src/$sourceSet/jniLibs/arm64-v8a")
        val library = jniDir.resolve("libphairplay_ffi.so")

        inputs.dir(repositoryRoot.resolve("platform"))
        inputs.dir(repositoryRoot.resolve("core"))
        inputs.file(repositoryRoot.resolve("Cargo.toml"))
        // The profile is an input, so that switching the variant rebuilds
        // the library instead of reusing the one of the other profile.
        inputs.property("buildProfile", profile)
        inputs.property("ndkDirectory", ndk.absolutePath)
        outputs.file(library)

        workingDir(repositoryRoot)
        commandLine("cargo", "build", "--target", "aarch64-linux-android", "-p", "phairplay-ffi")
        environment("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER", ndk.resolve("toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang").absolutePath)
        environment("CARGO_TARGET_AARCH64_LINUX_ANDROID_AR", ndk.resolve("toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar").absolutePath)
        environment("CC_aarch64_linux_android", ndk.resolve("toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android24-clang").absolutePath)
        environment("ANDROID_NDK_HOME", ndk.absolutePath)
        environment("CARGO_PROFILE_DEV_DEBUG", if (profile == "release") "0" else "2")
        doFirst {
            // The directory of the libraries is created by the task itself,
            // so that a build that never ran the task does not leave an
            // empty directory behind.
            jniDir.mkdirs()
            delete(library)
        }
        doLast {
            val built = repositoryRoot.resolve(
                "target/aarch64-linux-android/$profile/libphairplay_ffi.so",
            )
            if (!built.exists()) {
                throw GradleException(
                    "the native bridge was not built: ${built.absolutePath} does not exist",
                )
            }
            copy {
                from(built)
                into(jniDir)
            }
        }
    }
}

// The native library is built for the variant that is being built, and it
// is deployed into the source set of that variant.
androidComponents {
    onVariants { variant ->
        val profile = if (variant.buildType == "release") "release" else "debug"
        val variantName = variant.name.replaceFirstChar { it.uppercase() }
        // The source set of the variant: `debug` or `release`. The build
        // type is nullable in the API of the plugin; the name of the
        // variant carries the same word when the project has no flavours.
        val sourceSet = variant.buildType ?: variant.name
        val task = registerBuildRustBridge("buildRustBridge$variantName", profile, sourceSet)
        tasks.matching { it.name == "merge${variantName}JniLibFolders" }.configureEach {
            dependsOn(task)
        }
    }
}

// The unit tests load the shared library of the bridge from the build
// directory of the workspace, so that the tests of the mapping between the
// DTOs of the bridge and the model of the shell can run against the real
// library.
tasks.withType<Test> {
    val targetDebug = repositoryRoot.resolve("target/debug").absolutePath
    systemProperty("jna.library.path", targetDebug)
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}
