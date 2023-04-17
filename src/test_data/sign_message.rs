use crate::{message::CasperMessage, sample::Sample};

const SAMPLE_MESSAGE: &str = "Please sign this CSPR token donation";

/// Returns sample with valid CasperMessage for signing.
pub(crate) fn valid_casper_message_sample() -> Vec<Sample<CasperMessage>> {
    vec![Sample::new(
        "valid-casper-message",
        CasperMessage::new(SAMPLE_MESSAGE.as_bytes().to_vec()),
        true,
    )]
}

/// Returns invalid sample of CasperMessage for signing.
pub(crate) fn invalid_casper_message_sample() -> Vec<Sample<CasperMessage>> {
    let invalid_header = vec![
        "Casper:",
        "CasperMessage:",
        "Casper:\n",
        "casper message:\n",
        "Casper message:\n",
    ]
    .into_iter()
    .map(|prefix| prefix.as_bytes().to_vec());

    let msg = SAMPLE_MESSAGE.as_bytes();

    invalid_header
        .map(|prefix| {
            let mut output: Vec<u8> = prefix;
            output.extend(msg.clone());
            let message = CasperMessage::raw(output);
            Sample::new("invalid-casper-message", message, false)
        })
        .collect()
}
