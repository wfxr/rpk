RPK
---

A lightweight, cross-platform cli package manager.

### Installation

Download the latest release from the [releases page](https://github.com/wfxr/rpk/releases)
or just run the following command:

```bash
curl -fsSL https://raw.githubusercontent.com/wfxr/rpk/main/scripts/install \
    | bash -s -- --to ~/bin
```

You can port the configuration from remote after installation:

```bash
rpk init <config_url>
```

Or do it in one line:

```bash
curl -fsSL https://raw.githubusercontent.com/wfxr/rpk/main/scripts/install \
    | bash -s -- --to ~/bin --init <config_url>
```
