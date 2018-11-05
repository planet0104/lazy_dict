 REM cargo build --target aarch64-linux-android --release
 REM copy target\aarch64-linux-android\release\liblazy_dict.so ..\lazy_dict\app\src\main\jniLibs\arm64-v8a\liblazy_dict.so
 cargo build --target armv7-linux-androideabi --release
 copy target\armv7-linux-androideabi\release\liblazy_dict.so ..\lazy_dict\app\src\main\jniLibs\armeabi-v7a\liblazy_dict.so