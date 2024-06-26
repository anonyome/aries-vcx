use did_doc::schema::did_doc::DidDocument;

use crate::{
    error::DidPeerError,
    peer_did::{
        numalgos::{numalgo2::Numalgo2, Numalgo},
        FromDidDoc, PeerDid,
    },
};

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub struct Numalgo3;

impl Numalgo for Numalgo3 {
    const NUMALGO_CHAR: char = '3';
}

impl FromDidDoc for Numalgo3 {
    fn from_did_doc(did_document: DidDocument) -> Result<PeerDid<Numalgo3>, DidPeerError> {
        PeerDid::<Numalgo2>::from_did_doc(did_document)?.to_numalgo3()
    }
}

#[cfg(test)]
mod tests {
    use crate::peer_did::{
        numalgos::{numalgo2::Numalgo2, numalgo3::Numalgo3},
        PeerDid,
    };

    #[test]
    fn test_generate_numalgo3() {
        // from spec: https://identity.foundation/peer-did-method-spec/#method-3-did-shortening-with-sha-256-hash
        let full_did2_str = "did:peer:2.Ez6LSbysY2xFMRpGMhb7tFTLMpeuPRaqaWM1yECx2AtzE3KCc.Vz6MkqRYqQiSgvZQdnBytw86Qbs2ZWUkGv22od935YF4s8M7V.Vz6MkgoLTnTypo3tDRwCkZXSccTPHRLhF4ZnjhueYAFpEX6vg.SeyJ0IjoiZG0iLCJzIjoiaHR0cHM6Ly9leGFtcGxlLmNvbS9lbmRwb2ludCIsInIiOlsiZGlkOmV4YW1wbGU6c29tZW1lZGlhdG9yI3NvbWVrZXkiXSwiYSI6WyJkaWRjb21tL3YyIiwiZGlkY29tbS9haXAyO2Vudj1yZmM1ODciXX0";
        let peer_did_2 = PeerDid::<Numalgo2>::parse(full_did2_str).unwrap();
        assert_eq!(
            PeerDid::<Numalgo3>::parse(
                "did:peer:3zQmS19jtYDvGtKVrJhQnRFpBQAx3pJ9omx2HpNrcXFuRCz9".to_string()
            )
            .unwrap(),
            peer_did_2.to_numalgo3().unwrap()
        );
    }
}
