## Arithmetic Chips
A series of chips built for doing simple addition and subtration.

### Half-Adder
Outputs both a sum bit and a carry bit.

### Full-Adder
By adding a third bit in carry bits are now part of addition.

### Adder (Add16)
Addition implemented by beggining with a half-adder and then using full adders until bit limit is reached.

### Incrementer (Inc16)
Specialized chip for incremental addition opperations as these are very common.

### Arithmetic Logic Unit (ALU)
The ALU takes 2 16 bit digits as well as a series of single bit inputs.
Single bit inputs are:
* zx: bit thrown to zero x input
* nx: bit thrown to negate x input
* zy: bit thrown to zero y input
* ny: bit thrown to negate y input
* f: function code. 1 for ADD, 0 for AND
* no: negate output

The ALU outputs both the result of the desired opperation and 2 additional bits of information for control flow purposes.
Single bit outputs are:
* zr: True if output is equal to zero
* ng: True if output is less than zero 