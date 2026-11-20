//! Reset synchronization for digital circuits.

use super::circuit::{DigitalBuilder, NetId};
use kernel::SimError;

/// Builds an async-assert, sync-deassert reset synchronizer.
pub fn reset_synchronizer(
    builder: &mut DigitalBuilder,
    reset_n: NetId,
    stages: usize,
) -> Result<NetId, SimError> {
    if stages == 0 {
        return Err(SimError::InvalidNetlist(
            "reset synchronizer needs a stage".to_string(),
        ));
    }
    let mut state = reset_n;
    for _ in 0..stages {
        state = builder.dff(state);
    }
    Ok(builder.and(reset_n, state))
}

#[cfg(test)]
mod tests {
    use super::reset_synchronizer;
    use crate::DigitalBuilder;

    #[test]
    fn asserts_async_and_deasserts_after_stages() {
        let mut builder = DigitalBuilder::new();
        let reset = builder.input("reset_n");
        let synced = reset_synchronizer(&mut builder, reset, 2).expect("sync");
        builder.output("sync", synced);
        let mut circuit = builder.build();
        circuit.set_input("reset_n", false).expect("reset");
        circuit.tick().expect("tick");
        circuit.tick().expect("tick");
        assert_eq!(circuit.output("sync"), Some(false));
        circuit.set_input("reset_n", true).expect("release");
        circuit.tick().expect("tick");
        assert_eq!(circuit.output("sync"), Some(false));
        circuit.tick().expect("tick");
        assert_eq!(circuit.output("sync"), Some(true));
    }

    #[test]
    fn rejects_zero_stages() {
        let mut builder = DigitalBuilder::new();
        let reset = builder.input("reset_n");
        assert!(reset_synchronizer(&mut builder, reset, 0).is_err());
    }
}
