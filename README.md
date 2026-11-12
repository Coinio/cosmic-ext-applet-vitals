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
* Show available / total storage. Choose which mounts are included in the settings, `/` by default. A filesystem
  mounted in more than one place, e.g. btrfs subvolumes, is only counted once.
* Hover over the CPU indicator for the usage of each thread, or each physical core, and the CPU temperature and fan
  speed where the system reports them. See [CPU Temperature and Fan Speed](#cpu-temperature-and-fan-speed)
* Hover over the storage indicator for the free space of every mount
* Click an indicator to open its settings
* Horizontal / Vertical Layouts - The horizontal layout has seen a little more testing as I 
  don't use the vertical.
* Config:
  * Read Intervals - How often each monitor is polled
  * Max Samples - The number of samples to keep to average the readings across
  * Show / Hide each monitor
  * Show / Hide the CPU and memory icons
  * Icon Colours - The CPU and memory icon colours can be selected from the current theme libcosmic palette
  * Switch between SI and IEC units
  * Font - Uses the COSMIC system font by default, or switch to any installed font. See [Fonts](#fonts)

## Screenshots

##### Horizontal Layout Example 1
![Horizontal Panel Layout](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/horizontal-layout.png)

##### Horizontal Layout Example 2
![Horizontal Panel Layout 2](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/horizontal-layout-2.png)

##### Horizontal Layout Example 3
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
```

After just has been installed, navigate to the root of the cosmic-ext-applet-vitals project and install the applet with these commands:
 
```sh
just build-release
sudo just install
```

Then you can add the applet to the panel / dock as usual.

## Fonts

The indicators use the COSMIC system font by default. For the best results, a monospaced font is recommended, as
every digit is the same width and the values won't shift around as they update. The applet was designed around
[JetBrains Mono](https://www.jetbrains.com/lp/mono/), which can be installed from most distribution repositories:

```sh
# Pop!_OS / Ubuntu / Debian
sudo apt install fonts-jetbrains-mono

# Fedora
sudo dnf install jetbrains-mono-fonts

# Arch
sudo pacman -S ttf-jetbrains-mono
```

Or download it from the [JetBrains Mono releases](https://github.com/JetBrains/JetBrainsMono/releases). Then select it
under **Settings > General > Font**. If a newly installed font doesn't show up in the list, log out and back in so
the panel reloads its fonts.

## CPU Temperature and Fan Speed

The CPU detail popup reads the temperature and fan speed from the kernel's hardware monitoring drivers, the same
sensors `sensors` from lm-sensors shows. Nothing extra needs installing for the applet, but the driver for your
hardware has to be loaded. Each reading only shows in the popup when there's a sensor for it.

By default, the applet picks the sensors automatically:

* **Temperature** - The CPU's own sensor, e.g. `k10temp` on AMD, `coretemp` on Intel, or `cpu_thermal` on a
  Raspberry Pi. These load automatically on almost every system.
* **Fan** - A fan labelled as the CPU fan, or the only fan. Many motherboards don't label their fans, so if there are
  several, choose the one connected to the CPU fan header under **Settings > CPU > Fan Sensor**. Each sensor is listed
  with its current reading to help tell them apart.

If no fans are found, your motherboard's sensor chip probably needs its driver loading. `sensors-detect` from
lm-sensors finds the right one:

```sh
# Pop!_OS / Ubuntu / Debian
sudo apt install lm-sensors
sudo sensors-detect
```

It offers to load the driver on every boot. Restart the applet afterwards, e.g. by logging out and back in, so it
finds the new sensors. A few chips, e.g. on some MSI and Gigabyte boards, need a newer kernel or an out-of-tree driver,
and many laptops don't report their fans at all.
