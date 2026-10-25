# JNI calls from Rust require stable Activity and GameActivity class/method names.
-keep class com.duidev.craftmobile.CraftActivity { *; }
-keep class com.google.androidgamesdk.** { *; }
-keepclasseswithmembernames,includedescriptorclasses class * { native <methods>; }
