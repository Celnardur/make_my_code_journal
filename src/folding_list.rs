use super::Error;
use crate::{Colors, LineCounts};
use std::{error, fmt::Write};

const DEBUG: bool = false;

pub struct FoldingList {
    list: Vec<Box<dyn Expand>>,
    expanded: Vec<Segment>,
    cursor: usize,
    segment: usize, // segment that that cursor is currently in
}

/// This structure stores the start and ends of a currently expanded segment.
#[derive(Clone, Debug)]
struct Segment {
    start: usize, // inclusive
    end: usize,   // exclusive
}

pub trait Expand {
    fn expand(&self) -> (Vec<Box<dyn Expand>>, bool) {
        (Vec::new(), false)
    }
    fn display(
        &self,
        _stream: &mut String,
        _colors: &Colors,
        _line: u16,
        _width: u16,
    ) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }
    fn highlight(
        &self,
        _stream: &mut String,
        _colors: &Colors,
        _line: u16,
        _width: u16,
    ) -> Result<(), Box<dyn error::Error>> {
        Ok(())
    }
    fn counts(&self) -> Option<LineCounts> {
        None
    }
    fn id(&self) -> usize {
        0
    } // mostly for testing
}

impl FoldingList {
    pub fn new(list: Vec<Box<dyn Expand>>) -> Result<FoldingList, Error> {
        if list.is_empty() {
            return Err(Error::new("Cannot initialize FoldingList with empty list"));
        }
        let list_len = list.len();
        Ok(FoldingList {
            list,
            expanded: vec![Segment {
                start: 0,
                end: list_len,
            }],
            cursor: 0,
            segment: 0,
        })
    }

    pub fn scroll(&mut self, amount: i64) {
        let pos = self.cursor as i64 + amount;
        if pos <= 0 {
            self.cursor = 0;
        } else if pos as usize >= self.list.len() {
            self.cursor = self.list.len() - 1;
        } else {
            self.cursor = pos as usize;
        }

        self.update_current_segment();
    }

    pub fn jump(&mut self, pos: usize) {
        if pos >= self.list.len() {
            self.cursor = self.list.len() - 1;
        } else {
            self.cursor = pos;
        }

        self.update_current_segment();
    }

    fn update_current_segment(&mut self) {
        let mut diff = std::usize::MAX;
        for (index, segment) in self.expanded.iter().enumerate() {
            if self.cursor >= segment.start
                && self.cursor < segment.end
                && (self.cursor - segment.start) < diff
            {
                diff = self.cursor - segment.start;
                self.segment = index;
            }
        }
    }

    fn update_segments(&mut self, after: usize, to_add: usize, to_subtract: usize) {
        for segment in &mut self.expanded {
            if segment.end > after {
                segment.end += to_add;
                segment.end -= to_subtract;
            }
            if segment.start > after {
                segment.start += to_add;
                segment.start -= to_subtract;
            }
        }
    }

    pub fn expand(&mut self) {
        // check to make sure segment is not alreaded expanded
        for segment in &self.expanded {
            if segment.start == self.cursor + 1 {
                return;
            }
        }

        // expand the selected segment
        let (mut to_insert, recursive) = self.list[self.cursor].expand();
        let insert_len = to_insert.len();

        // if insert is empty no more work is needed and new segment shouldn't be created
        if insert_len == 0 {
            return;
        }

        // add segment to list
        let insert_index = self.cursor + 1;
        while let Some(e) = to_insert.pop() {
            self.list.insert(insert_index, e);
        }

        // update segment list
        self.update_segments(self.cursor, insert_len, 0);

        // add new segment
        self.expanded.push(Segment {
            start: insert_index,
            end: insert_index + insert_len,
        });

        // recursively expand if specified
        if recursive {
            let to_expand = &self.expanded[self.expanded.len() - 1];
            for pos in (to_expand.start..to_expand.end).rev() {
                self.jump(pos);
                self.expand();
            }
        }
    }

    pub fn collapse(&mut self) {
        if self.segment == 0 {
            return; // cannot collapse root segment
        }

        // remove collapsed segment from expanded list
        let collapsing = self.expanded.remove(self.segment);

        // remove encompased segments from expanded list
        self.expanded
            .retain(|seg| !(seg.start > collapsing.start && seg.end <= collapsing.end));

        // remove segment from list
        for _ in collapsing.start..collapsing.end {
            self.list.remove(collapsing.start);
        }

        // update cursor
        self.cursor = collapsing.start - 1;

        // update segment list
        let remove_size = collapsing.end - collapsing.start;
        self.update_segments(collapsing.end - 1, 0, remove_size);

        // find and update current segment
        self.update_current_segment();
    }

    pub fn render(
        &mut self,
        colors: &Colors,
    ) -> Result<String, Box<dyn error::Error>> {
        let mut stream = String::new();
        // TODO: make scroll customizable
        // TODO: make application width aware
        let (width, height) = termion::terminal_size().unwrap();
        let midpoint = height as usize / 2;
        let mut index = if self.list.len() < height as usize {
            0
        } else if self.cursor <= midpoint {
            0
        } else if (self.list.len() - self.cursor) < midpoint {
            self.list.len() - height as usize
        } else {
            self.cursor - midpoint
        };
        write!(
            stream, 
            "{}{}{}", 
            colors.bg("default"),
            termion::clear::All, 
            termion::cursor::Goto(1, 1),
        );

        for line in 1..(height + 1) {
            let lc = self.list[index].counts();
            let adj_width = match &lc {
                Some(c) => width - c.char_width(),
                None => width,
            };

            if index == self.cursor {
                write!(
                    stream, 
                    "{}{}{}", 
                    colors.bg("highlight"),
                    termion::clear::CurrentLine,
                    colors.fg("highlight"),
                )?;
                self.list[index].highlight(&mut stream, &colors, line, adj_width)?;
                if let Some(c) = &lc {
                    c.highlight(&mut stream, &colors, line, width)?;
                }
            } else {
                write!(
                    &mut stream, 
                    "{}{}{}", 
                    colors.bg("default"), 
                    termion::clear::CurrentLine,
                    colors.fg("default"),
                )?;
                self.list[index].display(&mut stream, &colors, line, adj_width)?;
                if let Some(c) = &lc {
                    c.display(&mut stream, &colors, line, width)?;
                }
            }

            index += 1;
            if index >= self.list.len() { break; }
            write!(stream, "{}", termion::cursor::Goto(1, line + 1))?;
        }
        write!(stream, "{}{}", colors.bg("default"), colors.fg("default"))?;
        if DEBUG {
            write!(
                stream,
                "Lines: {}\tExpanded: {}\tCursor: {}\tSegment: {}",
                self.list.len(),
                self.expanded.len(),
                self.cursor,
                self.segment,
            )?;
            write!(stream, "{}", termion::cursor::Goto(1, height - 3))?;
            write!(stream, "{:?}", self.expanded)?;
            write!(stream, "{}", termion::cursor::Goto(1, height - 2))?;
        }
        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::super::Error;
    use super::*;
    use rand::Rng;

    struct Fold {
        id: usize,
    }

    impl Fold {
        fn n(id: usize) -> Fold {
            Fold { id }
        }
    }

    impl Expand for Fold {
        fn expand(&self) -> (Vec<Box<dyn Expand>>, bool) {
            (
                vec![
                    Box::new(Fold { id: self.id + 1 }),
                    Box::new(Fold { id: self.id + 2 }),
                ],
                false,
            )
        }
        fn id(&self) -> usize {
            self.id
        }
    }

    fn new_test_list() -> FoldingList {
        let folds: Vec<Box<dyn Expand>> = vec![
            Box::new(Fold::n(0)),
            Box::new(Fold::n(10)),
            Box::new(Fold::n(20)),
            Box::new(Fold::n(30)),
        ];
        FoldingList::new(folds).unwrap()
    }

    #[test]
    fn new_test() -> Result<(), Error> {
        let fl = new_test_list();
        assert_eq!(4, fl.list.len());
        assert_eq!(0, fl.cursor);
        assert_eq!(0, fl.segment);
        assert_eq!(0, fl.expanded[0].start);
        assert_eq!(4, fl.expanded[0].end);

        if let Ok(_fl) = FoldingList::new(Vec::new()) {
            panic!("FoldingList::new called on empty list should return an error");
        }
        Ok(())
    }

    #[test]
    fn scroll_test() {
        let mut fl = new_test_list();
        fl.scroll(-1);
        assert_eq!(0, fl.cursor);

        fl.scroll(5);
        assert_eq!(3, fl.cursor);

        fl.scroll(-2);
        assert_eq!(1, fl.cursor);
        fl.scroll(1);
        assert_eq!(2, fl.cursor);
    }

    #[test]
    fn jump() {
        let mut fl = new_test_list();

        fl.jump(10);
        assert_eq!(3, fl.cursor);
        fl.jump(1);
        assert_eq!(1, fl.cursor);
    }

    #[test]
    fn expand_test() {
        let mut fl = new_test_list();
        fl.cursor = 1;
        fl.expand();
        assert_eq!(6, fl.list.len());
        assert_eq!(10, fl.list[1].id());
        assert_eq!(11, fl.list[2].id());
        assert_eq!(12, fl.list[3].id());
        assert_eq!(20, fl.list[4].id());

        assert_eq!(2, fl.expanded.len());
        assert_eq!(0, fl.expanded[0].start);
        assert_eq!(6, fl.expanded[0].end);
        assert_eq!(2, fl.expanded[1].start);
        assert_eq!(4, fl.expanded[1].end);

        assert_eq!(0, fl.segment);

        fl.cursor = 2;
        fl.segment = 1;
        fl.expand();
        assert_eq!(8, fl.list.len());
        assert_eq!(10, fl.list[1].id());
        assert_eq!(11, fl.list[2].id());
        assert_eq!(12, fl.list[3].id());
        assert_eq!(13, fl.list[4].id());
        assert_eq!(12, fl.list[5].id());
        assert_eq!(20, fl.list[6].id());

        assert_eq!(3, fl.expanded.len());
        assert_eq!(0, fl.expanded[0].start);
        assert_eq!(8, fl.expanded[0].end);
        assert_eq!(2, fl.expanded[1].start);
        assert_eq!(6, fl.expanded[1].end);
        assert_eq!(3, fl.expanded[2].start);
        assert_eq!(5, fl.expanded[2].end);

        fl.cursor = 0;
        fl.segment = 0;
        fl.expand();
        assert_eq!(4, fl.expanded[1].start);

        fl.expand();
        assert_eq!(4, fl.expanded.len());
    }

    #[test]
    fn collapse_test() {
        let mut fl = new_test_list();
        fl.cursor = 1;
        fl.expand();

        fl.cursor = 2;
        fl.segment = 1;
        fl.expand();

        fl.cursor = 4;
        fl.segment = 2;
        fl.collapse();
        assert_eq!(6, fl.list.len());
        assert_eq!(10, fl.list[1].id());
        assert_eq!(11, fl.list[2].id());
        assert_eq!(12, fl.list[3].id());
        assert_eq!(20, fl.list[4].id());

        assert_eq!(2, fl.expanded.len());
        assert_eq!(0, fl.expanded[0].start);
        assert_eq!(6, fl.expanded[0].end);
        assert_eq!(2, fl.expanded[1].start);
        assert_eq!(4, fl.expanded[1].end);

        assert_eq!(2, fl.cursor);
        assert_eq!(1, fl.segment);

        fl.cursor = 5;
        fl.segment = 0;
        fl.expand();

        fl.cursor = 6;
        fl.segment = 2;
        fl.collapse();
        assert_eq!(5, fl.cursor);
        assert_eq!(0, fl.segment);

        // testing for a corner case I found
        fl.cursor = 3;
        fl.segment = 1;
        fl.expand();

        fl.cursor = 4;
        fl.segment = 2;
        fl.collapse();
        assert_eq!(6, fl.list.len());

        fl.cursor = 2;
        fl.segment = 1;
        fl.collapse();
        assert_eq!(4, fl.list.len());
    }

    #[test]
    fn test_collapse_children() {
        // test to make sure nested segments are all collapsed
        let mut fl = new_test_list();
        fl.jump(1);
        fl.expand();

        fl.jump(3);
        fl.expand();

        fl.jump(2);
        fl.collapse();
        assert_eq!(1, fl.expanded.len());
    }

    #[test]
    fn monkey_test() {
        // nothing in here should cause panic
        let mut rng = rand::thread_rng();
        let mut fl = new_test_list();
        for _ in 0..10000 {
            let function = rng.gen_range(0, 4);
            let limit = fl.expanded[0].end as i64;
            match function {
                0 => {
                    let amount: i64 = rng.gen_range(-limit, limit);
                    fl.scroll(amount);
                },
                1 => {
                    let pos: usize = rng.gen_range(0, limit) as usize;
                    fl.jump(pos);
                },
                2 => fl.expand(),
                3 => fl.collapse(),
                _ => ()
            }
        }
    }
}
