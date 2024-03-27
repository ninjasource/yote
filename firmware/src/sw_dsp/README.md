# Software based Audio DSP library

The chapro library is used for various DSP operations. It can be found here:

https://github.com/BoysTownOrg/chapro

This is a dependency of the lib above:

https://github.com/BoysTownOrg/sigpro

Edit the `makefile.arm`` in each repo to change the compiler to target the nrf5340 chip as follows:
Note that hardware float has been enabled. 

```
CFLAGS = -W -Wall -Os -mthumb -ffunction-sections -fdata-sections -mcpu=cortex-m33 -DBR_ARMEL_CORTEXM_GCC -I$(INCDIR) -Wno-unused-local-typedef -fPIC -mfpu=fpv5-sp-d16 -mfloat-abi=hard
CC=arm-none-eabi-gcc
AR=arm-none-eabi-ar
```

Build and install sigpro like so:

```
# in sigpro cloned folder
sudo make -f makefile.arm install
```

Build chapro like so and copy the binary to the lib folder here:

```
# in chapro cloned folder
make -f makefile.arm libchapro.a
cp libchapro.a ../nrf5340-audio-dk-cs47l63/lib
```

Run bindgen to generate `chapro.rs` if it hasn't already been generated (or if it is out of date) 
NOTE: you may need to `cargo install bindgen-cli` first

```
# in this folder (same as this README.md file) 
bindgen --use-core chapro.h -o ../src/chapro.rs
```


