use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct PvHeaderFlags: u8 {
        // bit 0
        const CONTROL_MSG       = 0b0000_0001; // 1 = control message, 0 = application message

        // bits 1–3 reserved (you can ignore them for now)

        // bits 4–5: segmentation
        const SEGMENT_NONE      = 0b0000_0000; // 00
        const SEGMENT_FIRST     = 0b0001_0000; // 01 << 4
        const SEGMENT_LAST      = 0b0010_0000; // 10 << 4
        const SEGMENT_MIDDLE    = 0b0011_0000; // 11 << 4

        // bit 6: sender
        const FROM_CLIENT       = 0b0000_0000;
        const FROM_SERVER       = 0b0100_0000;

        // bit 7: endianness
        const LITTLE_ENDIAN     = 0b0000_0000;
        const BIG_ENDIAN        = 0b1000_0000;
    }
}

#[test]
pub fn test_pv_header_flags() {
    // Example usage
    let flags =
        PvHeaderFlags::FROM_SERVER | PvHeaderFlags::BIG_ENDIAN | PvHeaderFlags::SEGMENT_NONE;
    assert!(flags.contains(PvHeaderFlags::FROM_SERVER));
    assert!(flags.contains(PvHeaderFlags::BIG_ENDIAN));
    assert!(!flags.contains(PvHeaderFlags::CONTROL_MSG));
}
