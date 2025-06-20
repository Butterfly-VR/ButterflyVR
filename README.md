
# ButterflyVR
ButterflyVR is a new open source social VR game, built with godot and rust.
> [!CAUTION] 
> ButterflyVR is currently in a pre-alpha state. It is not representative of the final product and lacks basic features. 
> 
> In other words, if your looking for a game you can play this is not for you.
## features
Currently ButterflyVR is very early in development and as such has minimal features. If you want to find out about future planned features consider visiting the [roadmap.](#)

**current features:**
* Custom rust netcode using [netcode](https://github.com/benny-n/netcode) for security
* Initial support for user created content
* IK legs
* Simple server with text chat and network syncing for objects and avatars

## build instructions
1. Clone the repo
```bash
git clone https://github.com/Butterfly-VR/ButterflyVR.git
```
3. Build the rust module, if you plan to export without debug, build with -r
```bash
cd rust/butteryfly-rs-module 
cargo build
```
5. Download and open the [godot editor](https://godotengine.org/) (4.4.1)
6. Export the project using the correct template for your platform
## docs
Documentation is available within the project as godot docs, and a [wiki](#) is available for information about the project
## contact
Consider joining the [discord](https://discord.gg/vHdewgkj3e) for the latest development information or to contact us. 

If you wish to contribute please see the [contributing](contributing.md) file.

If you wish to open an issue please use the available templates and fill in the requested information

## license
ButterflyVR is licensed under GPLv3
