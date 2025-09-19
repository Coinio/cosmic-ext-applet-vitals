# COSMIC Vitals

A minimal applet to display basic system resource usage, i.e. CPU, Memory, Network. 

I started building this to learn some Rust and play around with Pop_OS COSMIC. It's definitely a WIP but works pretty well for my needs. Hopefully the code will be useful to somebody else fighting with iced/libcosmic, etc.

## Features

* Show CPU Usage
* Show RAM Usage
* Show Network upload / download. This filters out virtual devices so won't count upload / download twice when using a VPN, etc.
* Show Disk upload / down. This counts reads / writes to logical disks, not partitions, etc.
* Things that can be configured:
  * Read Intervals - How often each monitor is polled
  * Max Samples - The number of samples to keep to average the readings across
  * Show / Hide each monitor

## Things I might fix

* Calculate the maximum width of the labels so the indicators don't expand / contract
* Improve the vertical layout
* Add configurable units KiBs/MiBs/GiBs or progressively change units depending on how large / small reading is
* Better logging into journalctl

## Screenshots

##### Horizontal Layout
![Horizontal Panel Layout](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/horizontal-layout.png)
##### Vertical Layout
![Vertical Panel Layout](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/vertical-layout.jpg)
##### Main Settings Page
![Main Settings Page](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/main-settings.png)
##### Example Settings Page
![Example Settings Page](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/example-network-settings.png)

## Install

To install your COSMIC applet, you will need [just](https://github.com/casey/just), if you're on Pop!\_OS, you can install it with the following command:

```sh
sudo apt install just
```

After you install it, you can run the following commands to build and install your applet:

```sh
just build-release
sudo just install
```
