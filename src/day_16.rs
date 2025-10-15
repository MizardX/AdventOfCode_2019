#[aoc(day16, part1)]
fn part_1(signal: &[u8]) -> String {
    let mut signal = signal.to_vec();
    flawed_frequency_transmission(&mut signal, 0, 100);
    str::from_utf8(&signal[..8]).unwrap().to_string()
}

#[aoc(day16, part2)]
fn part_2(signal: &[u8]) -> String {
    let n = signal.len();
    let start: usize = str::from_utf8(&signal[..7]).unwrap().parse().unwrap();
    let end = n * 10_000;

    let mut real_signal = Vec::with_capacity(end - start);
    real_signal.extend_from_slice(&signal[start % n..]);
    for _ in (start.div_ceil(n) * n..end).step_by(n) {
        real_signal.extend_from_slice(signal);
    }
    assert_eq!(real_signal.len(), end - start);
    
    flawed_frequency_transmission2(&mut real_signal, 100);

    str::from_utf8(&real_signal[..8]).unwrap().to_string()
}

fn flawed_frequency_transmission(signal: &mut [u8], offset: usize, times: usize) {
    for _ in 0..times {
        run_phase(signal, offset);
    }
}

fn run_phase(signal: &mut [u8], offset: usize) {
    for output_ix in 0..signal.len() {
        let sum = signal
            .iter()
            .enumerate()
            .map(|(pattern_ix, &ch)| {
                (ch - b'0').cast_signed() * get_pattern(offset + output_ix, offset + pattern_ix)
            })
            .map(i32::from)
            .sum::<i32>();
        signal[output_ix] = (sum.unsigned_abs() % 10) as u8 + b'0';
    }
}

fn flawed_frequency_transmission2(signal: &mut [u8], times: usize) {
    for _ in 0..times {
        run_phase2(signal);
    }
}

fn run_phase2(signal: &mut [u8]) {
    let mut sum: i64 = signal.iter().map(|&x| i64::from(x - b'0')).sum();
    for value in signal {
        let t = sum;
        sum -= i64::from(*value - b'0');
        *value = (t.unsigned_abs() % 10) as u8 + b'0';
    }
}

const fn get_pattern(out_position: usize, pattern_position: usize) -> i8 {
    if pattern_position < out_position {
        0
    } else {
        [1, 0, -1, 0][((pattern_position - out_position) / (out_position + 1)) % 4]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(0 => [1, 0, -1, 0, 1, 0, -1, 0]; "Out position 0 -> Normal pattern")]
    #[test_case(1 => [0, 1, 1, 0, 0, -1, -1, 0]; "Out position 1 -> Slower pattern")]
    #[test_case(2 => [0, 0, 1, 1, 1, 0, 0, 0]; "Out position 2 -> Even slower pattern")]
    fn test_pattern<const N: usize>(out_position: usize) -> [i8; N] {
        (0..N)
            .map(|pat| get_pattern(out_position, pat))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    #[test_case(*b"12345678" => *b"48226158")]
    #[test_case(*b"48226158" => *b"34040438")]
    #[test_case(*b"34040438" => *b"03415518")]
    fn test_run_phase<const N: usize>(mut input: [u8; N]) -> [u8; N] {
        run_phase(&mut input, 0);
        input
    }

    // Second half will be correct using run_phase2
    #[test_case(*b"12345678" => *b"6158")]
    #[test_case(*b"48226158" => *b"0438")]
    #[test_case(*b"34040438" => *b"5518")]
    fn test_run_phase2<const N: usize, const N2: usize>(mut input: [u8; N]) -> [u8; N2] {
        run_phase2(&mut input);
        input[N-N2..N].try_into().unwrap()
    }

    #[test_case(b"80871224585914546619083218645595" => "24176176")]
    #[test_case(b"19617804207202209144916044189917" => "73745418")]
    #[test_case(b"69317163492948606335995924319873" => "52432133")]
    fn test_part_1(signal: &[u8]) -> String {
        let mut signal = signal.to_vec();
        flawed_frequency_transmission(&mut signal, 0, 100);
        str::from_utf8(&signal[..8]).unwrap().to_string()
    }

    #[test_case(b"03036732577212944063491565474664" => "84462026")]
    #[test_case(b"02935109699940807407585447034323" => "78725270")]
    #[test_case(b"03081770884921959731165446850517" => "53553731")]
    fn test_part_2(signal: &[u8]) -> String {
        part_2(signal)
    }
}
