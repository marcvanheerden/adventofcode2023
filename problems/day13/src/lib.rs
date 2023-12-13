use std::iter::{Rev, Zip};
use std::ops::Range;
use std::str::FromStr;
use tokio::sync::mpsc::Receiver;

const BLEED: usize = 1;

#[derive(Debug)]
struct Mirror {
    values: Vec<bool>,
    row_len: usize,
    col_len: usize,
    flip: Option<usize>,
}

impl FromStr for Mirror {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines: Vec<&str> = s
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();
        let row_len = lines[0].len();
        let col_len = lines.len();

        let values = lines
            .into_iter()
            .flat_map(|l| l.chars().collect::<Vec<char>>())
            .filter(|c| ['.', '#'].contains(c))
            .map(|c| c == '#')
            .collect();

        Ok(Mirror {
            values,
            row_len,
            col_len,
            flip: None,
        })
    }
}

impl Mirror {
    fn _get(&self, x: usize, y: usize) -> Option<bool> {
        if (x >= self.row_len) | (y >= self.col_len) {
            return None;
        }

        let index1d = x + y * self.row_len;

        if let Some(flipped_bit) = self.flip {
            if index1d == flipped_bit {
                return Some(!self.values[index1d]);
            }
        }

        Some(self.values[index1d])
    }

    fn _mirror_zip(&self, split: usize, horizontal: bool) -> Zip<Rev<Range<usize>>, Range<usize>> {
        // get the pairs of indices that should be equal in a reflection
        let limit = if horizontal {
            self.col_len
        } else {
            self.row_len
        };
        (0..split).rev().zip(split..limit)
    }

    fn vertical_reflection(&self, ignore: usize) -> Option<usize> {
        'a: for split in BLEED..=(self.row_len - BLEED) {
            if split == ignore {
                continue;
            }
            for compare_pair in self._mirror_zip(split, false) {
                for idx in 0..self.col_len {
                    if self._get(compare_pair.0, idx) != self._get(compare_pair.1, idx) {
                        continue 'a;
                    }
                }
            }
            return Some(split);
        }

        None
    }

    fn horizontal_reflection(&self, ignore: usize) -> Option<usize> {
        'a: for split in BLEED..=(self.col_len - BLEED) {
            if split == ignore {
                continue;
            }
            for compare_pair in self._mirror_zip(split, true) {
                for idx in 0..self.row_len {
                    if self._get(idx, compare_pair.0) != self._get(idx, compare_pair.1) {
                        continue 'a;
                    }
                }
            }
            return Some(split);
        }

        None
    }
    fn summarize(&self, ignore_vert: usize, ignore_hori: usize) -> (usize, usize) {
        let mut vert_score = 0;
        let mut hori_score = 0;

        if let Some(split) = self.vertical_reflection(ignore_vert) {
            vert_score = split;
        }
        if let Some(split) = self.horizontal_reflection(ignore_hori) {
            hori_score = split;
        }

        (vert_score, hori_score)
    }
}

pub async fn solve(mut rx: Receiver<String>) {
    let mut tasks = Vec::new();

    while let Some(mirror) = rx.recv().await {
        let task = tokio::spawn(async move {
            let mut mirr = Mirror::from_str(&mirror).unwrap();
            let ref_score = mirr.summarize(0, 0);

            for bit in 0..(mirr.row_len * mirr.col_len) {
                mirr.flip = Some(bit);
                let score = mirr.summarize(ref_score.0, ref_score.1);

                if (score.0 > 0) & (score.0 != ref_score.0) {
                    return (ref_score.0 + ref_score.1 * 100, score.0);
                }
                if (score.1 > 0) & (score.1 != ref_score.1) {
                    return (ref_score.0 + ref_score.1 * 100, score.1 * 100);
                }
            }
            dbg!("shouldn't get here");
            (
                ref_score.0 + ref_score.1 * 100,
                ref_score.0 + ref_score.1 * 100,
            )
        });
        tasks.push(task);
    }

    let mut total_score1 = 0;
    let mut total_score2 = 0;

    for task in tasks {
        if let Ok((score1, score2)) = task.await {
            total_score1 += score1;
            total_score2 += score2;
        }
    }

    println!("Part 1: {total_score1} Part 2: {total_score2}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vert() {
        let test_input = "#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.
";
        let mirr = Mirror::from_str(test_input).unwrap();
        assert_eq!(mirr.vertical_reflection(0), Some(5));
        assert_eq!(mirr.horizontal_reflection(0), None);
    }

    #[test]
    fn horiz() {
        let test_input = "#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#
";
        let mirr = Mirror::from_str(test_input).unwrap();
        assert_eq!(mirr.vertical_reflection(0), None);
        assert_eq!(mirr.horizontal_reflection(0), Some(4));
    }

    #[test]
    fn special() {
        let test_input = ".####..#..#..
.####........
#.##.#..##..#
#######..#.##
##..##......#
######.####.#
.####..####..";
        let mirr = Mirror::from_str(test_input).unwrap();
        assert_eq!(mirr.vertical_reflection(0), Some(3));
        assert_eq!(mirr.horizontal_reflection(0), None);
    }
}
