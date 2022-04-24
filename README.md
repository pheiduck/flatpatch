# flatpatch
flatpatch is a Script which uninstall snap from Ubuntu and install flatpak

## Install

### Ubuntu 22.04 or above:

```bash
git clone https://github.com/pheiduck/flatpatch.git
cd flatpatch
sudo ./flatpatch
```
or
```bash
curl -sSL https://raw.githubusercontent.com/pheiduck/flatpatch/main/flatpatch | bash
```

### Ubuntu 18.04:

```bash
git clone https://github.com/pheiduck/flatpatch.git
cd flatpatch
sed -i '29s/.//' flatpatch
sudo ./flatpatch
```

# Introduction video

[![Introduction of Flatpatch](https://img.youtube.com/vi/Z6TxxSECv6Y/0.jpg)](https://www.youtube.com/embed/Z6TxxSECv6Y)
