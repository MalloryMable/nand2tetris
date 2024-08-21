## Memory
Building computer memory out of flip-flops.
Split into two directories between component chips and increasingly large RAM denominations.

### Data-Flip-Flop (DFF)
A gate that remains flipped until the counter signal is sent. This, incidentally, is also how car ignitions work.

### Binary Cell (Bit)
A single bit of memory. Built out of DFF and a MUX.

### Register
16 binary cells chained together. The basic unit of memory in the HACK architecture.

### Counter (PC)
A mechanism for storing the data from the 16 bit incrementer chip.


### RAM Sticks
Notice that through 4 levels of 8 way ram we have created 16 bits worth of potential registers.

#### RAM8
8 Registers muxed together with 8WAY DMUX.

#### RAM64
8 RAM8 chips demuxed together.

#### RAM512
8 RAM64 chips demuxed together.

#### RAM4K
8 RAM512 chips demuxed together.

#### RAM16K
8 RAM4K chips demuxed together.