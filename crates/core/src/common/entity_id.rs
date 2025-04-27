use super::checkpoint::CheckpointNumberOrTag;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum EntityIdError {
    #[error("Invalid address")]
    InvalidAddress,
    #[error("Invalid tx hash")]
    InvalidTxHash,
    #[error("Invalid block number or tag: {0}")]
    InvalidCheckpointNumberOrTag(String),
    #[error("Unable resolve ENS name")]
    EnsResolution,
}

pub fn parse_checkpoint_number_or_tag(id: &str) -> Result<CheckpointNumberOrTag, EntityIdError> {
    match id.trim().parse::<u64>() {
        Ok(id) => Ok(CheckpointNumberOrTag::Number(id)),
        Err(_) => id
            .parse::<CheckpointNumberOrTag>()
            .map_err(|_| EntityIdError::InvalidCheckpointNumberOrTag(id.to_string())),
    }
}
