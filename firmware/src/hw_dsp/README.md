# Hardware DSP

This module interfaces with the CS47L63 driver. The module consists of a collection of register values known to work with the nRF5340 Audio DK and Yote. It also exposes a shared bus to let different async tasks share the same SPI bus.
This shared bus design is no good for low power since it currently does not disable the SPI bus after use. Therefore there are duplicate `_spi` prefixed functions that do the same thing but without a mutex. This should be fixed soon.