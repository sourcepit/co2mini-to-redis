# Cross-Compiling for Raspberry Pi

## Prepare Rust

See https://github.com/japaric/rust-cross#tldr-ubuntu-example

## Install required Lips

For sake of easiness we use Ubuntus Multiarch features to install required pre-compiled libs.

- udev
- hidapi-libusb

### Prepare APT

See https://wiki.debian.org/Multiarch/HOWTO.

- Pin current APT repos to current systems architecture by adding an arch filter to each source definition:
  
  ```
  # old
  deb http://us.archive.ubuntu.com/ubuntu/ focal main restricted
  # new
  deb [arch=amd64,i386] http://us.archive.ubuntu.com/ubuntu/ focal main restricted
  ```
  
- Add sources for `armhf` like this:
  
  ```
  deb [arch=armhf] http://ports.ubuntu.com/ubuntu-ports focal universe
  ```
  
- `sudo dpkg --add-architecture armhf`  

### Install Libs

```
sudo apt-get update
sudo apt-get install libudev-dev:armhf libhidapi-dev:armhf
```
