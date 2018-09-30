#windows 配置rust ndk环境

1、下载 C:\android-ndk-r17-beta2

2、配置环境变量 NDK_HOME=C:\android-ndk-r17-beta2

3、创建工具链
mkdir NDK

PS C:\> python C:\android-ndk-r17-beta2\build\tools\make_standalone_toolchain.py --api 26 --arch arm64 --install-dir C:\NDK\arm64

PS C:\> python C:\android-ndk-r17-beta2\build\tools\make_standalone_toolchain.py --api 26 --arch arm --install-dir C:\NDK\arm

PS C:\> python C:\android-ndk-r17-beta2\build\tools\make_standalone_toolchain.py --api 26 --arch x86 --install-dir C:\NDK\x86


4、配置.cargo

[target.aarch64-linux-android]
ar = "C:/NDK/arm64/bin/aarch64-linux-android-ar"
linker = "C:/NDK/arm64/bin/aarch64-linux-android-g++"

[target.armv7-linux-androideabi]
ar = "C:/NDK/arm/bin/arm-linux-androideabi-ar"
linker = "C:/NDK/arm/bin/arm-linux-androideabi-g++"

[target.i686-linux-android]
ar = "C:/NDK/x86/bin/i686-linux-android-ar"
linker = "C:/NDK/x86/bin/i686-linux-android-g++"

5、添加编译目标

rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android
