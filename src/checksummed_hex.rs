use casper_types::blake2b;

pub fn encode<T: AsRef<[u8]>>(input: T) -> String {
    if input.as_ref().len() > SMALL_BYTES_COUNT {
        return base16::encode_lower(&input);
    }
    encode_iter(&input).collect()
}

/// The number of input bytes, at or below which [`decode`] will checksum-decode the output.
pub const SMALL_BYTES_COUNT: usize = 75;

const HEX_CHARS: [char; 22] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'A', 'B', 'C',
    'D', 'E', 'F',
];

/// Takes a slice of bytes and breaks it up into a vector of *nibbles* (ie, 4-bit values)
/// represented as `u8`s.
fn bytes_to_nibbles<'a, T: 'a + AsRef<[u8]>>(input: &'a T) -> impl Iterator<Item = u8> + 'a {
    input
        .as_ref()
        .iter()
        .flat_map(move |byte| [4, 0].iter().map(move |offset| (byte >> offset) & 0x0f))
}

/// Takes a slice of bytes and outputs an infinite cyclic stream of bits for those bytes.
fn bytes_to_bits_cycle(bytes: Vec<u8>) -> impl Iterator<Item = bool> {
    bytes
        .into_iter()
        .cycle()
        .flat_map(move |byte| (0..8usize).map(move |offset| ((byte >> offset) & 0x01) == 0x01))
}

/// Returns the bytes encoded as hexadecimal with mixed-case based checksums following a scheme
/// similar to [EIP-55](https://eips.ethereum.org/EIPS/eip-55).
///
/// Key differences:
///   - Works on any length of data, not just 20-byte addresses
///   - Uses Blake2b hashes rather than Keccak
///   - Uses hash bits rather than nibbles
fn encode_iter<'a, T: 'a + AsRef<[u8]>>(input: &'a T) -> impl Iterator<Item = char> + 'a {
    let nibbles = bytes_to_nibbles(input);
    let mut hash_bits = bytes_to_bits_cycle(blake2b(input.as_ref()).to_vec());
    nibbles.map(move |mut nibble| {
        // Base 16 numbers greater than 10 are represented by the ascii characters a through f.
        if nibble >= 10 && hash_bits.next().unwrap_or(true) {
            // We are using nibble to index HEX_CHARS, so adding 6 to nibble gives us the index
            // of the uppercase character. HEX_CHARS[10] == 'a', HEX_CHARS[16] == 'A'.
            nibble += 6;
        }
        HEX_CHARS[nibble as usize]
    })
}
