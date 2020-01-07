use super::Error;
use std::{io::Stdout, error};
use crate::ColorSettings;

pub struct FoldingList {
    list: Vec<Box<dyn Expand>>,
    expanded: Vec<Segment>,
    cursor: usize,
    segment: usize, // segment that that cursor is currently in
    // these are both options to make testing better (I know it's not ideal)
    stream: Stdout,
    colors: ColorSettings,
    printing: bool,
}

/// This structure stores the start and ends of a currently expanded segment.
struct Segment {
    start: usize, // inclusive
    end: usize,   // exclusive
}

pub trait Expand {
    fn expand(&self) -> Vec<Box<dyn Expand>> {
        Vec::new()
    }
    fn display(&self, _stream: &mut Stdout, _colors: &ColorSettings) -> Result<(),Box<dyn error::Error>>{Ok(())}
    fn highlight(&self, _stream: &mut Stdout, _colors: &ColorSettings)-> Result<(),Box<dyn error::Error>>{Ok(())}
    fn id(&self) -> usize {
        0
    } // mostly for testing
}

impl FoldingList {
    pub fn new(list: Vec<Box<dyn Expand>>, stream: Stdout, colors: ColorSettings) -> Result<FoldingList, Error> {
        let mut fl = FoldingList::new_testing(list)?;
        fl.stream = stream;
        fl.colors = colors;
        fl.printing = true;
        Ok(fl)
    }

    fn new_testing(list: Vec<Box<dyn Expand>>) -> Result<FoldingList, Error> {
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
            stream: std::io::stdout(),
            colors: ColorSettings::default(),
            printing: false,
        })
    }

    pub fn scroll(&mut self, amount: i64) -> Result<(),Box<dyn error::Error>> {
        let pos = self.cursor as i64 + amount;
        if pos <= 0 {
            self.cursor = 0;
        } else if pos as usize >= self.list.len() {
            self.cursor = self.list.len() - 1;
        } else {
            self.cursor = pos as usize;
        }

        self.update_current_segment();
        self.render()?;
        Ok(())
    }

    pub fn jump(&mut self, pos: usize) -> Result<(),Box<dyn error::Error>>{
        if pos >= self.list.len() {
            self.cursor = self.list.len() - 1;
        } else {
            self.cursor = pos;
        }

        self.update_current_segment();
        self.render()?;
        Ok(())
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

    pub fn expand(&mut self) -> Result<(),Box<dyn error::Error>> {
        // expand the selected segment
        let mut to_insert = self.list[self.cursor].expand();
        let insert_len = to_insert.len();
        if insert_len == 0 {
            return Ok(());
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

        self.render()?;
        Ok(())
    }

    pub fn collapse(&mut self) -> Result<(),Box<dyn error::Error>>{
        if self.segment == 0 {
            return Ok(()); // cannot collapse root segment
        }

        // remove collapsed segment from expanded list
        let collapsing = self.expanded.remove(self.segment);

        // remove segment from list
        for _ in collapsing.start..collapsing.end {
            self.list.remove(collapsing.start);
        }

        // update cursor
        self.cursor = collapsing.start - 1;

        // update segment list
        let remove_size = collapsing.end - collapsing.start;
        self.update_segments(collapsing.end, 0, remove_size);

        // find and update current segment
        self.update_current_segment();

        self.render()?;
        Ok(())
    }

    pub fn render(&mut self) -> Result<(),Box<dyn error::Error>> {
        // don't print if in testing
        if !self.printing {
            return Ok(());
        }
        // TODO: make scroll customizable
        print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
        // TODO: make application width aware
        let height = termion::terminal_size().unwrap().1 as usize;
        let start = if (self.cursor as i64 - (height / 2) as i64) < 0 {
            0
        } else {
            self.cursor - (height / 2)
        };
        let end = if (start + height) > self.list.len() {
            self.list.len()
        } else {
            start + height
        };

        for index in start..end {
            if index == self.cursor {
                self.list[index].highlight(&mut self.stream, &self.colors)?;
            } else {
                self.list[index].display(&mut self.stream, &self.colors)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::Error;
    use super::*;

    struct Fold {
        id: usize,
    }

    impl Fold {
        fn n(id: usize) -> Fold {
            Fold { id }
        }
    }

    impl Expand for Fold {
        fn expand(&self) -> Vec<Box<dyn Expand>> {
            vec![
                Box::new(Fold { id: self.id + 1 }),
                Box::new(Fold { id: self.id + 2 }),
            ]
        }
        fn id(&self) -> usize {
            self.id
        }
    }

    #[test]
    fn new_test() -> Result<(), Error> {
        let folds: Vec<Box<dyn Expand>> = vec![
            Box::new(Fold::n(0)),
            Box::new(Fold::n(10)),
            Box::new(Fold::n(20)),
            Box::new(Fold::n(30)),
        ];
        let fl = FoldingList::new_testing(folds).unwrap();
        assert_eq!(4, fl.list.len());
        assert_eq!(0, fl.cursor);
        assert_eq!(0, fl.segment);
        assert_eq!(0, fl.expanded[0].start);
        assert_eq!(4, fl.expanded[0].end);

        if let Ok(_fl) = FoldingList::new_testing(Vec::new()) {
            panic!("FoldingList::new_testing called on empty list should return an error");
        }
        Ok(())
    }

    #[test]
    fn scroll_test() {
        let folds: Vec<Box<dyn Expand>> = vec![
            Box::new(Fold::n(0)),
            Box::new(Fold::n(10)),
            Box::new(Fold::n(20)),
            Box::new(Fold::n(30)),
        ];
        let mut fl = FoldingList::new_testing(folds).unwrap();
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
        let folds: Vec<Box<dyn Expand>> = vec![
            Box::new(Fold::n(0)),
            Box::new(Fold::n(10)),
            Box::new(Fold::n(20)),
            Box::new(Fold::n(30)),
        ];
        let mut fl = FoldingList::new_testing(folds).unwrap();

        fl.jump(10);
        assert_eq!(3, fl.cursor);
        fl.jump(1);
        assert_eq!(1, fl.cursor);
    }

    #[test]
    fn expand_test() {
        let folds: Vec<Box<dyn Expand>> = vec![
            Box::new(Fold::n(0)),
            Box::new(Fold::n(10)),
            Box::new(Fold::n(20)),
            Box::new(Fold::n(30)),
        ];
        let mut fl = FoldingList::new_testing(folds).unwrap();
        fl.cursor = 1;
        fl.expand().unwrap();
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
        fl.expand().unwrap();
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
        fl.expand().unwrap();
        assert_eq!(4, fl.expanded[1].start);
    }

    #[test]
    fn collapse_test() {
        let folds: Vec<Box<dyn Expand>> = vec![
            Box::new(Fold::n(0)),
            Box::new(Fold::n(10)),
            Box::new(Fold::n(20)),
            Box::new(Fold::n(30)),
        ];
        let mut fl = FoldingList::new_testing(folds).unwrap();
        fl.cursor = 1;
        fl.expand().unwrap();

        fl.cursor = 2;
        fl.segment = 1;
        fl.expand().unwrap();

        fl.cursor = 4;
        fl.segment = 2;
        fl.collapse().unwrap();
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
        fl.expand().unwrap();

        fl.cursor = 6;
        fl.segment = 2;
        fl.collapse().unwrap();
        assert_eq!(5, fl.cursor);
        assert_eq!(0, fl.segment);
    }
}
