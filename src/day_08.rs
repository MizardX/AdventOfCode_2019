#[aoc(day8, part1)]
fn part_1(input: &[u8]) -> usize {
    const LAYER_SIZE: usize = 25 * 6;
    let [_, one, two] = input
        .chunks_exact(LAYER_SIZE)
        .map(get_pixel_count)
        .min()
        .unwrap();
    one * two
}

fn get_pixel_count(layer: &[u8]) -> [usize; 3] {
    let mut count = [0; 3];
    for &digit in layer {
        count[(digit - b'0') as usize] += 1;
    }
    count
}

#[aoc(day8, part2)]
fn part_2(input: &[u8]) -> String {
    const WIDTH: usize = 25;
    const HEIGHT: usize = 6;
    let image = flatten_layers(input, WIDTH, HEIGHT);
    render_image(&image, WIDTH, HEIGHT)
}

fn flatten_layers(input: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut image = vec![b'2'; width * height];
    for layer in input.chunks_exact(width * height) {
        for (r, row) in layer.chunks_exact(width).enumerate() {
            for (c, &layer_pixel) in row.iter().enumerate() {
                let image_pixel = &mut image[r * width + c];
                if *image_pixel == b'2' {
                    *image_pixel = layer_pixel;
                }
            }
        }
    }
    image
}

fn render_image(image: &[u8], width: usize, height: usize) -> String {
    let mut rendered = String::with_capacity((width * '█'.len_utf8() + 1) * height / 2);
    for (row1, row2) in image
        .chunks_exact(width)
        .zip(image.chunks_exact(width).skip(1))
        .step_by(2)
    {
        rendered.push('\n');
        for (&px1, &px2) in row1.iter().zip(row2) {
            rendered.push(match (px1, px2) {
                (b'1', b'1') => '█',
                (b'1', _) => '▀',
                (_, b'1') => '▄',
                _ => ' ',
            });
        }
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_layers() {
        let input = b"0222112222120000";
        let result = flatten_layers(input, 2, 2);
        assert_eq!(result, b"0110");
    }

    #[test]
    fn test_render_image() {
        let image = b"0110";
        let result = render_image(image, 2, 2);
        assert_eq!(result, "\n▄▀"); // including linebreak at the start
    }
}
