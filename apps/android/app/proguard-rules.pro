# The rules of the shrinking of the release build.
#
# WHY: The bridge is reached through the generated bindings of UniFFI, and
# those bindings use the reflection of JNA to find the callbacks of the
# host. A shrinking that removes the names of the callback interfaces would
# break the bridge at runtime, and the failure would only show up on a
# device.
-keep class com.sun.jna.** { *; }
-keep class * implements com.sun.jna.** { *; }
-keepclassmembers class uniffi.phairplay_ffi.** { *; }
-keep class uniffi.phairplay_ffi.** { *; }
-keepclassmembers class * {
    @com.sun.jna.* <methods>;
}
-dontwarn com.sun.jna.**
