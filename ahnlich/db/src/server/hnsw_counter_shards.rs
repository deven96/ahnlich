use clap::ValueEnum;

/// User-selected HNSW counter allocation level; not an adaptive traffic policy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum HnswCounterShardLevel {
    #[default]
    Minimal,
    Low,
    Medium,
    High,
    Maximum,
}

impl HnswCounterShardLevel {
    /// Resolve on the host owning the HNSW graph, capped like Papaya's default.
    pub fn resolve(self) -> std::num::NonZeroUsize {
        let cpus =
            std::thread::available_parallelism().unwrap_or(std::num::NonZeroUsize::new(1).unwrap());
        self.resolve_for_cpus(cpus)
    }

    fn resolve_for_cpus(self, cpus: std::num::NonZeroUsize) -> std::num::NonZeroUsize {
        let maximum = cpus.get().next_power_of_two();
        let requested = match self {
            Self::Minimal => 1,
            Self::Low => 2,
            Self::Medium => 4,
            Self::High => 8,
            Self::Maximum => maximum,
        };
        std::num::NonZeroUsize::new(requested.min(maximum)).unwrap()
    }
}

#[cfg(test)]
mod counter_shard_level_tests {
    use super::*;
    use crate::cli::{Cli, Commands};
    use clap::Parser;
    use std::num::NonZeroUsize;

    #[test]
    fn cli_accepts_levels_and_rejects_invalid_values() {
        for (name, expected) in [
            ("minimal", HnswCounterShardLevel::Minimal),
            ("low", HnswCounterShardLevel::Low),
            ("medium", HnswCounterShardLevel::Medium),
            ("high", HnswCounterShardLevel::High),
            ("maximum", HnswCounterShardLevel::Maximum),
        ] {
            let cli =
                Cli::try_parse_from(["ahnlich-db", "run", "--hnsw-counter-shard-level", name])
                    .unwrap();
            let Commands::Run(config) = cli.command;
            assert_eq!(config.hnsw_counter_shard_level, expected);
        }
        assert!(
            Cli::try_parse_from(["ahnlich-db", "run", "--hnsw-counter-shard-level", "unknown"])
                .is_err()
        );
    }

    #[test]
    fn levels_resolve_to_power_of_two_counts_capped_by_cpus() {
        let levels = [
            HnswCounterShardLevel::Minimal,
            HnswCounterShardLevel::Low,
            HnswCounterShardLevel::Medium,
            HnswCounterShardLevel::High,
            HnswCounterShardLevel::Maximum,
        ];
        for (cpus, expected) in [
            (1, [1, 1, 1, 1, 1]),
            (2, [1, 2, 2, 2, 2]),
            (3, [1, 2, 4, 4, 4]),
            (10, [1, 2, 4, 8, 16]),
            (16, [1, 2, 4, 8, 16]),
        ] {
            for (level, count) in levels.into_iter().zip(expected) {
                assert_eq!(
                    level
                        .resolve_for_cpus(NonZeroUsize::new(cpus).unwrap())
                        .get(),
                    count
                );
            }
        }
        assert_eq!(
            HnswCounterShardLevel::default(),
            HnswCounterShardLevel::Minimal
        );
    }
}
