/// `PollPhy` manages decoding of sampled signals into `Message`s
/// 
/// It presumes we're polling the fabric approximately every 0.5 microseconds
/// (iow: twice the expected rate), but allows polling at exactly 1 microsecond,
/// or faster. It only requires that no edges are missed. Sampling faster will
/// require more internal storage.
/// 
/// It operates on a simplified 3 state input rather than analog voltages.
/// 
/// Electrical signals of HP-IL use 3-states: positive, zero, and negative. These
/// differentials are the measurement of voltage between the 2 HP-IL conductors.
/// 
/// -1.5V, 0V, and +1.5V are the levels.
/// 1 microsecond is used as the pulse width.
/// 
/// These 3 states, in combination with timing, are used to encode bits. The
/// first bit always has a special "sync" format.
/// 
/// The following is the bit decoding. `N` is a negative level, `P` is a positive
/// level, and `Z` is the zero level.
/// 
/// - 1: `PNZZ`
/// - 0: `NPZZ`
/// - 1 sync: `PNPNZZ`
/// - 0 sync: `NPNPZZ`
/// 
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct PollPhy {
    // packed into 2 bit representations,
    // `32 / 2 = 16` samples possible
    //
    // theoretically allows sampling at `(16/6) = 2 2/3` times the actual edge
    // rate.
    //
    // lower bits are older, higher bits are newer
    packed_samples: u32,

    // bit (sample / 2) offset to be filled in next. all samples before this
    // are valid and can be examined.
    packed_sample_offs: u8,

    // next bit to be filled in
    message_bit_offs: u8,
    // accumulated message bits
    // note: when >0, we have recieved a sync bit
    message_bits: u16,

    // when we identify the message_bits as having a _prefix_ that indicates the
    // need to be retransmitted, we should begin retransmission.
    // HP-IL spec refers to this as "echo" vs "hold".
}

struct PollPhySampleIter<'a> {
    p: &'a PollPhy,
    sample_offs: u8,
}

impl<'a> Iterator for PollPhySampleIter<'a> {
    type Item = PhySample;

    fn next(&mut self) -> Option<Self::Item> {
        
    }
}

impl PollPhy {
    pub fn samples(&self) -> PollPhySampleIter {
        unimplemented!()
    }

    pub fn check_seq(&mut self) -> bool {
        unimplemented!()
    }

    /// push a new sample into the Phy
    pub fn push(&mut self, sample: PhySample) -> bool {
        assert!((self.packed_sample_offs & 1) == 0);

        if (self.packed_sample_offs == 32) {
            // drop a sample. right now we rotate manually, and given we're using a u32, it should be pretty fast on most platforms.
            self.packed_samples.wrapping_shr(2);
            self.packed_sample_offs -= 2;
        }

        assert!(self.packed_sample_offs <= 30);

        self.packed_samples |= (sample.as_bits() as u32) << self.packed_sample_offs;
        self.packed_sample_offs += 2;
    }
}

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum PhySample {
    Zero,
    Pos,
    Neg,
}

impl PhySample {
    fn as_bits(self) -> u8 {

    }
}

#[derive(Debug,Clone,PartialEq,Eq)]
pub enum MessageClass {
    /// "DOE"
    DataOrEnd,

    /// "CMD"
    Command,

    /// "RDY"
    Ready,

    /// "IDY"
    Identify,
}

pub enum MessageType {
    /// "RFC"
    /// 
    /// `100_10010000`
    ReadyForCommand,

    /// Sent by a `controller`/`master`.
    /// Causes any previously active listener to become inactive
    /// 
    /// `100_00111111`
    Unlisten,

    /// `101_01100000`
    SendDataReady,    
}


/// 11-bits of on-bus data
pub struct Message {
    /// 1: sync
    /// 2: control
    /// 8: data
    raw: u16,
}

impl Message {
    /// Defines the major type/class of the message (`MessageClass`)
    /// 
    /// Includes the `sync` bit (0b100)
    pub fn control(&self) -> u8 {
        ((self.raw & (0b111 << 8)) >> 8) as u8
    }

    /// payload of a message, meaning determined by `major()`.
    /// remaining 8 bits
    pub fn data(&self) -> u8 {
        self.raw as u8
    }
}