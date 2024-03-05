#[allow(unused)]
pub fn asm_mmio_rw_test() -> &'static [u8] {
    let asm_code: &[u8];
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        asm_code = &[
            0xba, 0xf8, 0x03, 0x00, 0x00, /* mov $0x3f8, %dx */
            0x00, 0xd8, /* add %bl, %al */
            0x04, b'0', /* add $'0', %al */
            0xee, /* out %al, %dx */
            0xec, /* in %dx, %al */
            0xba, 0x00, 0x00, 0x00, 0x80, /* mov edx, 0x80000000 */
            0x67, 0xc7, 0x02, 0x78, 0x56, 0x34, 0x12, /* mov DWORD PTR [edx], 0x12345678 */ // MmmiWrite
            0xf4, /* hlt */
        ];
    }
    if asm_code.len() == 0 {
        todo!("Architecture not implemented yet");
    }
    return asm_code;
}
