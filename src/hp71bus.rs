
enum Command {
    Id,
    Config,
}

///
struct BusDevice {
    // state
    configured: bool,

    // per device


    /// 5-nibbles used (2.5 bytes)
    /// 
    /// sent in responce to `Command::Id` when `daisy_in` is high.
    /// 
    /// Nibbles:
    /// 
    /// 0: (14 - log2(size in KB)) if memory
    ///         9 to F allowed for RAM
    ///         7 to F allowed for other memory
    ///    (18 - log2(size in KB)) if memory-mapped-io
    /// 1: reserved (0? 0xf?)
    /// 2: device type
    ///     0: RAM
    ///     1: ROM
    ///     2 to E: unassigned (reserved?)
    ///     F: memory-mapped-io
    /// 3: if memory: unassigned (reserved?)
    ///    if memory-mapped-io: 0 = HP-IL mailbox. 1 to F = unassigned (reserved?)
    /// 4: high bit set if device is last in daisy chain (how does it know it's last?)
    id: u32
}

struct BusSignalsIn {
    // if not `configured`, only respond to `Command::Config` and `Command::Id` when `daisy_in` is high.
    daisy_in: bool,
}

struct BusSignalsOut {
    daisy_out: bool,
}

impl BusDevice {
    fn command(&mut self, cmd: Command, sig: BusSignalsIn)
    {
        if !self.configured && !sig.daisy_in {
            // no action if unconfigured and no daisy_in present
            return;
        }

        if !self.configured && sig.daisy_in {
            // respond to `Id` or `Config` only
        }

        unimplemented!()
    }

    fn out_signals(&self) -> BusSignalsOut
    {
        BusSignalsOut {
            // hold daisy_out low when unconfigured
            daisy_out: self.configured,
        }
    }
}