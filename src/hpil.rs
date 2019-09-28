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
/// See `PhyBitDecoder` for more details.
#[derive(Debug,Clone,PartialEq,Eq,Default)]
pub struct PollPhy {
    bit_decode: PhyBitDecoder,

    // next bit to be filled in
    message_bit_offs: u8,
    // accumulated message bits
    // note: when >0, we have recieved a sync bit
    message_bits: u16,

    // when we identify the message_bits as having a _prefix_ that indicates the
    // need to be retransmitted, we should begin retransmission.
    // HP-IL spec refers to this as "echo" vs "hold".
}

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
#[derive(Debug,Clone,PartialEq,Eq,Default)]
pub struct PhyBitDecoder {
    // packed into 2 bit representations,
    // `32 / 2 = 16` samples possible
    //
    // theoretically allows sampling at `(16/6) = 2 2/3` times the actual edge
    // rate.
    //
    // lower bits are older, higher bits are newer
    // filled in high bits first:
    //      |xxxxxxxxxxxxxxxx|
    //      |Axxxxxxxxxxxxxxx| (pushed sample A)
    //      |BAxxxxxxxxxxxxxx| (pushed sample B)
    //      ...
    //      |PONMLKJIHGFEDCBA|
    //      |QPONMLKJIHGFEDCB| (pushed sample Q, dropped A)
    packed_samples: u32,

    // bit (sample / 2) offset to be filled in next. all samples before this
    // are valid and can be examined.
    packed_sample_offs: u8,
}

impl PhyBitDecoder {
    pub fn samples(&self) -> PhySampleIter {
        PhySampleIter {
            p: self,
            sample_offs: 0,
        }
    }

    /// push a new sample into the Phy
    pub fn push(&mut self, sample: PhySample) {
        assert!((self.packed_sample_offs & 1) == 0);

        if self.packed_sample_offs == 32 {
            // we essentially cap at 32 bits. old data gets shifted off below
            self.packed_sample_offs -= 2;
        }

        assert!(self.packed_sample_offs <= 30);

        // XXX: consider if avoiding a constant rotation might make sense
        self.packed_samples = self.packed_samples.wrapping_shr(2);

        // NOTE: 32 here is the number of bits in `packed_samples`, and `2` is the bits-per-sample
        self.packed_samples |= (sample.as_bits() as u32) << (32 - 2);
        self.packed_sample_offs += 2;
    }
}

/// Iterate over samples recieved from oldest to newest
pub struct PhySampleIter<'a> {
    p: &'a PhyBitDecoder,
    sample_offs: u8,
}

impl<'a> Iterator for PhySampleIter<'a> {
    type Item = PhySample;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sample_offs == self.p.packed_sample_offs {
            None
        } else {
            let shift = 32 - self.sample_offs - 2;
            let mask = 0b11 << shift;
            self.sample_offs += 2;
            Some(PhySample::from_bits(((self.p.packed_samples & mask) >> shift) as u8).unwrap())
        }
    }
}

#[test]
fn test_sample_iter() {
    let mut phy = PhyBitDecoder::default();

    let samples = [
        PhySample::Neg,
        PhySample::Pos,
        PhySample::Zero,
    ];

    for &s in samples.iter().rev() {
        phy.push(s);
    }

    let rs: Vec<PhySample> = phy.samples().collect();

    assert_eq!(&samples[..], &rs[..]);
}

impl PollPhy {
    pub fn check_seq(&mut self) -> bool {
        unimplemented!()
    }

}

#[derive(Debug,Copy,Clone,PartialEq,Eq)]
pub enum PhySample {
    Zero,
    Pos,
    Neg,
}

impl PhySample {
    // Note: 0 is avoided so it can be used in packed samples to represent the lack of a sample
    fn as_bits(self) -> u8 {
        match self {
            Self::Zero => 0b11,
            Self::Pos => 0b01,
            Self::Neg => 0b10,
        }
    }

    fn from_bits(b: u8) -> Option<Self> {
        match b {
            0b11 => Some(Self::Zero),
            0b01 => Some(Self::Pos),
            0b10 => Some(Self::Neg),
            _ => None,
        }
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

    /// "SOT"
    /// "IFC"
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