import org.gradle.api.tasks.TaskProvider
import java.util.Properties
import java.io.FileInputStream

// The properties of the signing of the release.
//
// WHY the order: the file beside the project is for the machine of the
// owner of the key, the environment variable is for the machine that builds
// without a checkout of the project, and the file of the Gradle home is for
// the machine that builds for everybody. The first one that exists wins.
//
// WHY the key does not live in the repository: a signing key is a secret,
// and a secret in a repository is a secret that everybody has. The key of a
// release is the private half of the identity of the application.
//
// WHY the release still builds without a key: an APK that is not signed
// cannot be installed, and a build that fails without a key is a build that
// nobody but the owner of the key can run. The release build therefore falls
// back to a build that is **not** signed, and the acceptance record names
// the APK as **not signed** — a signed APK is the trigger of the beta in
// the release plan, and the owner of the key decides when it is reached.
//
// EXAMPLE:
//
//   printf 'phairplay.signing.storeFile=/secure/keys/phairplay.jks\n\
//   phairplay.signing.storePassword=…\n\
//   phairplay.signing.keyAlias=phairplay\n\
//   phairplay.signing.keyPassword=…\n' > ~/.gradle/gradle.properties
fun signingProperties(): Properties? {
    val candidates = listOfNotNull(
        System.getenv("ANDROID_SIGNING_PROPERTIES")?.let { File(it) },
        project.rootProject.file("keystore.properties").takeIf { it.exists() },
        File(System.getProperty("user.home"), ".gradle/gradle.properties")
            .takeIf { it.exists() },
    )
    for (candidate in candidates) {
        val properties = Properties()
        FileInputStream(candidate).use { properties.load(it) }
        if (properties.getProperty("phairplay.signing.storeFile") != null) {
            return properties
        }
    }
    return null
}

val signing = signingProperties()

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
        versionName = "0.3.0"

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

            if (signing != null) {
                // WHY the check on the file: a property that points at a key
                // that does not exist is a broken build with a message that
                // says nothing. The check says what is missing.
                val storeFile = signing.getProperty("phairplay.signing.storeFile")
                val store = file(storeFile)
                if (!store.exists()) {
                    throw GradleException(
                        "the signing key $storeFile does not exist — the release cannot be signed",
                    )
                }
                signingConfig = signingConfigs.create("release") {
                    this.storeFile = store
                    storePassword = signing.getProperty("phairplay.signing.storePassword")
                    keyAlias = signing.getProperty("phairplay.signing.keyAlias")
                    keyPassword = signing.getProperty("phairplay.signing.keyPassword")
                }
            }
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
        // WHY the profile on the command line: the task builds the library
        // for the variant that is being built — the release variant needs
        // the release profile of Cargo, and the debug variant the debug
        // profile. Without the flag the build always uses the debug profile,
        // and the release variant ships a library that was never optimised.
        commandLine(
            "cargo",
            "build",
            "--target",
            "aarch64-linux-android",
            "-p",
            "phairplay-ffi",
            "--profile",
            profile,
        )
        // WHY the toolchain in the PATH: a dependency with a C part — `ring`
        // — looks for the compiler and the archiver of the target **by
        // name**, and the name of the archiver of the NDK is `llvm-ar` and
        // not `aarch64-linux-android-ar`. The build script of that
        // dependency therefore finds nothing, and the build stops with
        // "failed to find tool".
        val toolchain = ndk.resolve("toolchains/llvm/prebuilt/linux-x86_64/bin")
        environment("PATH", "${toolchain.absolutePath}:${System.getenv("PATH")}")
        environment("CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER", toolchain.resolve("aarch64-linux-android24-clang").absolutePath)
        environment("CARGO_TARGET_AARCH64_LINUX_ANDROID_AR", toolchain.resolve("llvm-ar").absolutePath)
        environment("CC_aarch64_linux_android", toolchain.resolve("aarch64-linux-android24-clang").absolutePath)
        environment("AR_aarch64_linux_android", toolchain.resolve("llvm-ar").absolutePath)
        environment("RANLIB_aarch64_linux_android", toolchain.resolve("llvm-ar").absolutePath)
        environment("ANDROID_NDK_HOME", ndk.absolutePath)
        // WHY the profile is not touched here: the profile of Cargo decides
        // itself what it optimises. The environment variable that was set
        // here overwrote the debug information of the release profile, and
        // the release shipped a library with the symbols of a debug build.
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
