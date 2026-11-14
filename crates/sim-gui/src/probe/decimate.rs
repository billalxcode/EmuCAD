use super::*;

/// Decimates samples into min/max pairs, preserving extremes.
#[must_use]
pub fn decimate(samples: &VecDeque<ProbePoint>, max_points: usize) -> Vec<ProbePoint> {
    let bucket_size = samples.len().div_ceil(max_points.max(2));
    let mut output = Vec::new();
    let mut bucket = Vec::new();
    for point in samples {
        bucket.push(*point);
        if bucket.len() == bucket_size {
            push_min_max(&mut output, &bucket);
            bucket.clear();
        }
    }
    if !bucket.is_empty() {
        push_min_max(&mut output, &bucket);
    }
    output
}

fn push_min_max(output: &mut Vec<ProbePoint>, bucket: &[ProbePoint]) {
    let Some(min) = bucket.iter().min_by(|a, b| a.value.total_cmp(&b.value)) else {
        return;
    };
    let Some(max) = bucket.iter().max_by(|a, b| a.value.total_cmp(&b.value)) else {
        return;
    };
    let (first, second) = if min.cycle <= max.cycle {
        (min, max)
    } else {
        (max, min)
    };
    output.push(*first);
    if first.cycle != second.cycle {
        output.push(*second);
    }
}
