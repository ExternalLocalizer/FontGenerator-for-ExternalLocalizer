use std::fmt::Display;

use font::characters::Character;

pub trait Counter {
    fn count(&self) -> usize;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharRange {
    start: u32,
    end: u32,
}

impl CharRange {
    // 範囲が隣接している、または重複しているかを判定
    fn merges_with(&self, other: &CharRange) -> bool {
        self.end + 1 >= other.start && self.start <= other.end + 1
    }

    // 2つの範囲をマージ
    fn merge(&self, other: &CharRange) -> CharRange {
        CharRange {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    // 範囲の引き算: self から other を引く
    fn subtract(&self, other: &CharRange) -> Vec<CharRange> {
        // 引き算の結果として得られる新しい範囲を格納するベクター
        let mut result = Vec::new();

        // 範囲が完全に重なっている場合は何も残らない
        if self.start >= other.end + 1 || self.end as i64 <= other.start as i64 - 1 {
            result.push(*self); // 重なっていなければ元の範囲をそのまま残す
        } else {
            // 左部分の残り範囲
            if self.start < other.start {
                result.push(CharRange {
                    start: self.start,
                    end: other.start - 1,
                });
            }
            // 右部分の残り範囲
            if self.end > other.end {
                result.push(CharRange {
                    start: other.end + 1,
                    end: self.end,
                });
            }
        }

        result
    }
}

impl Counter for CharRange {
    fn count(&self) -> usize {
        (self.end - self.start + 1) as usize
    }
}

impl AsRef<CharRange> for CharRange {
    fn as_ref(&self) -> &CharRange {
        self
    }
}

impl From<Character> for CharRange {
    fn from(c: Character) -> Self {
        match c {
            Character::Scalar(c) => CharRange {
                start: c as u32,
                end: c as u32,
            },
            Character::Range((start, end)) => CharRange {
                start: start as u32,
                end: end as u32,
            },
        }
    }
}

impl Display for CharRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.start == self.end {
            write!(
                f,
                "{}(U+{:04X})",
                char::from_u32(self.start).unwrap(),
                self.start
            )
        } else {
            write!(
                f,
                "{}(U+{:04X}) - {}(U+{:04X})",
                char::from_u32(self.start).unwrap(),
                self.start,
                char::from_u32(self.end).unwrap(),
                self.end
            )
        }
    }
}

#[derive(Debug)]
pub struct CharRangeList {
    ranges: Vec<CharRange>,
}

impl CharRangeList {
    // 新しいCharRangeListを作成
    pub fn new() -> Self {
        CharRangeList { ranges: Vec::new() }
    }

    // 範囲を適切な位置に挿入し、前後を確認してマージ
    pub fn add_range(&mut self, range: impl Into<CharRange>) {
        let range = range.into();
        // 挿入する位置を決める
        let pos = self.find_insert_position(&range);
        self.ranges.insert(pos, range);
        self.merge_around(pos);
    }

    // `binary_search_by_key` を使って挿入位置を決定
    fn find_insert_position(&self, range: &CharRange) -> usize {
        self.ranges
            .binary_search_by_key(&range.start, |r| r.start)
            .unwrap_or_else(|e| e) // 見つからなければその位置に挿入
    }

    // 挿入位置周辺の範囲をマージする
    fn merge_around(&mut self, pos: usize) {
        let mut index = pos;

        // 前方向にマージ
        if index > 0 && self.ranges[index].merges_with(&self.ranges[index - 1]) {
            let merged = self.ranges[index].merge(&self.ranges[index - 1]);
            self.ranges[index - 1] = merged;
            self.ranges.remove(index);
            index -= 1;
        }

        // 後方向にマージ
        while index < self.ranges.len() - 1
            && self.ranges[index].merges_with(&self.ranges[index + 1])
        {
            let merged = self.ranges[index].merge(&self.ranges[index + 1]);
            self.ranges[index] = merged;
            self.ranges.remove(index + 1);
        }
    }

    // 範囲の引き算
    pub fn subtract_range(&mut self, range: impl AsRef<CharRange>) {
        let mut new_ranges = Vec::new();

        // 各範囲から引き算を行い、新しい範囲を新たに作成
        for existing_range in self.ranges.iter() {
            new_ranges.extend(existing_range.subtract(range.as_ref()));
        }

        // 引き算後の新しい範囲でリストを更新
        self.ranges = new_ranges;
    }

    // 範囲の引き算
    pub fn subtract_range_list(&mut self, other: &CharRangeList) {
        for range in other.ranges.iter() {
            self.subtract_range(range);
        }
    }

    // pub fn iter(&self) -> std::slice::Iter<CharRange> {
    //     self.ranges.iter()
    // }
}

impl Counter for CharRangeList {
    fn count(&self) -> usize {
        self.ranges.iter().map(|r| r.count()).sum()
    }
}

impl<T> From<Vec<T>> for CharRangeList
where
    T: Into<CharRange>,
{
    fn from(ranges: Vec<T>) -> Self {
        let mut list = CharRangeList::new();
        for range in ranges {
            list.add_range(range);
        }
        list
    }
}

impl IntoIterator for CharRangeList {
    type Item = CharRange;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.ranges.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_single_range() {
        let mut range_list = CharRangeList::new();
        range_list.add_range(CharRange {
            start: 0x0020,
            end: 0x005A,
        });

        assert_eq!(range_list.ranges.len(), 1);
        assert_eq!(
            range_list.ranges[0],
            CharRange {
                start: 0x0020,
                end: 0x005A
            }
        );
    }

    #[test]
    fn test_add_multiple_ranges() {
        let mut range_list = CharRangeList::new();
        range_list.add_range(CharRange {
            start: 0x0020,
            end: 0x005A,
        });
        range_list.add_range(CharRange {
            start: 0x0060,
            end: 0x0080,
        });

        assert_eq!(range_list.ranges.len(), 2);
        assert_eq!(
            range_list.ranges[0],
            CharRange {
                start: 0x0020,
                end: 0x005A
            }
        );
        assert_eq!(
            range_list.ranges[1],
            CharRange {
                start: 0x0060,
                end: 0x0080
            }
        );
    }

    #[test]
    fn test_merge_adjacent_ranges() {
        let mut range_list = CharRangeList::new();
        range_list.add_range(CharRange {
            start: 0x0020,
            end: 0x005A,
        });
        range_list.add_range(CharRange {
            start: 0x005B,
            end: 0x0080,
        });

        assert_eq!(range_list.ranges.len(), 1);
        assert_eq!(
            range_list.ranges[0],
            CharRange {
                start: 0x0020,
                end: 0x0080
            }
        );
    }

    #[test]
    fn test_merge_non_adjacent_ranges() {
        let mut range_list = CharRangeList::new();
        range_list.add_range(CharRange {
            start: 0x0020,
            end: 0x005A,
        });
        range_list.add_range(CharRange {
            start: 0x0060,
            end: 0x0080,
        });

        assert_eq!(range_list.ranges.len(), 2);
        assert_eq!(
            range_list.ranges[0],
            CharRange {
                start: 0x0020,
                end: 0x005A
            }
        );
        assert_eq!(
            range_list.ranges[1],
            CharRange {
                start: 0x0060,
                end: 0x0080
            }
        );
    }

    #[test]
    fn test_merge_multiple_overlapping_ranges() {
        let mut range_list = CharRangeList::new();
        range_list.add_range(CharRange {
            start: 0x0020,
            end: 0x005A,
        });
        range_list.add_range(CharRange {
            start: 0x0055,
            end: 0x0080,
        });
        range_list.add_range(CharRange {
            start: 0x0070,
            end: 0x00A0,
        });

        // これらはすべて1つの範囲に統合されるべき
        assert_eq!(range_list.ranges.len(), 1);
        assert_eq!(
            range_list.ranges[0],
            CharRange {
                start: 0x0020,
                end: 0x00A0
            }
        );
    }

    #[test]
    fn test_merge_with_gap() {
        let mut range_list = CharRangeList::new();
        range_list.add_range(CharRange {
            start: 0x0020,
            end: 0x005A,
        });
        range_list.add_range(CharRange {
            start: 0x0060,
            end: 0x0080,
        });
        range_list.add_range(CharRange {
            start: 0x00A0,
            end: 0x00B0,
        });

        assert_eq!(range_list.ranges.len(), 3);
        assert_eq!(
            range_list.ranges[0],
            CharRange {
                start: 0x0020,
                end: 0x005A
            }
        );
        assert_eq!(
            range_list.ranges[1],
            CharRange {
                start: 0x0060,
                end: 0x0080
            }
        );
        assert_eq!(
            range_list.ranges[2],
            CharRange {
                start: 0x00A0,
                end: 0x00B0
            }
        );
    }

    #[test]
    fn test_add_range_with_large_gap() {
        let mut range_list = CharRangeList::new();
        range_list.add_range(CharRange {
            start: 0x0020,
            end: 0x005A,
        });
        range_list.add_range(CharRange {
            start: 0x0100,
            end: 0x0120,
        });

        assert_eq!(range_list.ranges.len(), 2);
        assert_eq!(
            range_list.ranges[0],
            CharRange {
                start: 0x0020,
                end: 0x005A
            }
        );
        assert_eq!(
            range_list.ranges[1],
            CharRange {
                start: 0x0100,
                end: 0x0120
            }
        );
    }

    #[test]
    fn test_merge_adjacent_range_after_insert() {
        let mut range_list = CharRangeList::new();
        range_list.add_range(CharRange {
            start: 0x0020,
            end: 0x005A,
        });
        range_list.add_range(CharRange {
            start: 0x0070,
            end: 0x0080,
        });
        range_list.add_range(CharRange {
            start: 0x005B,
            end: 0x006f,
        });

        println!("{:#?}", range_list.ranges);

        assert_eq!(range_list.ranges.len(), 1);
        assert_eq!(
            range_list.ranges[0],
            CharRange {
                start: 0x0020,
                end: 0x0080
            }
        );
    }

    // テストケース1: 完全一致
    #[test]
    fn test_subtract_full_range() {
        let mut char_range_list = CharRangeList::new();
        char_range_list.add_range(CharRange {
            start: 0x0000,
            end: 0x001F,
        });
        char_range_list.add_range(CharRange {
            start: 0x0020,
            end: 0x003F,
        });

        let subtract_range = CharRange {
            start: 0x0000,
            end: 0x001F,
        }; // 完全一致の範囲

        char_range_list.subtract_range(subtract_range);

        // 範囲 [0x0000-0x001F] が削除されるので、残るべき範囲は [0x0020-0x003F]
        assert_eq!(char_range_list.ranges.len(), 1);
        assert_eq!(
            char_range_list.ranges[0],
            CharRange {
                start: 0x0020,
                end: 0x003F
            }
        );
    }

    // テストケース2: 範囲の一部を削除 (前方)
    #[test]
    fn test_subtract_partial_overlap_start() {
        let mut char_range_list = CharRangeList::new();
        char_range_list.add_range(CharRange {
            start: 0x0000,
            end: 0x005F,
        });

        let subtract_range = CharRange {
            start: 0x0000,
            end: 0x001F,
        }; // 範囲の前半を削除

        char_range_list.subtract_range(subtract_range);

        // 範囲 [0x0000-0x001F] が削除され、残りは [0x0020-0x005F]
        assert_eq!(char_range_list.ranges.len(), 1);
        assert_eq!(
            char_range_list.ranges[0],
            CharRange {
                start: 0x0020,
                end: 0x005F
            }
        );
    }

    // テストケース3: 範囲の一部を削除 (後方)
    #[test]
    fn test_subtract_partial_overlap_end() {
        let mut char_range_list = CharRangeList::new();
        char_range_list.add_range(CharRange {
            start: 0x0000,
            end: 0x005F,
        });

        let subtract_range = CharRange {
            start: 0x0030,
            end: 0x005F,
        }; // 範囲の後半を削除

        char_range_list.subtract_range(subtract_range);

        // 範囲 [0x0030-0x005F] が削除され、残りは [0x0000-0x002F]
        assert_eq!(char_range_list.ranges.len(), 1);
        assert_eq!(
            char_range_list.ranges[0],
            CharRange {
                start: 0x0000,
                end: 0x002F
            }
        );
    }

    // テストケース4: 範囲が完全に重なる場合
    #[test]
    fn test_subtract_overlap_both_sides() {
        let mut char_range_list = CharRangeList::new();
        char_range_list.add_range(CharRange {
            start: 0x0000,
            end: 0x005F,
        });

        let subtract_range = CharRange {
            start: 0x0010,
            end: 0x003F,
        }; // 範囲 [0x0010-0x003F] が重なる

        char_range_list.subtract_range(subtract_range);

        // 範囲 [0x0010-0x003F] が削除されるので、残る範囲は [0x0000-0x000F] と [0x0040-0x005F]
        assert_eq!(char_range_list.ranges.len(), 2);
        assert_eq!(
            char_range_list.ranges[0],
            CharRange {
                start: 0x0000,
                end: 0x000F
            }
        );
        assert_eq!(
            char_range_list.ranges[1],
            CharRange {
                start: 0x0040,
                end: 0x005F
            }
        );
    }

    // テストケース5: 引き算される範囲が存在しない場合
    #[test]
    fn test_subtract_no_overlap() {
        let mut char_range_list = CharRangeList::new();
        char_range_list.add_range(CharRange {
            start: 0x0000,
            end: 0x001F,
        });

        let subtract_range = CharRange {
            start: 0x0020,
            end: 0x003F,
        }; // 重ならない範囲

        char_range_list.subtract_range(subtract_range);

        // 引き算範囲と重ならないので、リストはそのまま
        assert_eq!(char_range_list.ranges.len(), 1);
        assert_eq!(
            char_range_list.ranges[0],
            CharRange {
                start: 0x0000,
                end: 0x001F
            }
        );
    }

    // テストケース6: 範囲が完全に引き算される場合
    #[test]
    fn test_subtract_full_overlap() {
        let mut char_range_list = CharRangeList::new();
        char_range_list.add_range(CharRange {
            start: 0x0000,
            end: 0x005F,
        });

        let subtract_range = CharRange {
            start: 0x0000,
            end: 0x005F,
        }; // 完全に一致する範囲

        char_range_list.subtract_range(subtract_range);

        // 範囲が完全に削除されるので、リストは空になる
        assert_eq!(char_range_list.ranges.len(), 0);
    }
}
