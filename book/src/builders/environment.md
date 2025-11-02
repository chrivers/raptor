# Environment variables

Several settings can be adjusted via environment variables. These can be specified either on the command line:

```sh
sudo raptor run -e OUTPUT_FORMAT=qcow2 ...
```

or in `Raptor.toml`:

```toml
[run.example-target]
target = "example"

# notice the `env.` prefix!
env.OUTPUT_FORMAT = "qcow2"
```

## Environment reference

| Variable        | `deblive` | `live-disk-image` | `disk-image` | `part-image` |
|:----------------|-----------|-------------------|--------------|--------------|
| `GRUB_TIMEOUT`  | ✅        | ✅                | ❌           | ❌           |
| `OUTPUT_FORMAT` | ❌        | ✅                | ✅           | ✅           |
| `EFI_MB`        | ❌        | ✅                | ✅           | ❌           |
| `BOOT_MB`       | ❌        | ✅                | ✅           | ❌           |
| `FREE_MB`       | ❌        | ✅                | ✅           | ✅           |
| `USED_MB`       | ❌        | ✅                | ✅           | ✅           |
| `SIZE_MB`       | ❌        | ✅                | ✅           | ✅           |

### **`GRUB_TIMEOUT`**

- Default: `5`
- Supported by: `deblive`, `live-disk-image`

The auto-generated GRUB boot menu will use `GRUB_TIMEOUT` as the countdown until
automatically booting the first target.

Setting this to `0` will boot without showing the GRUB menu.

### **`OUTPUT_FORMAT`**

- Default: `raw`
- Supported by: `live-disk-image`, `disk-image`, `part-image`

Output format of the image. Common values include `raw` and `qcow2`.

See `qemu-img --help` for the complete list

### **`EFI_MB`**

- Default: `32`
- Supported by: `live-disk-image`, `disk-image`

Size of ESP, the EFI System Partition (`/boot/efi`).

### **`BOOT_MB`**

- Default: `512`
- Supported by: `live-disk-image`, `disk-image`

Size of the boot partition (`/boot`)

### **`FREE_MB`**

- Default: `512`
- Supported by: `live-disk-image`, `disk-image`, `part-image`

How much free space to target in the resulting image.

### **`USED_MB`**

- Default: **none**
- Supported by: `live-disk-image`, `disk-image`, `part-image`

Total disk space of all files in the root filesystem.

Calculated when building the image, unless specified manually.

### **`SIZE_MB`**

- Default: **none**
- Supported by: `live-disk-image`, `disk-image`, `part-image`

The size of the resulting image.

Calculated as:
 - `live-disk-image`: `EFI_MB + BOOT_MB + USED_MB + FREE_MB`
 - `disk-image`: `EFI_MB + BOOT_MB + USED_MB + FREE_MB`
 - `part-image`: `USED_MB + FREE_MB`

unless specified manually for exact control.
