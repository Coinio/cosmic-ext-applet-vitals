# COSMIC Vitals
![Main Image](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/main-image.png)


A minimal configurable applet to display basic system resource usage, i.e. CPU, Memory, Network. 

This came from wanting a project to build to learn some Rust, being a Pop_OS COSMIC user and 
not finding other applets that worked as I wanted. Hopefully, the code in the repo will be useful to 
for those fighting with iced/libcosmic, etc.

## Features

* Show CPU Usage
* Show RAM Usage
* Show Network upload / download. This filters out virtual devices so won't count upload / download twice when using a VPN, etc.
* Show Disk upload / down. This counts reads / writes to logical disks, not partitions, etc.
* Horizontal / Vertical Layouts - The horizontal layout has seen a little more testing as I 
  don't use the vertical.
* Config:
  * Read Intervals - How often each monitor is polled
  * Max Samples - The number of samples to keep to average the readings across
  * Show / Hide each monitor
  * Show / Hide the labels for each monitor
  * Label Colours - The colours can be selected from the current theme libcosmic palette
  * Label Text - The label text for each monitor
  * Switch between SI and IEC units

## Screenshots

##### Horizontal Layout Example 1
![Horizontal Panel Layout](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/horizontal-layout.png)

##### Horizontal Layout Example 2
![Horizontal Panel Layout 2](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/horizontal-layout-2.png)

##### Horizontal Layout Example 2
![Horizontal Panel Layout 3](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/horizontal-layout-3.png)

##### Vertical Layout
![Vertical Panel Layout](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/vertical-layout.jpg)
##### Main Settings Page
![Main Settings Page](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/main-settings.png)
##### Network Settings Page
![Example Settings Page](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/example-network-settings.png)
##### General Settings Page
![Example Settings Page](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/general-settings.png)

## Install

To install your COSMIC applet, you will need [just](https://github.com/casey/just), if you're on Pop!\_OS, you can install it with the following commands:

```sh
sudo apt install just libxkbcommon-dev
sudo apt install just
```

After you install it, you can run the following commands to build and install your applet:

```sh
just build-release
sudo just install
```
