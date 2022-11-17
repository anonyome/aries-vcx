use vdrtools::anoncreds;
use crate::error::{VcxError, VcxResult};

pub async fn libindy_verifier_verify_proof(
    proof_req_json: &str,
    proof_json: &str,
    schemas_json: &str,
    credential_defs_json: &str,
    rev_reg_defs_json: &str,
    rev_regs_json: &str,
) -> VcxResult<bool> {
    anoncreds::verifier_verify_proof(
        proof_req_json,
        proof_json,
        schemas_json,
        credential_defs_json,
        rev_reg_defs_json,
        rev_regs_json,
    )
        .await
        .map_err(VcxError::from)
}