pub(crate) const DIM: usize = 768;

#[derive(Debug)]
pub(crate) struct Int8CosineVector {
    code: [i8; DIM],
    inverse_norm: f32,
}

impl Int8CosineVector {
    pub(crate) fn encode(vector: &[f32; DIM]) -> Option<Self> {
        let mut max = 0.0_f32;
        for &value in vector {
            if !value.is_finite() {
                return None;
            }
            max = max.max(value.abs());
        }
        if max == 0.0 {
            return None;
        }
        let scale = 127.0 / max;
        if !scale.is_finite() {
            return None;
        }
        let code = vector.map(|value| (value * scale).round().clamp(-127.0, 127.0) as i8);
        let norm_squared: i32 = code.iter().map(|&q| i32::from(q) * i32::from(q)).sum();
        Some(Self {
            code,
            inverse_norm: (norm_squared as f32).sqrt().recip(),
        })
    }
}

fn scalar_dot<const N: usize>(query: &[i8; DIM], neighbors: [&[i8; DIM]; N]) -> [i32; N] {
    neighbors.map(|neighbor| {
        query
            .iter()
            .zip(neighbor)
            .map(|(&a, &b)| i32::from(a) * i32::from(b))
            .sum()
    })
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x86 {
    use super::DIM;
    use pulp::{cast, x86::V3};
    use std::sync::OnceLock;

    static V3_SUPPORT: OnceLock<Option<V3>> = OnceLock::new();

    pub(super) fn capabilities() -> Option<V3> {
        *V3_SUPPORT.get_or_init(V3::try_new)
    }

    pub(super) fn dot<const N: usize>(
        simd: V3,
        query: &[i8; DIM],
        neighbors: [&[i8; DIM]; N],
    ) -> [i32; N] {
        simd.vectorize(|| {
            let zero = simd.avx._mm256_setzero_si256();
            let mut sums = [zero; N];
            let blocks = neighbors.map(|vector| vector.as_chunks::<16>().0);
            for (index, query_block) in query.as_chunks::<16>().0.iter().enumerate() {
                let a = simd.avx2._mm256_cvtepi8_epi16(cast(*query_block));
                for (sum, vector) in sums.iter_mut().zip(blocks) {
                    let b = simd.avx2._mm256_cvtepi8_epi16(cast(vector[index]));
                    // Multiply signed lanes in pairs and accumulate into i32.
                    let pair_products = simd.avx2._mm256_madd_epi16(a, b);
                    *sum = simd.avx2._mm256_add_epi32(*sum, pair_products);
                }
            }
            sums.map(|sum| cast::<_, [i32; 8]>(sum).into_iter().sum())
        })
    }
}

pub(crate) fn has_avx2_kernel() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        x86::capabilities().is_some()
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}

pub(crate) fn score_int8<const N: usize>(
    query: &Int8CosineVector,
    neighbors: [&Int8CosineVector; N],
) -> [super::Similarity; N] {
    let codes = neighbors.map(|neighbor| &neighbor.code);
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    let dots = x86::capabilities().map_or_else(
        || scalar_dot(&query.code, codes),
        |simd| x86::dot(simd, &query.code, codes),
    );
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    let dots = scalar_dot(&query.code, codes);
    std::array::from_fn(|i| {
        super::Similarity(dots[i] as f32 * query.inverse_norm * neighbors[i].inverse_norm)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    const WIDTH: usize = 4;

    fn score_int8_x4(
        query: &Int8CosineVector,
        neighbors: [&Int8CosineVector; WIDTH],
    ) -> [f32; WIDTH] {
        score_int8(query, neighbors).map(crate::Similarity::value)
    }

    #[test]
    fn int8_scorer_matches_integer_reference() {
        let query =
            Int8CosineVector::encode(&std::array::from_fn(|i| (i % 251) as f32 - 125.0)).unwrap();
        let neighbors: [_; WIDTH] = std::array::from_fn(|j| {
            let vector = std::array::from_fn(|i| ((i * (j + 2)) % 233) as f32 - 116.0);
            Int8CosineVector::encode(&vector).unwrap()
        });
        let refs = neighbors.each_ref();
        let expected = scalar_dot(&query.code, refs.map(|neighbor| &neighbor.code));
        let actual = score_int8_x4(&query, refs);
        for i in 0..WIDTH {
            assert_eq!(
                actual[i].to_bits(),
                (expected[i] as f32 * query.inverse_norm * refs[i].inverse_norm).to_bits(),
            );
        }
    }

    #[test]
    fn int8_scorer_handles_opposite_vectors_and_invalid_input() {
        let mut vector = [0.0; DIM];
        vector[0] = 1.0;
        vector[DIM - 1] = -1.0;
        let positive = Int8CosineVector::encode(&vector).unwrap();
        let negative = Int8CosineVector::encode(&vector.map(|v| -v)).unwrap();
        let values = score_int8_x4(&positive, [&positive, &negative, &positive, &negative]);
        for (actual, expected) in values.into_iter().zip([1.0_f32, -1.0, 1.0, -1.0]) {
            assert!((expected - actual).abs() < 1e-6);
        }
        assert!(Int8CosineVector::encode(&[0.0; DIM]).is_none());
        vector[0] = f32::NAN;
        assert!(Int8CosineVector::encode(&vector).is_none());
    }
}
