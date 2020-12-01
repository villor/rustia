use num_bigint::BigUint;

pub fn rsa_decrypt(data: &mut [u8]) {
    if data.len() != 128 {
        panic!("rsa_decrypt: input was not 128 bytes")
    }

    let d = BigUint::parse_bytes(b"428BD3B5346DAF71A761106F71A43102F8C857D6549C54660BB6378B52B0261399DE8CE648BAC410E2EA4E0A1CED1FAC2756331220CA6DB7AD7B5D440B7828865856E7AA6D8F45837FEEE9B4A3A0AA21322A1E2AB75B1825E786CF81A28A8A09A1E28519DB64FF9BAF311E850C2BFA1FB7B08A056CC337F7DF443761AEFE8D81", 16).unwrap();
    let n = BigUint::parse_bytes(b"009B646903B45B07AC956568D87353BD7165139DD7940703B03E6DD079399661B4A837AA60561D7CCB9452FA0080594909882AB5BCA58A1A1B35F8B1059B72B1212611C6152AD3DBB3CFBEE7ADC142A75D3D75971509C321C5C24A5BD51FD460F01B4E15BEB0DE1930528A5D3F15C1E3CBF5C401D6777E10ACAAB33DBE8D5B7FF5", 16).unwrap();
    let c = BigUint::from_bytes_be(data);
    let m = c.modpow(&d, &n);

    let m_bytes = m.to_bytes_be();
    for b in data[..128 - m_bytes.len()].iter_mut() {
        *b = 0;
    }
    data[128 - m_bytes.len()..128].clone_from_slice(m_bytes.as_slice());
}

// TODO: encrypt (for clients)