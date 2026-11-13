# COSMIC Vitals
![COSMIC Vitals in the panel](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/main-image.png)


A minimal configurable applet to display basic system resource usage, i.e. CPU, Memory, Network. 

This came from wanting a project to build to learn some Rust, being a Pop_OS COSMIC user and 
not finding other applets that worked as I wanted. Hopefully, the code in the repo will be useful to 
for those fighting with iced/libcosmic, etc.

## Features

* CPU, memory, GPU, network, disk and storage usage
* Hover any indicator for a detailed breakdown
* CPU temperature and fan speed ([setup](#cpu-temperature-and-fan-speed))
* GPU usage and VRAM for AMD and NVIDIA GPUs ([details](#gpu-usage-and-vram))
* Click any indicator to open its settings
* Horizontal and vertical panel layouts
* Show or hide each indicator and icon
* Custom icon colours
* Configurable update intervals and sample averaging
* Choose which mounts count towards storage
* SI or IEC units
* Any installed font ([recommended](#fonts))

## Screenshots

##### CPU Details
The CPU's model, cores, threads and clock speed, its temperature, and the usage of each thread.

![CPU Detail Popup](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/cpu-detail.png)

##### Memory Details
Memory used, cached and available, and the swap used.

![Memory Detail Popup](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/memory-detail.png)

##### GPU Details
Usage and VRAM of each GPU, plus its temperature and power draw where reported.

![GPU Detail Popup](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/gpu-detail.png)

##### Network Details
Download / upload and IPv4 address of each interface.

![Network Detail Popup](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/network-detail.png)

##### Disk Details
Reads / writes, model and busy time of each disk, including disks that aren't mounted.

![Disk Detail Popup](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/disk-detail.png)

##### Storage Details
Used space of each included mount, grouped by drive with its model, and the size of any partitions that aren't mounted.

![Storage Detail Popup](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/storage-detail.png)

##### Storage Settings Page
Each indicator has its own settings page. The storage page shows the custom icon colour picker and the mounts
included in the total.

![Storage Settings Page](https://github.com/Coinio/cosmic-vitals/blob/main/res/screenshots/storage-settings.png)

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

## GPU Usage and VRAM

The GPU indicator shows the usage and VRAM used of one GPU, the discrete one by default. Another can be chosen under
**Settings > GPU > Panel GPU**. Hovering it shows every GPU, with its temperature and power draw where the GPU reports
them.

* **AMD** - Read from the files the `amdgpu` driver provides. Nothing extra is needed.
* **NVIDIA** - Read through NVML (`libnvidia-ml.so.1`), which comes with NVIDIA's driver. The applet loads it when an
  NVIDIA GPU is first read, so it doesn't need to be installed on systems without one.
* **Intel** - Not supported, as Intel's drivers don't report the usage cheaply. Without a supported GPU, there's no
  GPU indicator.

Laptops put their discrete GPU to sleep when it isn't in use, to save battery, and reading a GPU that's asleep wakes
it. So the applet never reads a GPU that's asleep, and shows its usage as 0%. A GPU only falls asleep after it has been
idle for a few seconds, and reading it counts as using it, so once a GPU that can sleep has been idle for a few
seconds, the applet leaves it alone long enough for it to fall asleep. The popup shows it as idle meanwhile. If
something starts using it during that time, the indicator catches up within 15 seconds.
