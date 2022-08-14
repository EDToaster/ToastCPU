- [Toast CPU](#toast-cpu)
- [Todo](#todo)

# Toast CPU
`toast` is a 16-bit architecture spec, an emulator, an FPGA implementation, and an assembler.

Read documents on the [architecture](spec/arch.md), and the [application binary interface](spec/abi.md).

# Todo
- [ ] Fix assembler to include calling convention and register renamings
- [ ] Fix the emulator, create an emulated screen
- [x] Finish VGA text mode (Maybe a smaller VGA text mode font for more lines)
- [ ] VGA buffer currently takes up too much of the IO mapped space, maybe consider changing to 0x0000: Address, 0x0001: Value type of memory addressing.
- [ ] Implement elementary OS / Basic interpreter
- [ ] Extend functionality of assembler
    - [ ] Add `call` macro
- [ ] Implement a compiler backend for C->`toast` compilation
- [ ] Add floating point module, maybe a 32-bit variant?
- [x] IRQ and handlers (keyboard driver testing TBD...)
- [ ] Implement a `rti` instruction that allows for more interrupts to occur. Currently, interrupts will nest themselves and cause a ton of issues.
- [ ] Add Register Banking (Maybe...)