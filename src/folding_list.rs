pub struct FoldingList {
    list: Vec<Box<dyn Expand>>,
    expanded: Vec<Segment>,
    cursor: usize,
    segment: usize, // segment that that cursor is currently in
}

/// This structure stores the start and ends of a currently expanded segment.
struct Segment {
    start: usize, // inclusive
    end: usize, // exclusive
}

pub trait Expand {
    fn expand(&self) -> Vec<Box<dyn Expand>> { Vec::new() }
    fn display(&self) {}
    fn highlight(&self) {}
}

impl FoldingList {
    fn collapse(&mut self) {
        // remove collapsed segment from expanded list
        let collapsing = self.expanded.remove(self.segment);
        
        // remove segment from list
        for index in collapsing.start..collapsing.end {
            self.list.remove(index);
        }

        // update cursor
        self.cursor = collapsing.start;

        // update segment list
        let remove_size = collapsing.end - collapsing.start;
        for segment in &mut self.expanded {
            if segment.end < collapsing.end {
                segment.end -= remove_size;
            }
            if segment.start < collapsing.end {
                segment.start -= remove_size;
            }
        }

        // find and update current segment
        let mut diff = std::usize::MAX;
        for (index, segment) in self.expanded.iter().enumerate() {
            if self.cursor  <= segment.start && self.cursor > segment.end && (self.cursor - segment.start) < diff {
                diff = self.cursor - segment.start;
                self.segment = index;
            }
        }
        self.render();
    }

    fn expand(&mut self) {
        let to_insert = self.list[self.cursor].expand();
        let insert_index = self.cursor + 1;
        for element in to_insert {
            self.list.insert(insert_index, element);
        }
        self.render();
    }

    fn render(&self) {
        // TODO: make scroll customizable
        print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 1));
        // TODO: make application width aware
        let height = termion::terminal_size().unwrap().1 as usize;
        let start = if (self.cursor as i64 - (height / 2) as i64) < 0 { 0 } else { self.cursor - (height / 2) };
        let end = if (start + height) > self.list.len() { self.list.len() } else { start + height };

        for index in start..end {
            if index == self.cursor {
                self.list[index].highlight();
            } else {
                self.list[index].display();
            }
        }
    }
}

