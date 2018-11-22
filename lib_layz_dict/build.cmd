cargo build --target aarch64-linux-android
copy target\aarch64-linux-android\debug\liblazy_dict.so ..\lazy_dict\app\src\main\jniLibs\arm64-v8a\liblazy_dict.so
REM  cargo build --target armv7-linux-androideabi
REM  copy target\armv7-linux-androideabi\debug\liblazy_dict.so ..\lazy_dict\app\src\main\jniLibs\armeabi-v7a\liblazy_dict.so